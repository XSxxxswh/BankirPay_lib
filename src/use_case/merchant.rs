use std::sync::Arc;
use tracing::error;
use crate::errors::LibError;
use crate::{models, repository};
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