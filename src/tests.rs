#[cfg(test)]
pub mod tests {

    use crate::startup::get_connection_pool;
    use crate::utils::get_configuration;
    use dotenv::dotenv;

    use sqlx::PgPool;

    pub async fn get_test_pool() -> PgPool {
        dotenv().ok();
        let mut configuration = get_configuration().expect("Failed to read configuration.");
        configuration.database.name = configuration.database.test_name.to_string();
        eprintln!("{}", configuration.database.name);
        configuration.application.port = 0;
        get_connection_pool(&configuration.database)
    }
}
