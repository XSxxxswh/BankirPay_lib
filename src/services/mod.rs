use tonic::{Code, Status};
use tracing::error;
use crate::errors::LibError;
use crate::errors::LibError::{Conflict, InsufficientFunds, InternalError, InvalidAmount, NoAvailableRequisites, NotFound};


pub mod merchants;
pub mod requisites;
pub mod traders;
pub mod devices;
pub mod banks;

fn status_to_err(status: Status) -> LibError {
    error!("GRPC client_err {}", status.to_string());
    match status.code() {
        Code::Internal => return InternalError,
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


fn need_retry(code: Code) -> bool {
    match code {
        Code::DeadlineExceeded => true,
        Code::ResourceExhausted => true,
        Code::Aborted => true,
        Code::Unavailable => true,
        _ => false,
    }
}