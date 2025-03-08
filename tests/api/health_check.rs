use crate::helpers::spawn_app;

#[actix_web::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("failed to send health check request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
