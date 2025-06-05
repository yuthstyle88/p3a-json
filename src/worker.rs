use actix::prelude::*;
use lapin::{options::BasicConsumeOptions, types::FieldTable, Channel};
use futures_util::stream::StreamExt;
use std::sync::Arc;
use lapin::options::BasicAckOptions;

use crate::telemetry_event::{insert_event, models::TelemetryEvent};
use aws_sdk_dynamodb::Client as DynamoDbClient;


#[derive(Clone)]
pub struct AppContext {
    pub pool: sqlx::PgPool,
    pub brave_service_key: String,
    pub rabbit_channel: Arc<Channel>,
    pub dynamodb_client: DynamoDbClient,
}

pub struct RabbitMqWorker {
    pub channel: Arc<Channel>,
    pub pool: sqlx::PgPool,
}

impl Actor for RabbitMqWorker {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("RabbitMqWorker actor started");
        let channel = self.channel.clone();
        let pool = self.pool.clone();

        actix_rt::spawn(async move {
            let mut consumer = channel
                .basic_consume(
                    "my_queue",
                    "my_consumer",
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await
                .expect("Failed to start consumer");

            while let Some(delivery) = consumer.next().await {
                match delivery {
                    Ok(delivery) => {
                        // Deserialize เป็น TelemetryEvent
                        let data: TelemetryEvent = match serde_json::from_slice(&delivery.data) {
                            Ok(event) => event,
                            Err(e) => {
                                eprintln!("Failed to deserialize payload: {:?}", e);
                                delivery
                                    .ack(BasicAckOptions::default())
                                    .await
                                    .expect("Ack failed");
                                continue;
                            }
                        };
                        // ดึง pool จาก app_context
                        let pool = pool.clone();

                        if let Err(e) = insert_event(&pool, &data).await {
                            eprintln!("Failed to insert event: {:?}", e);
                        }

                        delivery
                            .ack(BasicAckOptions::default())
                            .await
                            .expect("Ack failed");
                    }
                    Err(err) => {
                        eprintln!("Actor error: {:?}", err);
                    }
                }
            }
            actix_rt::Arbiter::current().stop();
        });
    }
}