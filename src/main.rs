use std::fmt::{Debug, Display};

use newsletter::configuration::get_configuration;
use newsletter::idempotency_cleaner_worker::run_idempotency_worker_until_stopped;
use newsletter::issue_delivery_worker::run_worker_until_stopped;
use newsletter::startup::Application;
use newsletter::telemetry::{get_subscriber, init_subscriber};
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing/telemetry setup
    let subscriber = get_subscriber("newsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // get configuration for the application
    let configuration = get_configuration().expect("Failed to read configuration.");

    // build the application and run the server
    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());

    // worker task
    let worker_task = tokio::spawn(run_worker_until_stopped(configuration.clone()));
    let idempotency_cleaner_task =
        tokio::spawn(run_idempotency_worker_until_stopped(configuration));

    // All selected futures are polled on same task, concurrency not parallel.
    // Both run on the same thread, if one branch blocks the thread, all other expressions will be unable to continue
    // If want parallelism, run on separate threads
    tokio::select! {
        o = application_task => report_exit("API", o),
        o = worker_task => report_exit("Background worker", o),
        o = idempotency_cleaner_task => report_exit("Idempotency worker", o),
    };

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} task failed to complete",
                task_name
            )
        }
    }
}
