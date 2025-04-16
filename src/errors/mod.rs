use axum::body::Body;
use axum::response::{IntoResponse, Response};
#[derive(Debug, PartialOrd, PartialEq, Eq)]
pub enum LibError {
    TraderNotFound,
    Forbidden,
    Unauthorized,
    InternalError,
    MerchantNotFound,
    NotFound,
    NoAvailableRequisites,
    InsufficientFunds,
    InvalidAmount,
    Conflict
}

impl IntoResponse for LibError {
    fn into_response(self) -> Response {
        match self {
            LibError::Unauthorized => {
                Response::builder()
                    .status(401)
                    .header("Content-Type", "application/json")
                    .body(Body::from("{\"error\":401, \"message\":\"Unauthorized\"}"))
                    .unwrap()
            }
            LibError::InternalError => {
                Response::builder()
                    .status(500)
                    .header("Content-Type", "application/json")
                    .body(Body::from("{\"error\":500, \"message\":\"Internal Error\"}"))
                    .unwrap()
            }
            LibError::NotFound => {
                Response::builder()
                    .status(404)
                    .header("Content-Type", "application/json")
                    .body(Body::from("{\"error\":404, \"message\":\"Not Found\"}"))
                    .unwrap()
            }

            LibError::TraderNotFound => {
                Response::builder()
                    .status(404)
                    .header("Content-Type", "application/json")
                    .body(Body::from("{\"error\":404, \"message\":\"Trader not found\"}"))
                    .unwrap()
            }
            LibError::Forbidden => {
                Response::builder()
                    .status(403)
                    .header("Content-Type", "application/json")
                    .body(Body::from("{\"error\":403, \"message\":\"Forbidden\"}"))
                    .unwrap()
            }

            LibError::MerchantNotFound => {
                Response::builder()
                .status(404)
                .header("Content-Type", "application/json")
                .body(Body::from("{\"error\":404, \"message\":\"Merchant not found\"}"))
                .unwrap()
            }
            LibError::NoAvailableRequisites => {
                Response::builder()
                .status(500)
                .header("Content-Type", "application/json")
                .body(Body::from("{\"error\":500, \"message\":\"No Available Requisites\"}"))
                .unwrap()
            }
            LibError::InsufficientFunds => {
                Response::builder()
                .status(400)
                .header("Content-Type", "application/json")
                .body(Body::from("{\"error\":400,\"message\":\"Insufficient Funds\"}"))
                .unwrap()
            }
            LibError::InvalidAmount => {
                Response::builder()
                .status(400)
                .header("Content-Type", "application/json")
                .body(Body::from("{\"error\":400, \"message\":\"Invalid Amount\"}"))
                .unwrap()
            }
            LibError::Conflict => {
                Response::builder()
                .status(409)
                .header("Content-Type", "application/json")
                .body(Body::from("{\"error\":409, \"message\":\"Conflict\"}"))
                .unwrap()
            }
        }
    }

}