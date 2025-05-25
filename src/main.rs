use vitis::commands::run_custom_commands;
use vitis::startup::Application;
use vitis::telemetry::{get_subscriber_with_jeager, init_subscriber};
use vitis::utils::get_configuration;
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let configuration = get_configuration().expect("Failed to read configuration.");
    if args.len() > 1 {
        run_custom_commands(&configuration, args).await?;
    } else {
        let subscriber =
            get_subscriber_with_jeager("ondc-user".into(), "info".into(), std::io::stdout);
        init_subscriber(subscriber);
        let application = Application::build(configuration).await?;
        application.run_until_stopped().await?;
    }
    Ok(())
}
