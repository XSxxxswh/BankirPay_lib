use crate::services::{connect_to_grpc_server, LibError};
use std::str::FromStr;
use std::time::Duration;
use deadpool::managed::{Metrics, Object, Pool, RecycleResult};
use tokio::time::{sleep, Instant};
use tonic::Request;
use tonic::transport::Endpoint;
use tracing::{error, warn};
use crate::errors::LibError::InternalError;
use crate::{requisites_proto, retry_grpc};
use crate::services::{need_retry, status_to_err};


const RETRY_COUNT: usize = 3;

#[derive(Clone)]
pub struct RequisitesService {
    client: requisites_proto::requisite_service_client::RequisiteServiceClient<tonic::transport::Channel>,
}

impl RequisitesService {
    pub fn new(addr: String) -> Self {
        let channel = connect_to_grpc_server(addr.as_str());
        let client = requisites_proto::requisite_service_client::RequisiteServiceClient::new(channel);
        Self { client }
    }

    pub async fn get_requisites_for_payment(&mut self, method_type: Option<String>, amount: f64, currency: String, bank: Option<String>, cross_border: Option<bool>) -> Result<Vec<requisites_proto::Requisite>, LibError> {
        let request = requisites_proto::GetRequisitesForPaymentRequest{
            method_type,
            amount,
            currency,
            cross_border,
            bank,
        };
        let start = Instant::now();

        let res =
            retry_grpc!(self.client.get_requisites_for_payment(Request::new(request.clone())), 3);
        match res {
            Ok(result) => {
                if start.elapsed().as_millis() > 50 {
                    warn!("[GRPC] Requisite SLOW to get requisites! {:?}", start.elapsed());
                }
                Ok(result.into_inner().requisites)
            },
            Err(e) => Err(status_to_err(e))
        }
    }
}


pub struct RequisitesServiceManager {
    addr: String,
}

impl RequisitesServiceManager {
    pub fn new(addr: String) -> RequisitesServiceManager {
        RequisitesServiceManager{ addr }
    }
}



impl deadpool::managed::Manager for RequisitesServiceManager {
    type Type = RequisitesService;
    type Error = LibError;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let requisite_service = RequisitesService::new(self.addr.clone());
        Ok(requisite_service)
    }

    async fn recycle(&self, _obj: &mut Self::Type, _metrics: &Metrics) -> RecycleResult<Self::Error> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct RequisiteServicePool {
    pool: Pool<RequisitesServiceManager>,
}

impl RequisiteServicePool {
    pub fn new(addr: String) -> Self {
        let manager = RequisitesServiceManager::new(addr);
        let pool = Pool::builder(manager)
            .max_size(100) // максимальное количество соединений
            .build()
            .unwrap();

        RequisiteServicePool { pool }
    }
    pub async fn get(&self) -> Object<RequisitesServiceManager> {
        self.pool.get().await.unwrap()
    }
}