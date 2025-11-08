use newsletter::configuration::get_configuration;
use newsletter::issue_delivery_worker::run_worker_until_stopped;
use newsletter::startup::Application;
use newsletter::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing/telemetry setup
    let subscriber = get_subscriber("newsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // get configuration for the application
    let configuration = get_configuration().expect("Failed to read configuration.");

    // build the application and run the server
    let application = Application::build(configuration.clone())
        .await?
        .run_until_stopped();
    let worker = run_worker_until_stopped(configuration);

    // All selected futures are polled on same task, concurrency not parallel.
    // Both run on the same thread, if one branch blocks the thread, all other expressions will be unable to continue
    // If want parallelism, run on separate threads
    tokio::select! {
        _ = application => {},
        _ = worker => {},
    };

    Ok(())
}
