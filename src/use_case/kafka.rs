use std::time::Duration;
use rdkafka::producer::FutureProducer;
use tracing::{debug, warn};


pub async fn send_kafka_message(producer: &FutureProducer, topic: &str, key: &str, payload: &[u8]) {
    for i in 0..3 {
        if i > 0 {
            warn!("Kafka retry send attempt {}", i);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let record = rdkafka::producer::FutureRecord::to(topic)
            .key(key)
            .payload(payload);
        match producer.send(record, Duration::from_secs(5)).await {
            Ok(f) => {
                debug!("Sent kafka message {:?}", f);
                break
            },
            Err((e, _)) => {
                warn!(err=e.to_string(), "Error sending kafka message");
                continue
            }
        }
    }
}