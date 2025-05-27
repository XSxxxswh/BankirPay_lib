use std::str::FromStr;
use std::time::Duration;
use tonic::{Code, Status};
use tonic::transport::Endpoint;
use tracing::error;
use crate::errors::LibError;
use crate::errors::LibError::{Conflict, InsufficientFunds, InternalError, InvalidAmount, NoAvailableRequisites, NotFound};


pub mod merchants;
pub mod requisites;
pub mod traders;
pub mod devices;
pub mod banks;
pub mod exchange;
pub mod payments;

fn status_to_err(status: Status) -> LibError {
    error!("GRPC client_err {}", status.to_string());
    match status.code() {
        Code::Internal =>  InternalError,
        Code::NotFound => match status.message().to_lowercase().as_str() {
            "no available requisites" => NoAvailableRequisites,
            _ => NotFound,
        },
        Code::InvalidArgument => match status.message().to_lowercase().as_str() {
            "insufficient funds" => InsufficientFunds,
            "invalid amount" => InvalidAmount,
            _ => {
                error!("GRPC trader something went wrong status: {}", status.to_string());
                InternalError
            },
        },
        Code::Cancelled => Conflict,
        _ =>  {
            error!("GRPC trader something went wrong status: {}", status.to_string());
            InternalError
        },
    }
}

fn connect_to_grpc_server(addr: &str) -> tonic::transport::Channel
{
    Endpoint::from_str(addr)
        .unwrap()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .tcp_keepalive(Some(Duration::from_secs(10)))
        .connect_lazy()
}

fn need_retry(code: Code) -> bool {
    match code {
        Code::DeadlineExceeded => true,
        Code::ResourceExhausted => true,
        Code::Aborted => true,
        Code::Unavailable => true,
        _ => false,
    }
}
#[macro_export]
macro_rules! retry_grpc {
    ($request:expr, $max_retries:expr) => {{
        let mut response;
        let mut attempt = 0;
        loop {
            attempt += 1;
            response = $request.await;
            match &response {
                Ok(response) => {
                    break;
                },
                &Err(ref e) if need_retry(e.code()) && attempt < $max_retries => {
                    warn!(err=e.message(),"[GRPC] error get response. Retrying...");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                },
                &Err(ref e) => {
                    error!(err=e.message(),"[GRPC] error get response");
                    break;
                }
            }
        }
        response
    }};
}