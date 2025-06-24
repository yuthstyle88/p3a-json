use actix::prelude::*;
use lapin::Channel;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use lapin::message::Delivery;
use lapin::options::BasicAckOptions;

use crate::telemetry_event::{insert_events, models::TelemetryEvent};


#[derive(Clone)]
pub struct AppContext {
    pub pool: sqlx::PgPool,
    pub brave_service_key: String,
    pub rabbit_channel: Arc<Channel>,
}

pub struct RabbitMqWorker {
    pub channel: Arc<Channel>,
    pub pool: sqlx::PgPool,
    pub buffer: Vec<Delivery>,
}

struct DeliveryMessage(pub Delivery);

impl actix::Message for DeliveryMessage {
    type Result = ();
}

impl Handler<DeliveryMessage> for RabbitMqWorker {
    type Result = ();

    fn handle(&mut self, msg: DeliveryMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.buffer.push(msg.0);

        if self.buffer.len() >= 100 {
            let deliveries = std::mem::take(&mut self.buffer);
            let pool = self.pool.clone();

            // สมมติคุณมี pub async fn insert_events(pool: &PgPool, events: &[TelemetryEvent]) -> Result<(), Error>
            actix::spawn(async move {
                let mut events = Vec::with_capacity(deliveries.len());
                for delivery in &deliveries {
                    if let Ok(data) = serde_json::from_slice::<TelemetryEvent>(&delivery.data) {
                        events.push(data);
                    }
                    // ถ้า deserialize ไม่ได้ก็สามารถ ack หรือ log ตามต้องการ
                }
                if let Err(e) = insert_events(&pool, &events).await {
                    eprintln!("Failed to insert events: {:?}", e);
                }
                // ack ทุกอันหลังจาก insert (ถ้าจำเป็น)
                for delivery in deliveries {
                    delivery.ack(BasicAckOptions::default()).await.expect("Ack failed");
                }
            });
        }
    }
}

impl Actor for RabbitMqWorker {
    type Context = Context<Self>;
    
}
