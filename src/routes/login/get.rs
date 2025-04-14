use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryParams {
    error: Option<String>,
}

pub async fn login_form(query: web::Query<QueryParams>) -> HttpResponse {
    let error = query.0.error;
    let error_html = match error {
        None => "".into(),
        Some(error) => format!("<p><i>{}</i><>/p", htmlescape::encode_minimal(&error)),
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta http-equiv="content-type" content="text/html; charset=utf-8">
                    <title>Login</title>
                </head>
                <body>
                    <form action="/login" method="post">
                        {}
                        <label>
                            Username
                            <input type="text" placeholder="Enter Username" name="username">
                        </label>

                        <label>
                            Password
                            <input type="password" placeholder="Enter Password" name="password">
                        </label>

                        <button type="submit">Login</button>
                    </form>
                </body>
                </html>
            "#,
            error_html
        ))
}
