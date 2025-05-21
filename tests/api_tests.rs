mod common;

use actix_web::{test, web, App};
use telemetry_events::{AuthMiddleware, insert_event, TelemetryEvent};
use serde_json::json;

#[actix_web::test]
async fn test_telemetry_endpoint_success() {
    // Setup
    let pool = common::setup_test_db().await;
    let app = test::init_service(
        App::new()
            .wrap(AuthMiddleware::new("test_key".to_string()))
            .app_data(web::Data::new(pool.clone()))
            .route("/telemetry", web::post().to(insert_event))
    ).await;

    // Test data
    let test_event = common::get_test_event();

    // Execute
    let req = test::TestRequest::post()
        .uri("/telemetry")
        .insert_header(("BraveServiceKey", "test_key"))
        .set_json(&test_event)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Verify
    assert!(resp.status().is_success());

    // Verify database
    let saved_event = sqlx::query_as!(
        TelemetryEvent,
        "SELECT * FROM telemetry_events WHERE metric_name = $1",
        test_event.metric_name
    )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch saved event");

    assert_eq!(saved_event.metric_name, test_event.metric_name);
    assert_eq!(saved_event.metric_value, test_event.metric_value);
}

#[actix_web::test]
async fn test_telemetry_endpoint_unauthorized() {
    let pool = common::setup_test_db().await;
    let app = test::init_service(
        App::new()
            .wrap(AuthMiddleware::new("test_key".to_string()))
            .app_data(web::Data::new(pool.clone()))
            .route("/telemetry", web::post().to(insert_event))
    ).await;

    let test_event = common::get_test_event();

    // Test without auth header
    let req = test::TestRequest::post()
        .uri("/telemetry")
        .set_json(&test_event)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Test with wrong auth key
    let req = test::TestRequest::post()
        .uri("/telemetry")
        .insert_header(("BraveServiceKey", "wrong_key"))
        .set_json(&test_event)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_telemetry_endpoint_validation() {
    let pool = common::setup_test_db().await;
    let app = test::init_service(
        App::new()
            .wrap(AuthMiddleware::new("test_key".to_string()))
            .app_data(web::Data::new(pool.clone()))
            .route("/telemetry", web::post().to(insert_event))
    ).await;

    // Test invalid JSON
    let req = test::TestRequest::post()
        .uri("/telemetry")
        .insert_header(("BraveServiceKey", "test_key"))
        .set_json(json!({
            "metric_name": "test",
            // missing required fields
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // Test invalid metric_value
    let mut test_event = common::get_test_event();
    test_event.metric_value = -1; // assuming negative values are invalid

    let req = test::TestRequest::post()
        .uri("/telemetry")
        .insert_header(("BraveServiceKey", "test_key"))
        .set_json(&test_event)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}