use crate::configuration::{Config, DatabaseConfig};
use crate::middlewares::SaveRequestResponse;
use crate::route::routes;
use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::net::TcpListener;
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
        .acquire_timeout(std::time::Duration::from_secs(
            configuration.acquire_timeout,
        ))
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
    let server = HttpServer::new(move || {
        App::new()
            //.app_data(web::JsonConfig::default().limit(1024 * 1024 * 50))
            .wrap(SaveRequestResponse)
            .wrap(Cors::permissive())
            .wrap(TracingLogger::default())
            .app_data(db_pool.clone())
            .app_data(secret_obj.clone())
            .app_data(application_obj.clone())
            .app_data(user_obj.clone())
            .configure(routes)
    })
    .workers(workers)
    .listen(listener)?
    .run();

    Ok(server)
}
