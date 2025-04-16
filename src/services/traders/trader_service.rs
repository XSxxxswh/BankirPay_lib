use std::str::FromStr;
use std::time::Duration;

use deadpool::managed::{Metrics, Object, Pool, RecycleResult};
use prost::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use tokio::time::{sleep, Instant};
use tonic::{Request};
use tonic::transport::{Channel, Endpoint};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use crate::errors::LibError;
use crate::errors::LibError::InternalError;

use crate::services::{need_retry, status_to_err};
use crate::trader_proto;

const RETRY_COUNT: i32 = 3;
pub const TRADER_CHANGE_BALANCE_TOPIC: &'static str = "trader_change_balance";


pub async fn send_trader_change_balance_event(producer: FutureProducer, balance_request: trader_proto::ChangeBalanceRequest)
{
    let buff = balance_request.encode_to_vec();
    let record = FutureRecord::to(TRADER_CHANGE_BALANCE_TOPIC)
        .payload(&buff)
        .key(balance_request.trader_id.as_str());
    match producer.send(record, Duration::from_secs(5)).await {
        Ok(delivery) => debug!("Kafka sent: {:?}", delivery),
        Err((e, _)) => error!("Kafka send error: {:?}", e),
    }
}


#[derive(Clone)]
pub struct TraderService {
    client: trader_proto::trader_service_client::TraderServiceClient<Channel>,
}

impl TraderService {
    pub async fn new(addr : String) -> Self {

        let channel = Endpoint::from_str(addr.as_str())
            .unwrap()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .tcp_keepalive(Some(Duration::from_secs(10))).connect()
            .await
            .unwrap();

        let client = trader_proto::trader_service_client::TraderServiceClient::new(channel);
        info!("trader service connected");
        Self { client }
    }

    pub async fn change_balance(&mut self, trader_id: String, amount: f64, action_type: trader_proto::BalanceActionType) -> Result<(), LibError> {
        let idempotent_key = Uuid::now_v7().to_string();
        let request = trader_proto::ChangeBalanceRequest {
            trader_id,
            amount,
            action_type: action_type as i32,
            idempotent_key,
        };
        let start = Instant::now();
        for _ in 0..RETRY_COUNT {
            match self.client.change_balance(Request::new(request.clone())).await {
                Ok(result) => {
                    if start.elapsed().as_millis() > 200 {
                        warn!("[GRPC] Slow changing balance {:?}", start.elapsed());
                    }
                    return Ok(result.into_inner())
                },
                Err(e) if need_retry(e.code()) => sleep(Duration::from_millis(100)).await,
                Err(e) => return Err(status_to_err(e))
            }
        }
        error!("retry count exceeded");
        Err(InternalError)
    }

    pub async fn get_trader_margin(&mut self, trader_id: String) -> Result<trader_proto::GetTraderMarginResponse, LibError> {
        debug!(trader_id=trader_id.as_str(),"[GRPC] get_trader_margin");
        let request = trader_proto::GetTraderMarginRequest {
            trader_id,
        };
        for _ in 0..RETRY_COUNT {
            match self.client.get_trader_margin(Request::new(request.clone())).await {
                Ok(result) => return Ok(result.into_inner()),
                Err(e) if need_retry(e.code()) => sleep(Duration::from_millis(100)).await,
                Err(e) => return Err(status_to_err(e))
            }
        }
        error!("retry count exceeded");
        Err(InternalError)
    }
}


pub struct TraderServiceManager {
    addr: String,
}

impl TraderServiceManager {
    pub fn new(addr: String) -> TraderServiceManager {
        TraderServiceManager { addr }
    }
}



impl deadpool::managed::Manager for TraderServiceManager {
    type Type = TraderService;
    type Error = LibError;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let trader_service = TraderService::new(self.addr.clone()).await;
        Ok(trader_service)
    }

    async fn recycle(&self, _obj: &mut Self::Type, _metrics: &Metrics) -> RecycleResult<Self::Error> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct TraderServicePool {
    pool: Pool<TraderServiceManager>,
}

impl TraderServicePool {
    pub fn new(addr: String) -> Self {
        let manager = TraderServiceManager::new(addr);
        let pool = Pool::builder(manager)
            .max_size(500) // максимальное количество соединений
            .build()
            .unwrap();

        TraderServicePool { pool }
    }
    pub async fn get(&self) -> Object<TraderServiceManager> {
        self.pool.get().await.unwrap()
    }
}
