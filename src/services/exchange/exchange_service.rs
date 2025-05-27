use std::str::FromStr;
use std::time::Duration;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use tokio::time::sleep;
use tonic::Request;
use tonic::transport::Endpoint;
use tracing::{error, warn};
use crate::{exchange_proto, retry_grpc};
use crate::errors::LibError;
use crate::errors::LibError::InternalError;
use crate::services::{connect_to_grpc_server, need_retry, status_to_err};

#[derive(Clone)]
pub struct ExchangeService {
    client: exchange_proto::exchange_service_client::ExchangeServiceClient<tonic::transport::Channel>,
}

impl ExchangeService {
    pub fn new(addr : String) -> Self {
        let channel = connect_to_grpc_server(addr.as_str());
        let client = exchange_proto::exchange_service_client::ExchangeServiceClient::new(channel);
        Self { client }
    }

    pub async fn get_exchange_rate(&mut self) -> Result<Decimal, LibError> {
        match retry_grpc!(self.client.get_exchange_rate(()), 3) {
            Ok(result) => Ok(Decimal::from_f64(result.get_ref().rate).ok_or(InternalError)?),
            Err(e) => Err(status_to_err(e))
        }
    }
}