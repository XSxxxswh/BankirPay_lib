use std::str::FromStr;
use std::time::Duration;
use tonic::{Request};
use tonic::transport::Endpoint;
use crate::{bank_proto};
use crate::bank_proto::{BankShort};
use crate::errors::LibError;
use crate::services::{need_retry, status_to_err};

const RETRY_COUNT: usize = 3;
#[derive(Debug, Clone)]
pub struct BankService {
    client: bank_proto::bank_service_client::BankServiceClient<tonic::transport::Channel>,
}


impl  BankService {
    pub fn new(addr: String) -> Self {
        let channel = Endpoint::from_str(addr.as_str())
            .unwrap()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .tcp_keepalive(Some(Duration::from_secs(10)))
            .connect_lazy();

        let client = bank_proto::bank_service_client::BankServiceClient::new(channel);
        Self { client }
    }
    pub async fn get_bank_info(&mut self, bank_id: String) -> Result<BankShort, LibError> {
       let request = bank_proto::GetBankInfoRequest { bank_id };
        for  _ in 0..RETRY_COUNT {
            match self.client.get_bank_info(Request::new(request.clone())).await {
                Ok(response) => return Ok(response.into_inner()),
                Err(status) if need_retry(status.code()) => {
                    continue;
                }
                Err(status) => return Err(status_to_err(status)),
            }
        }
        Err(LibError::InternalError)
    }
}