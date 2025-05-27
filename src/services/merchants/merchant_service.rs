use std::time::Duration;
use tonic::Request;
use tracing::{debug, error, warn};
use uuid::Uuid;
use crate::errors::LibError;
use crate::errors::LibError::InternalError;
use crate::{merchant_proto, retry_grpc};
use crate::services::{connect_to_grpc_server, need_retry, status_to_err};

pub(crate) const RETRY_COUNT: usize = 5;

pub const MERCHANT_CHANGE_BALANCE_TOPIC: &'static str = "merchant_change_balance";


#[derive(Clone)]
pub struct MerchantService {
    client: merchant_proto::merchant_service_client::MerchantServiceClient<tonic::transport::Channel>,
}
impl MerchantService {
    pub fn new(addr : String) -> Self {
        let channel = connect_to_grpc_server(addr.as_str());
        let client = merchant_proto::merchant_service_client::MerchantServiceClient::new(channel);
        Self { client }
    }

    pub async fn change_balance(&mut self, merchant_id: String, amount: f64, action_type: merchant_proto::BalanceActionType) -> Result<(), LibError> {
        debug!(mechant_id = %merchant_id, action_type = action_type.as_str_name(), "[GRPC] send merchant change_balance");
        let idempotent_key = Uuid::now_v7().to_string();
        let request = merchant_proto::ChangeBalanceRequest {
            merchant_id: merchant_id.clone(),
            amount,
            action_type: action_type as i32,
            idempotent_key,
        };
        match retry_grpc!(self.client.change_balance(Request::new(request.clone())), 3) {
            Ok(result) => Ok(result.into_inner()),
            Err(e) => Err(status_to_err(e))
        }
    }

    pub async fn get_payment_method(&mut self, merchant_id: String, payment_method_id: String ) -> Result<merchant_proto::PaymentMethod, LibError> {
        debug!(merchant_id=merchant_id, payment_method_id=%payment_method_id, "[GRPC] send get_payment_method");
        let request = merchant_proto::GetPaymentMethodRequest{
            merchant_id: merchant_id.clone(),
           payment_method_id: payment_method_id.clone(),
        };

        match retry_grpc!(self.client.get_payment_method(Request::new(request.clone())), 3) {
            Ok(result) => Ok(result.into_inner()),
            Err(e) => Err(status_to_err(e))
        }
    }

    pub async fn get_merchant_pms(&mut self, merchant_id: String) -> Result<merchant_proto::PaymentMethodList, LibError> {
        debug!(merchant_id=merchant_id,"[GRPC] send get_merchant_pms");
        let request = merchant_proto::GetPmListReq{
            merchant_id: merchant_id.clone(),
        };
        match retry_grpc!(self.client.get_pm_list(Request::new(request.clone())), 3) {
            Ok(result) => Ok(result.into_inner()),
            Err(e) => Err(status_to_err(e))
        }
    }

    pub async fn get_merchant_webhook_url(&mut self, merchant_id: String)
    -> Result<String, LibError> {
        debug!(merchant_id=merchant_id,"[GRPC] send get_merchant_webhook_url");
        let request = merchant_proto::GetWebhookUrlRequest{
            merchant_id: merchant_id.clone(),
        };
        match retry_grpc!(self.client.get_webhook_url(Request::new(request.clone())), 3) {
            Ok(result) => Ok(result.into_inner().webhook_url),
            Err(e) => Err(status_to_err(e))
        }
    }
}