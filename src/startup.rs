use crate::configuration::{Config, DatabaseConfig};

use crate::middlewares::SaveRequestResponse;
use crate::pulsar_client::PulsarTopic;
use crate::route::routes;
use crate::websocket_client;
use actix::Actor;
use actix_cors::Cors;
use actix_files::Files;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::net::TcpListener;
use std::time::Duration;

use tracing_actix_web::TracingLogger;
pub struct Application {
    port: u16,
    server: Server,
}
impl Application {
    pub async fn build(configuration: Config) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let address = format!(
            "{}:{}",
            &configuration.application.host, &configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool, configuration).await?;
        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseConfig) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(configuration.acquire_timeout))
        .max_connections(configuration.max_connections)
        .min_connections(configuration.min_connections)
        .connect_lazy_with(configuration.with_db())
}

async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    configuration: Config,
) -> Result<Server, anyhow::Error> {
    let db_pool = web::Data::new(db_pool);
    let secret_obj = web::Data::new(configuration.secret);
    let workers = configuration.application.workers;
    let application_obj = web::Data::new(configuration.application);
    let user_obj = web::Data::new(configuration.user);
    let ws_server = web::Data::new(websocket_client::Server::new().start());
    let email_client = web::Data::new(configuration.email.client());
    let email_config = web::Data::new(configuration.email);
    let slack_client = web::Data::new(configuration.slack.client());
    let pulsar_client = configuration.pulsar.client().await?;
    pulsar_client
        .start_ws_consumer(
            "ws_consumer",
            "ws_subscription",
            db_pool.clone(),
            &pulsar_client.get_topic_name(PulsarTopic::WebSocket),
            ws_server.clone(),
        )
        .await;

    pulsar_client
        .start_scheduler_consumer(
            "scheduler_consumer",
            "scheduler_subscription",
            db_pool.clone(),
            &pulsar_client.get_topic_name(PulsarTopic::Scheduler),
            slack_client.clone(),
        )
        .await;

    let pulsar_client_data = web::Data::new(pulsar_client);
    let governor_config = GovernorConfigBuilder::default()
        .seconds_per_request(10)
        .burst_size(12)
        .finish()
        .unwrap();

    let server = HttpServer::new(move || {
        App::new()
            //.app_data(web::JsonConfig::default().limit(1024 * 1024 * 50))
            .service(Files::new("/static", "static").show_files_listing())
            .wrap(SaveRequestResponse)
            .wrap(Cors::permissive())
            .wrap(TracingLogger::default())
            .wrap(Governor::new(&governor_config))
            .app_data(db_pool.clone())
            .app_data(secret_obj.clone())
            .app_data(application_obj.clone())
            .app_data(user_obj.clone())
            .app_data(ws_server.clone())
            .app_data(email_client.clone())
            .app_data(email_config.clone())
            // .app_data(pulsar_producer.clone())
            .app_data(slack_client.clone())
            .app_data(pulsar_client_data.clone())
            .configure(routes)
    })
    .workers(workers)
    .listen(listener)?
    .run();

    Ok(server)
}
