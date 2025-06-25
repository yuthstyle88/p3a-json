use actix::prelude::*;
use std::sync::Arc;
use crate::models::DBPool;
use crate::payload::MyPayload;
use crate::telemetry_event::insert_events;

#[derive(Clone)]
pub struct ActorWorker {
    pub pool: Arc<DBPool>,
    pub buffer: Vec<MyPayload>,
}

pub struct DeliveryMessage(pub MyPayload);

impl actix::Message for DeliveryMessage {
    type Result = ();
}

impl Handler<DeliveryMessage> for ActorWorker {
    type Result = ();

    fn handle(&mut self, msg: DeliveryMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.buffer.push(msg.0);

        if self.buffer.len() >= 100 {
            let pool = self.pool.clone();
            let buffer = self.buffer.clone();
            actix::spawn(async move {
                if let Err(e) = insert_events(pool.clone(), buffer).await {
                    eprintln!("Failed to insert events: {:?}", e);
                }
            });
            self.buffer.clear();
        }
    }
}

impl Actor for ActorWorker {
    type Context = Context<Self>;
    
}