use actix_web::{HttpResponse, ResponseError, http::StatusCode, web};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, pool))]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, SubscribeConfirmError> {
    // get subscriber_id using token from the database
    let subscriber_id = get_subscriber_id_from_token(&pool, &parameters.subscription_token)
        .await
        .context("Failed to get subscriber id associated with the provided token.")? // context also does the work of map_err + add additional info
        .ok_or(SubscribeConfirmError::SubscribeTokenError)?;

    // mark status as confirmed in subscriptions table for this subscriber id
    confirm_subscriber(&pool, subscriber_id)
        .await
        .context("Failed to update subscription status to succeeded.")?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1
        "#,
        subscription_token,
    )
    .fetch_optional(pool)
    .await?;

    // maps Option<Record> -> Option<Uuid>
    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions SET status = 'confirmed' WHERE id = $1
        "#,
        subscriber_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(thiserror::Error)]
pub enum SubscribeConfirmError {
    #[error("There is no subscriber associated with the provided token.")]
    SubscribeTokenError,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

fn error_chain_fmt(
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

impl std::fmt::Debug for SubscribeConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscribeConfirmError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            SubscribeConfirmError::SubscribeTokenError => StatusCode::UNAUTHORIZED,
            SubscribeConfirmError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
