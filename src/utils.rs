use std::{fs, io};

use crate::configuration::DatabaseConfig;
use crate::errors::CustomJWTTokenError;
use crate::models::NotficationDataModel;
use crate::routes::user::schemas::JWTClaims;
use crate::websocket_client::MessageToClient;
use actix_web::dev::ServiceRequest;
use actix_web::rt::task::JoinHandle;
use chrono::Utc;
use core::str;
use jsonwebtoken::{Algorithm as JWTAlgorithm, DecodingKey, Validation, decode};
use secrecy::{ExposeSecret, SecretString};
use serde_json::Value;
use sqlx::types::Json;
use sqlx::{Connection, Executor, PgConnection, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument(name = "Decode JWT token")]
pub fn decode_token<T: Into<String> + std::fmt::Debug>(
    token: T,
    secret: &SecretString,
) -> Result<Uuid, CustomJWTTokenError> {
    let decoding_key = DecodingKey::from_secret(secret.expose_secret().as_bytes());
    let decoded = decode::<JWTClaims>(
        &token.into(),
        &decoding_key,
        &Validation::new(JWTAlgorithm::HS256),
    );
    match decoded {
        Ok(token) => Ok(token.claims.sub),
        Err(e) => match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => Err(CustomJWTTokenError::Expired),
            _ => Err(CustomJWTTokenError::Invalid("Invalid Token".to_string())),
        },
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

#[tracing::instrument(name = "Execute Queries")]
async fn execute_query(path: &str, pool: &PgPool) -> io::Result<()> {
    let migration_files = fs::read_dir(path)?;
    for migration_file in migration_files {
        let migration_file = migration_file?;
        let migration_path = migration_file.path();
        let migration_sql = fs::read_to_string(&migration_path)?;
        let statements: String = migration_sql.replace('\n', "");
        let new_statement: Vec<&str> = statements
            .split(';')
            .filter(|s| !s.trim().is_empty() & !s.starts_with("--"))
            .collect();
        for statement in new_statement {
            if let Err(err) = sqlx::query(statement).execute(pool).await {
                eprintln!("Error executing statement {:?}: {} ", statement, err);
            } else {
                eprintln!("Migration applied: {:?}", statement);
            }
        }

        eprintln!("Migration applied: {:?}", migration_path);
    }

    Ok(())
}

#[tracing::instrument(name = "Create Database")]
pub async fn create_database(config: &DatabaseConfig) {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    let db_count: Result<Option<i64>, sqlx::Error> =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM pg_database WHERE datname = $1")
            .bind(&config.name)
            .fetch_optional(&mut connection)
            .await;

    match db_count {
        Ok(Some(count)) => {
            if count > 0 {
                tracing::info!("Database {} already exists.", &config.name);
            } else {
                connection
                    .execute(format!(r#"CREATE DATABASE "{}";"#, config.name).as_str())
                    .await
                    .expect("Failed to create database.");
                eprintln!("Database created.");
            }
        }
        Ok(_) => eprintln!("No rows found."),
        Err(err) => eprintln!("Error: {}", err),
    }

    let test_db_count: Result<Option<i64>, sqlx::Error> =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM pg_database WHERE datname = $1")
            .bind(&config.test_name)
            .fetch_optional(&mut connection)
            .await;

    match test_db_count {
        Ok(Some(count)) => {
            if count > 0 {
                eprintln!("Test database {} already exists.", &config.test_name);
            } else {
                connection
                    .execute(format!(r#"CREATE DATABASE "{}";"#, config.test_name).as_str())
                    .await
                    .expect("Failed to create test database.");
                eprintln!("Test database {} created.", &config.test_name);
            }
        }
        Ok(_) => eprintln!("No rows found for the test database check."),
        Err(err) => eprintln!("Error checking test database existence: {}", err),
    }
}

#[tracing::instrument(name = "Confiure Database")]
pub async fn configure_database(config: &DatabaseConfig) -> PgPool {
    create_database(config).await;
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    let test_connection_pool = PgPool::connect_with(config.test_with_db())
        .await
        .expect("Failed to connect to Postgres.");

    let _ = execute_query("./migrations", &connection_pool).await;
    let _ = execute_query("./migrations", &test_connection_pool).await;
    connection_pool
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    actix_web::rt::task::spawn_blocking(move || current_span.in_scope(f))
}

pub fn pascal_to_snake_case(pascal_case: &str) -> String {
    let mut snake_case = String::new();
    let mut is_first_word = true;

    for c in pascal_case.chars() {
        if c.is_uppercase() {
            if !is_first_word {
                snake_case.push('_');
            }
            is_first_word = false;
        }
        snake_case.push(c.to_ascii_lowercase());
    }

    snake_case
}

#[tracing::instrument(name = "Get header value")]
pub fn get_header_value<'a>(req: &'a ServiceRequest, header_name: &'a str) -> Option<&'a str> {
    req.headers().get(header_name).and_then(|h| h.to_str().ok())
}

pub fn to_title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(f) => f.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn snake_to_title_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(f) => f.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[tracing::instrument(name = "save_notification_to_database", skip(pool))]
pub async fn save_notification_to_database(
    pool: &PgPool,
    websocket_id: &str,
    data: &Value,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        INSERT INTO pending_notification (id, data, connection_id,  created_on)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        data,
        websocket_id,
        Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while saving ONDC order request")
    })?;
    Ok(())
}

#[tracing::instrument(name = "fetch_notifications_by_connection_id", skip(transaction))]
pub async fn fetch_notifications_by_connection_id(
    transaction: &mut Transaction<'_, Postgres>,
    connection_id: &str,
) -> Result<Vec<NotficationDataModel>, anyhow::Error> {
    let records = sqlx::query_as!(
        NotficationDataModel,
        r#"
        SELECT data as "data: Json<MessageToClient>", connection_id
        FROM pending_notification
        WHERE connection_id = $1 ORDER BY created_on
        FOR UPDATE
        "#,
        connection_id
    )
    .fetch_all(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow::Error::new(e).context("A database failure occurred while fetching notifications")
    })?;

    Ok(records)
}

#[tracing::instrument(name = "delete_notifications_by_connection_id", skip(transaction))]
pub async fn delete_notifications_by_connection_id(
    transaction: &mut Transaction<'_, Postgres>,
    connection_id: &str,
) -> Result<u64, anyhow::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM pending_notification
        WHERE connection_id = $1
        "#,
        connection_id
    )
    .execute(&mut **transaction) // Explicitly dereference the transaction
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute delete query: {:?}", e);
        anyhow::Error::new(e).context("Failed to delete notifications for the given connection_id")
    })?;

    Ok(result.rows_affected()) // Return the number of rows deleted
}
