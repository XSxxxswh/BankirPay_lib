
use redis::AsyncCommands;
use tracing::error;
use crate::errors::LibError;
use crate::errors::LibError::{InternalError, TraderNotFound};

const TTL_HOUR: usize = 60 * 60;
pub async fn check_trader_is_blocked_from_redis(conn: &mut redis::aio::MultiplexedConnection, trader_id : &str) -> Result<Option<bool>, LibError> {
    let key = format!("trader:{}:is_blocked", trader_id);
    match conn.get::<_, Option<String>>(key).await.map_err(|e| {
        error!("Error getting trader: {}", e);
        InternalError
    })? {
        Some(value) => Ok(Some(value == "1")),
        None => Ok(None)
    }
}

pub async fn check_trader_is_blocked_from_db(client : &tokio_postgres::Client, trader_id : &str) -> Result<bool, LibError> {
    let row  = client.query_opt(
        "SELECT is_blocked FROM traders WHERE id=$1",
        &[&trader_id]
    ).await.map_err(|e| {
        error!(trader_id=trader_id,err=e.to_string(), "Error check trader is blocked");
        InternalError
    })?;
    Ok(row.ok_or(TraderNotFound)?.get(0))
}


pub async fn set_trader_is_blocked_to_redis(
    conn: &mut redis::aio::MultiplexedConnection,
    trader_id: String,
    is_blocked: bool,
) -> Result<(), LibError> {
    let key = format!("trader:{}:is_blocked", trader_id);

    redis::pipe()
        .atomic()
        .cmd("SET")
        .arg(&key)
        .arg(if is_blocked { "1" } else { "0" })
        .cmd("EXPIRE")
        .arg(&key)
        .arg(TTL_HOUR)
        .exec_async(conn)
        .await
        .map_err(|e| {
            error!("Error setting trader is_blocked to redis: {}", e);
            InternalError
        })?;

    Ok(())
}


