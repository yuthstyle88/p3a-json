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
            .route("/", web::post().to(insert_event))
    ).await;

    // Test data
    let test_event = common::get_test_event();

    // Execute
    let req = test::TestRequest::post()
        .uri("/")
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
            .route("/", web::post().to(insert_event))
    ).await;

    let test_event = common::get_test_event();

    // Test without auth header
    let req = test::TestRequest::post()
        .uri("/")
        .set_json(&test_event)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Test with wrong auth key
    let req = test::TestRequest::post()
        .uri("/")
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
            .route("/", web::post().to(insert_event))
    ).await;

    // Test invalid JSON
    let req = test::TestRequest::post()
        .uri("/")
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
        .uri("/")
        .insert_header(("BraveServiceKey", "test_key"))
        .set_json(&test_event)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}
#[actix_web::test]
async fn test_telemetry_endpoint_insert() {
    // Setup
    let pool = common::setup_test_db().await;
    let app = test::init_service(
        App::new()
            .wrap(AuthMiddleware::new("test_key".to_string()))
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::post().to(insert_event))
    ).await;

    // Test Case 1: Insert complete event
    let test_event = TelemetryEvent {
        id: None,
        cadence: "daily".to_string(),
        channel: "stable".to_string(),
        country_code: "US".to_string(),
        metric_name: "test_metric".to_string(),
        metric_value: 100,
        platform: "macos".to_string(),
        version: "1.0.0".to_string(),
        woi: 1,
        wos: 2,
        yoi: 2024,
        yos: 2024,
        received_at: None,
    };

    let req = test::TestRequest::post()
        .uri("/")
        .insert_header(("BraveServiceKey", "test_key"))
        .set_json(&test_event)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify all fields were saved correctly
    let saved_event = sqlx::query_as!(
        TelemetryEvent,
        "SELECT * FROM telemetry_events WHERE metric_name = $1",
        test_event.metric_name
    )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch saved event");

    assert!(saved_event.id.is_some());
    assert_eq!(saved_event.cadence, test_event.cadence);
    assert_eq!(saved_event.channel, test_event.channel);
    assert_eq!(saved_event.country_code, test_event.country_code);
    assert_eq!(saved_event.metric_name, test_event.metric_name);
    assert_eq!(saved_event.metric_value, test_event.metric_value);
    assert_eq!(saved_event.platform, test_event.platform);
    assert_eq!(saved_event.version, test_event.version);
    assert_eq!(saved_event.woi, test_event.woi);
    assert_eq!(saved_event.wos, test_event.wos);
    assert_eq!(saved_event.yoi, test_event.yoi);
    assert_eq!(saved_event.yos, test_event.yos);
    assert!(saved_event.received_at.is_some()); // Should be automatically set

    // Test Case 2: Test with different values
    let second_event = TelemetryEvent {
        id: None,
        cadence: "weekly".to_string(),
        channel: "beta".to_string(),
        country_code: "JP".to_string(),
        metric_name: "second_metric".to_string(),
        metric_value: 200,
        platform: "windows".to_string(),
        version: "2.0.0".to_string(),
        woi: 5,
        wos: 6,
        yoi: 2024,
        yos: 2024,
        received_at: None,
    };

    let req = test::TestRequest::post()
        .uri("/")
        .insert_header(("BraveServiceKey", "test_key"))
        .set_json(&second_event)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify the second event
    let saved_second = sqlx::query_as!(
        TelemetryEvent,
        "SELECT * FROM telemetry_events WHERE metric_name = $1",
        second_event.metric_name
    )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch second event");

    assert!(saved_second.id.is_some());
    assert_eq!(saved_second.cadence, second_event.cadence);
    assert_eq!(saved_second.channel, second_event.channel);
    assert_eq!(saved_second.country_code, second_event.country_code);
    assert_eq!(saved_second.metric_name, second_event.metric_name);
    assert_eq!(saved_second.metric_value, second_event.metric_value);
    assert_eq!(saved_second.platform, second_event.platform);
    assert_eq!(saved_second.version, second_event.version);
    assert_eq!(saved_second.woi, second_event.woi);
    assert_eq!(saved_second.wos, second_event.wos);
    assert_eq!(saved_second.yoi, second_event.yoi);
    assert_eq!(saved_second.yos, second_event.yos);
    assert!(saved_second.received_at.is_some());
}