use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::routes::error_chain_fmt;
use crate::telemetry::spawn_blocking_with_tracing;
use actix_web::http::header::{HeaderMap, HeaderValue, WWW_AUTHENTICATE};
use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use anyhow::{anyhow, Context};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use std::fmt::{Debug, Formatter};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl Debug for PublishError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::AuthError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();

                response
                    .headers_mut()
                    .insert(WWW_AUTHENTICATE, header_value);

                response
            }
            PublishError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

#[tracing::instrument(name = "Publish newsletter issue", skip(body, pool, email_client))]
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    http_request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let credentials =
        basic_authentication(http_request.headers()).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
            AuthError::UnexpectedError(e) => PublishError::UnexpectedError(e),
        })?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    let subscribers = get_confirmed_subscribers(&pool).await?;

    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })?;
            }
            Err(error) => {
                tracing::warn!("Failed to parse email of a confirmed subscriber: {}", error);
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get all confirmed subscribers from the database", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
            SELECT email
            FROM subscriptions
            WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow!(error)),
    })
    .collect();

    Ok(confirmed_subscribers)
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("Missing Authorization header")?
        .to_str()
        .context("Authorization header is not valid UTF-8")?;

    let base64_encoded = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'")?;

    let decoded_bytes = STANDARD
        .decode(base64_encoded)
        .context("Failed to decode authorization header from base64")?;

    let decoded_string =
        String::from_utf8(decoded_bytes).context("The credentials is not valid UTF-8 string")?;

    let mut credentials = decoded_string.splitn(2, ":");
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing username"))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing password"))?
        .to_string();

    Ok(Credentials {
        username,
        password: SecretString::new(Box::from(password)),
    })
}

