use crate::services::need_retry;
use tracing::warn;
use tonic::transport::Channel;
use crate::errors::LibError;
use crate::{models, retry_grpc};
use std::time::Duration;
use tracing::error;
use crate::models::payments::payment_proto;
use crate::models::payments::payment_proto::{ByExternalId, ById};
use crate::services::{connect_to_grpc_server, status_to_err};

pub struct PaymentService {
    client: payment_proto::payment_service_client::PaymentServiceClient<Channel>
}


impl PaymentService {
    pub fn new(addr: String) -> Self {
        let channel = connect_to_grpc_server(&addr);
        let client =
            payment_proto::payment_service_client::PaymentServiceClient::new(channel);
        PaymentService { client }
    }
    pub async fn get_payment_by_id(&mut self, payment_id: String, merchant_id: String)
    -> Result<payment_proto::PaymentProto, LibError>
    {
        let request = payment_proto::GetPaymentByIdRequest{
            search_type: Some(payment_proto::get_payment_by_id_request::SearchType::Local(ById{
                id: payment_id,
                merchant_id,
            }))
        };
        match retry_grpc!(self.client.get_payment_by_id(request.clone()), 3) {
            Ok(response) => Ok(response.into_inner()),
            Err(e) => Err(status_to_err(e))
        }
    }
    pub async fn get_payment_by_external_id(&mut self, external_id: String, merchant_id: String)
    -> Result<payment_proto::PaymentProto, LibError>
    {
        let req = payment_proto::GetPaymentByIdRequest{
            search_type: Some(payment_proto::get_payment_by_id_request::SearchType::External(ByExternalId{
                external_id,
                merchant_id,
            }))
        };
        match retry_grpc!(self.client.get_payment_by_id(req.clone()), 3) {
            Ok(resp) => Ok(resp.into_inner()),
            Err(e) => Err(status_to_err(e))
        }
    }
    pub async fn close_payment(&mut self, payment_id: String, amount: Option<f64>)
    -> Result<(), LibError>
    {
        let request = payment_proto::ClosePaymentRequest{ payment_id, amount };
        match retry_grpc!(self.client.close_payment(request.clone()), 3) {
            Ok(response) => Ok(response.into_inner()),
            Err(e) => Err(status_to_err(e))
        }
    }
}