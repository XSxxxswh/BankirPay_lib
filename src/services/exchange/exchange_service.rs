use std::str::FromStr;
use std::time::Duration;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use tokio::time::sleep;
use tonic::Request;
use tonic::transport::Endpoint;
use tracing::{error, warn};
use crate::{exchange_proto};
use crate::errors::LibError;
use crate::errors::LibError::InternalError;
use crate::services::{need_retry, status_to_err};

#[derive(Clone)]
pub struct ExchangeService {
    client: exchange_proto::exchange_service_client::ExchangeServiceClient<tonic::transport::Channel>,
}

impl ExchangeService {
    pub fn new(addr : String) -> Self {
        let channel = Endpoint::from_str(addr.as_str())
            .unwrap()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .tcp_keepalive(Some(Duration::from_secs(10)))
            .connect_lazy();

        let client = exchange_proto::exchange_service_client::ExchangeServiceClient::new(channel);
        Self { client }
    }

    pub async fn get_exchange_rate(&mut self) -> Result<Decimal, LibError> {
        for i in 0..5 {
            if i >= 1 {
                warn!("[GRPC] exchange retry send get exchange rate action attempt {}", i+1);
            }
            match self.client.get_exchange_rate(()).await {
                Ok(result) => return Ok(Decimal::from_f64(result.get_ref().rate).ok_or(InternalError)?),
                Err(e) if need_retry(e.code()) => sleep(Duration::from_millis(100)).await,
                Err(e) => return Err(status_to_err(e))
            }
        }
        error!("[GRPC] exchange retry count exceeded");
        Err(InternalError)
    }
}