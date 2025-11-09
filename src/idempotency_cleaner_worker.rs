use std::time::Duration;

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};

use crate::{configuration::Settings, startup::get_connection_pool};

type PgTransaction = Transaction<'static, Postgres>;

#[allow(dead_code)]
struct IdempotentEntry {
    idempotency_key: String,
    created_at: DateTime<Utc>,
}

pub enum IdempotentExecutionOutcome {
    TaskCompleted,
    EmptyTable,
}

pub async fn run_idempotency_worker_until_stopped(
    configuration: Settings,
) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);

    worker_loop(connection_pool, configuration.idempotent_time_interval).await
}

async fn worker_loop(pool: PgPool, time_interval: f64) -> Result<(), anyhow::Error> {
    loop {
        match delete_expired_idempotent_entries(&pool, time_interval).await {
            Ok(IdempotentExecutionOutcome::EmptyTable) => {
                // Exponential backoff with jitter would be better
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Err(_) => {
                // Exponential backoff with jitter would be better
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Ok(IdempotentExecutionOutcome::TaskCompleted) => {}
        }
    }
}

#[tracing::instrument(skip_all)]
pub async fn delete_expired_idempotent_entries(
    pool: &PgPool,
    time_interval: f64,
) -> Result<IdempotentExecutionOutcome, anyhow::Error> {
    let (mut transaction, records) = get_idempotent_entries(pool, time_interval).await?;
    if records.is_empty() {
        return Ok(IdempotentExecutionOutcome::EmptyTable);
    }

    let idempotency_keys: Vec<String> = records.iter().map(|r| r.idempotency_key.clone()).collect();

    sqlx::query!(
        r#"
        DELETE FROM idempotency
        WHERE idempotency_key = ANY($1)
        "#,
        &idempotency_keys
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(IdempotentExecutionOutcome::TaskCompleted)
}

#[tracing::instrument(skip_all)]
async fn get_idempotent_entries(
    pool: &PgPool,
    time_interval: f64,
) -> Result<(PgTransaction, Vec<IdempotentEntry>), anyhow::Error> {
    let mut transaction = pool.begin().await?;
    let records = sqlx::query_as!(
        IdempotentEntry,
        r#"
        SELECT idempotency_key, created_at
        FROM idempotency
        WHERE created_at + ($1 * INTERVAL '1 second') < now()
        FOR UPDATE SKIP LOCKED
        LIMIT 100
        "#,
        time_interval
    )
    .fetch_all(&mut *transaction)
    .await?;

    Ok((transaction, records))
}
