use actix_web::{web, HttpResponse, Responder};
use lapin::BasicProperties;
use lapin::options::BasicPublishOptions;
use crate::payload::MyPayload;
use crate::worker::AppContext;

pub async fn queue_job(
    ctx: web::Data<AppContext>,
    item: web::Json<MyPayload>,
    path: web::Path<String>,
) -> impl Responder {
    let worker = ctx.rabbit_channel.clone();
    let payload = item.into_inner();
    let path = path.into_inner();
    // Serialize payload เป็น JSON
    let payload_bytes = match serde_json::to_vec(&payload) {
        Ok(val) => val,
        Err(_) => return HttpResponse::InternalServerError().body("Payload serialization failed"),
    };

    // ส่ง payload เข้า queue แบบ async, หลีกเลี่ยง block thread
    let worker_clone = worker.clone();
    let payload_bytes = payload_bytes; // rename for move clarity
    tokio::spawn(async move {
        let publish = worker_clone.basic_publish(
            "",
            "my_queue",
            BasicPublishOptions::default(),
            &payload_bytes,
            BasicProperties::default(),
        );
        if let Err(e) = publish.await {
            eprintln!("Failed to publish to queue: {:?}", e);
        }
    });

    HttpResponse::Ok().json("Job queued") // ส่งกลับว่า queue สำเร็จ
}
