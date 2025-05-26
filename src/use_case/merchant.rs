use std::sync::Arc;
use base64::Engine;
use base64::engine::general_purpose;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs1::der::zeroize::Zeroizing;
use rsa::{Pkcs1v15Sign, RsaPublicKey};
use rsa::sha2::{Digest, Sha256};
use rsa::traits::SignatureScheme;
use tracing::{error, warn};
use crate::errors::LibError;
use crate::{map_err_with_log, models, repository};
use crate::errors::LibError::InternalError;

pub(crate) async fn check_merchant_is_blocked(
    state: Arc<models::AuthState>,
    trader_id: &str
) -> Result<bool, LibError> {
    let conn_result = state.rdb.get().await;
    let mut conn = match conn_result {
        Ok(conn) => conn,
        Err(e) => {
            let pg = state.pool.get().await.map_err(|e|{
                error!(err=e.to_string(), "Error get PG connection");
                InternalError
            })?;
            error!("Error getting Redis connection: {}", e);
            return repository::merchant::check_merchant_is_blocked_from_db(&pg, trader_id).await;
        }
    };


    match repository::merchant::check_merchant_is_blocked_from_redis(&mut conn.clone(), trader_id).await {
        Ok(Some(val)) => Ok(val),
        Ok(None) | Err(_) => {
            let pg = state.pool.get().await.map_err(|e|{
                error!(err=e.to_string(), "Error get PG connection");
                InternalError
            })?;
            let blocked = repository::merchant::check_merchant_is_blocked_from_db(&pg, trader_id).await?;
            let trader_id = trader_id.to_string();
            tokio::spawn(async move {
                let _ = repository::merchant::set_trader_is_blocked_to_redis(&mut conn, trader_id, blocked).await;

            });
            Ok(blocked)
        }
    }
}

pub async fn verify_signature(state: Arc<models::AuthState>, merchant_id: &str, signature: &str, raw_line: &str)
                              -> Result<bool, LibError>
{
    let public_key = RsaPublicKey::from_pkcs1_pem(get_public_key(state.clone(), merchant_id).await?.as_str()).map_err(|_| InternalError)?;
    let raw_line_bytes = raw_line.as_bytes();
    let signature = general_purpose::STANDARD
        .decode(signature)
        .map_err(|_| LibError::Unauthorized)?;
    let padding = Pkcs1v15Sign::new::<Sha256>();
    match padding.verify(&public_key, &*Sha256::digest(raw_line_bytes), &signature) {
        Ok(()) => Ok(true),
        Err(e) => {
            error!(err=e.to_string(), "Error verifying signature");
            Ok(false)
        }
    }
}

pub async fn get_public_key(state: Arc<models::AuthState>, merchant_id: &str)
                            -> Result<Zeroizing<String>, LibError>
{
    let mut conn = match state.rdb.get().await {
        Ok(c) => c,
        Err(e) => {
            error!(err=e.to_string(), "Error get redis connection");
            let pg = map_err_with_log!(state.pool.get().await, "Error get DB connection", InternalError, merchant_id)?;
            let key = repository::merchant::get_public_key_from_db(&pg, merchant_id).await?;
            return Ok(Zeroizing::new(key));
        }
    };
    match repository::merchant::get_public_key_from_redis(&mut conn, merchant_id).await {
        Ok(Some(public_key)) => Ok(Zeroizing::new(public_key)),
        Ok(None) => {
            warn!(merchant_id=merchant_id,"public not found in redis");
            let pg = map_err_with_log!(state.pool.get().await,"Error get DB connection",
                InternalError, merchant_id)?;
            let key = repository::merchant::get_public_key_from_db(&pg, merchant_id).await?;
            let _ = repository::merchant::set_public_key_in_redis(&mut conn, merchant_id, &key).await;
            Ok(Zeroizing::new(key))
        },
        Err(_) => {
            let pg = map_err_with_log!(state.pool.get().await,"Error get DB connection",
                InternalError, merchant_id)?;
            let key = repository::merchant::get_public_key_from_db(&pg, merchant_id).await?;
            Ok(Zeroizing::new(key))
        }
    }
}