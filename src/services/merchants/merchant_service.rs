use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use tonic::Request;
use tonic::transport::Endpoint;
use tracing::{debug, error, warn};
use uuid::Uuid;
use crate::errors::LibError;
use crate::errors::LibError::InternalError;
use crate::merchant_proto;
use crate::services::{need_retry, status_to_err};

pub(crate) const RETRY_COUNT: usize = 5;

#[derive(Clone)]
pub struct MerchantService {
    client: merchant_proto::merchant_service_client::MerchantServiceClient<tonic::transport::Channel>,
}
impl MerchantService {
    pub fn new(addr : String) -> Self {
        let channel = Endpoint::from_str(addr.as_str())
            .unwrap()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .tcp_keepalive(Some(Duration::from_secs(10)))
            .connect_lazy();

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
        for i in 0..RETRY_COUNT {
            if i >= 1 {
                warn!(merchant_id=merchant_id, action_type=action_type.as_str_name(), "[GRPC] retry send balance action attempt {}", i+1);
            }
            match self.client.change_balance(Request::new(request.clone())).await {
                Ok(result) => return Ok(result.into_inner()),
                Err(e) if need_retry(e.code()) => sleep(Duration::from_millis(100)).await,
                Err(e) => return Err(status_to_err(e))
            }
        }
        error!(merchant_id=merchant_id,amount=amount, action_type=action_type.as_str_name(),"[GRPC] retry count exceeded");
        Err(InternalError)
    }

    pub async fn get_payment_method(&mut self, merchant_id: String, payment_method_id: String ) -> Result<merchant_proto::PaymentMethod, LibError> {
        debug!(merchant_id=merchant_id, payment_method_id=%payment_method_id, "[GRPC] send get_payment_method");
        let request = merchant_proto::GetPaymentMethodRequest{
            merchant_id: merchant_id.clone(),
            payment_method_id: payment_method_id.clone(),
        };
        for _ in 0..RETRY_COUNT {
            match self.client.get_payment_method(Request::new(request.clone())).await {
                Ok(result) => return Ok(result.into_inner()),
                Err(e) if need_retry(e.code()) => sleep(Duration::from_millis(100)).await,
                Err(e) => return Err(status_to_err(e))
            }
        }
        error!(merchant_id=merchant_id,payment_method_id=payment_method_id,"[GRPC] retry count exceeded method = get_payment_method");
        Err(InternalError)
    }

    pub async fn get_merchant_pms(&mut self, merchant_id: String) -> Result<merchant_proto::PaymentMethodList, LibError> {
        debug!(merchant_id=merchant_id,"[GRPC] send get_merchant_pms");
        let request = merchant_proto::GetPmListReq{
            merchant_id: merchant_id.clone(),
        };
        for i in 0..RETRY_COUNT {
            if i >= 1 {
                warn!(merchant_id=merchant_id, "[GRPC] retry send pm list action attempt {}", i+1);
            }
            match self.client.get_pm_list(Request::new(request.clone())).await {
                Ok(result) => return Ok(result.into_inner()),
                Err(e) if need_retry(e.code()) => sleep(Duration::from_millis(100)).await,
                Err(e) => return Err(status_to_err(e))
            }
        }
        error!(merchant_id=merchant_id,"[GRPC] retry count exceeded method = get_merchant_pms");
        Err(InternalError)
    }
}