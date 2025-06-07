use secrecy::ExposeSecret;

use crate::{
    configuration::Config, configuration::get_configuration,
    routes::user::utils::generate_jwt_token_for_user, utils::configure_database,
};

#[tracing::instrument(name = "Default Migration")]
pub async fn run_migrations() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    configure_database(&configuration.database).await;
}

#[tracing::instrument(name = "Generate user token")]
pub async fn generate_user_token(configuration: &Config) {
    let token = generate_jwt_token_for_user(
        configuration.application.service_id,
        configuration.secret.jwt.expiry,
        &configuration.secret.jwt.secret,
    )
    .map_err(|e| anyhow::anyhow!("JWT generation error: {}", e));
    eprint!(
        "Token for {} is: {}",
        configuration.application.service_id,
        token.unwrap().expose_secret()
    )
}

#[tracing::instrument(name = "Run custom command")]
pub async fn run_custom_commands(
    configuration: &Config,
    args: Vec<String>,
) -> Result<(), anyhow::Error> {
    if args.len() > 1 {
        if args[1] == "migrate" {
            run_migrations().await;
        } else if args[1] == "generate_token" {
            generate_user_token(configuration).await;
        }
    } else {
        eprintln!("Invalid command. Please enter a valid command.");
    }

    Ok(())
}
