use config::{ConfigError, Environment};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::ConnectOptions;
use sqlx::postgres::PgConnectOptions;
use uuid::Uuid;

use crate::email::EmailObject;
use crate::email_client::SmtpEmailClient;
use crate::pulsar_client::PulsarClient;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub name: String,
    pub test_name: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
}

impl DatabaseConfig {
    // Renamed from `connection_string_without_db`
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
    }
    // Renamed from `connection_string`
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.name)
            .log_statements(tracing::log::LevelFilter::Trace)
    }

    pub fn test_with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.test_name)
            .log_statements(tracing::log::LevelFilter::Trace)
    }
}
#[derive(Debug, Deserialize, Clone)]
pub struct ApplicationConfig {
    pub port: u16,
    pub host: String,
    pub workers: usize,
    pub service_id: Uuid,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Jwt {
    pub secret: SecretString,
    pub expiry: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OtpConfig {
    pub expiry: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecretConfig {
    pub jwt: Jwt,
    pub otp: OtpConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PulsarConfig {
    pub topic: String,
    pub consumer: String,
    pub subscription: String,
    pub url: String,
}

impl PulsarConfig {
    pub async fn client(self) -> Result<PulsarClient, pulsar::Error> {
        PulsarClient::new(self.url, self.topic).await
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub application: ApplicationConfig,
    pub secret: SecretConfig,
    pub user: UserConfig,
    pub email: EmailClientConfig,
    pub pulsar: PulsarConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserConfig {
    pub admin_list: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PersonalEmailClientConfig {
    pub message_id_suffix: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailClientConfig {
    pub base_url: String,
    pub username: String,
    pub password: SecretString,
    pub sender_email: String,
    pub timeout_milliseconds: u64,
    pub personal: PersonalEmailClientConfig,
}
impl EmailClientConfig {
    pub fn sender(&self) -> Result<EmailObject, String> {
        EmailObject::parse(self.sender_email.to_owned())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
    pub fn client(&self) -> SmtpEmailClient {
        SmtpEmailClient::new(self).expect("Failed to create SmtpEmailClient")
    }
    // pub fn personal_client(&self) -> SmtpEmailClient {
    //     SmtpEmailClient::new_personal().expect("Failed to create SmtpEmailClient")
    // }
}

pub fn get_configuration() -> Result<Config, ConfigError> {
    let builder = config::Config::builder()
        .add_source(Environment::default().separator("__"))
        .add_source(
            Environment::with_prefix("LIST")
                .try_parsing(true)
                .separator("__")
                .keep_prefix(false)
                .list_separator(","),
        )
        .build()?;
    builder.try_deserialize::<Config>()
}
