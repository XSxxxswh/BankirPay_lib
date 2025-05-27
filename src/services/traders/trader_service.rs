use std::time::Duration;
use deadpool::managed::{Metrics, Object, Pool, RecycleResult};
use prost::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use tonic::{Request};
use tonic::transport::{Channel, Endpoint};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use crate::errors::LibError;
use crate::errors::LibError::InternalError;

use crate::services::{connect_to_grpc_server, need_retry, status_to_err};
use crate::{retry_grpc, trader_proto};

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
    pub fn new(addr : String) -> Self {
        let channel = connect_to_grpc_server(addr.as_str());
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

        retry_grpc!(self.client.change_balance(Request::new(request.clone())), 3)
            .map_err(|e| status_to_err(e))?;
        error!("retry count exceeded");
        Err(InternalError)
    }

    pub async fn get_trader_margin(&mut self, trader_id: String) -> Result<trader_proto::GetTraderMarginResponse, LibError> {
        debug!(trader_id=trader_id.as_str(),"[GRPC] get_trader_margin");
        let request = trader_proto::GetTraderMarginRequest {
            trader_id,
        };

        match retry_grpc!(self.client.get_trader_margin(Request::new(request.clone())), 3) {
            Ok(result) => Ok(result.into_inner()),
            Err(e) => Err(status_to_err(e))
        }


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
        let trader_service = TraderService::new(self.addr.clone());
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
