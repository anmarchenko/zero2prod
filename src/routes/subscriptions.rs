use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use log::{error, info};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> impl Responder {
    let request_id = Uuid::new_v4();
    info!(
        "request_id {} - Adding '{}' '{}' a new subscriber",
        request_id, form.email, form.name
    );

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    // get_ref is to unwrap Arc
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );

            HttpResponse::Ok().finish()
        }
        Err(e) => {
            error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id, e
            );
            HttpResponse::InternalServerError().body(format!("{}", e))
        }
    }
}
