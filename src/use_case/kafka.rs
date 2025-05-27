use std::time::Duration;
use rdkafka::error::KafkaError;
use rdkafka::producer::FutureProducer;
use rdkafka::util::Timeout;
use tracing::{debug, error, warn};
use crate::errors::LibError;
use crate::errors::LibError::InternalError;

pub async fn send_kafka_message(producer: &FutureProducer, topic: &str, key: &str, payload: &[u8])
                                -> Result<(), LibError>
{
    for i in 0..3 {
        if i > 0 {
            warn!("Kafka retry send attempt {}", i);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let record = rdkafka::producer::FutureRecord::to(topic)
            .key(key)
            .payload(payload);
         match tokio::time::timeout(Duration::from_millis(300), producer.send(record, Timeout::Never)).await {
            Ok(Ok(f)) => {
                debug!("Sent kafka message {:?}", f);
                return Ok(())
            },
            Ok(Err((e, _))) => {
                warn!(err=e.to_string(), "Error sending kafka message. Retrying...");
                continue
            },
             Err(_) => {
                 warn!("sending kafka message timeout. Retrying...");
                 continue;
             }
        }
    }
    error!("Kafka send kafka message: Timeout");
    Err(InternalError)
}