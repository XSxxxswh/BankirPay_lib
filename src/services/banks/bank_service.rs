use std::str::FromStr;
use std::time::Duration;
use tonic::{Request};
use tonic::transport::Endpoint;
use crate::{bank_proto, retry_grpc};
use tracing::{error, warn};
use crate::bank_proto::{BankShort};
use crate::errors::LibError;
use crate::services::{connect_to_grpc_server, need_retry, status_to_err};

const RETRY_COUNT: usize = 3;
#[derive(Debug, Clone)]
pub struct BankService {
    client: bank_proto::bank_service_client::BankServiceClient<tonic::transport::Channel>,
}


impl  BankService {
    pub fn new(addr: String) -> Self {
        let channel = connect_to_grpc_server(&addr);
        let client = bank_proto::bank_service_client::BankServiceClient::new(channel);
        Self { client }
    }
    pub async fn get_bank_info(&mut self, bank_id: String) -> Result<BankShort, LibError> {
       let request = bank_proto::GetBankInfoRequest { bank_id };
        match retry_grpc!(self.client.get_bank_info(Request::new(request.clone())), 3) {
            Ok(response) => Ok(response.into_inner()),
            Err(status) => Err(status_to_err(status)),
        }
    }
}