use newsletter::configuration::get_configuration;
use newsletter::startup::Application;
use newsletter::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // tracing/telemetry setup
    let subscriber = get_subscriber("newsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // get configuration for the application
    let configuration = get_configuration().expect("Failed to read configuration.");

    // build the application and run the server
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
