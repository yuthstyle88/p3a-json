use actix::Addr;
use actix_web::{web, HttpResponse, Responder};
use crate::payload::MyPayload;
use crate::worker::{DeliveryMessage, ActorWorker};

pub async fn queue_job(
    ctx: web::Data<Addr<ActorWorker>>,
    item: web::Json<MyPayload>,
) -> impl Responder {
    let payload = item.into_inner();
    let addr = ctx.get_ref();
    let data = DeliveryMessage { 0: payload };
    match addr.send(data).await {
        Ok(_) => HttpResponse::Ok().json("Job queued"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to queue job")
    }
}