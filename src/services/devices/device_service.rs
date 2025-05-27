use tracing::warn;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use tonic::Request;
use tonic::transport::Endpoint;
use tracing::error;
use crate::{device_proto, retry_grpc};
use crate::errors::LibError;
use crate::errors::LibError::InternalError;
use crate::services::{connect_to_grpc_server, need_retry, status_to_err};
use crate::services::merchants::merchant_service::RETRY_COUNT;

#[derive(Clone, Debug)]
pub struct DeviceService {
    client: device_proto::device_service_client::DeviceServiceClient<tonic::transport::Channel>,
}

impl DeviceService {
    pub fn new(addr : String) -> Self {
        let channel = connect_to_grpc_server(addr.as_str());
        let client = device_proto::device_service_client::DeviceServiceClient::new(channel);
        Self { client }
    }
    pub async fn get_device_status(&mut self, device_id: String) -> Result<device_proto::Status, LibError>  {
        let request = device_proto::GetDeviceStatusReq{
            device_id,
        };
        match retry_grpc!(self.client.get_device_status(Request::new(request.clone())), 3) {
            Ok(result) => Ok(result.into_inner()),
            Err(e) => Err(status_to_err(e))
        }
    }
}
