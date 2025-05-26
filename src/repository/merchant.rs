
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use tokio_postgres::types::Type;
use tracing::error;
use crate::errors::LibError;
use crate::errors::LibError::{InternalError, MerchantNotFound};
use crate::map_err_with_log;

const TTL_HOUR: usize = 60 * 60;
pub async fn check_merchant_is_blocked_from_redis(conn: &mut redis::aio::MultiplexedConnection, trader_id : &str) -> Result<Option<bool>, LibError> {
    let key = format!("merchant:{}:is_blocked", trader_id);
    match conn.get::<_, Option<String>>(key).await.map_err(|e| {
        error!("Error getting trader: {}", e);
        InternalError
    })? {
        Some(value) => Ok(Some(value == "1")),
        None => Ok(None)
    }
}

pub async fn check_merchant_is_blocked_from_db(client : &tokio_postgres::Client, merchant_id : &str) -> Result<bool, LibError> {
    let row  = client.query_typed(
        "SELECT is_blocked FROM merchants WHERE id=$1",
        &[(&merchant_id, Type::VARCHAR)]
    ).await.map_err(|e| {
        error!(merchant_id=merchant_id,err=e.to_string(), "Error check merchant is blocked");
        InternalError
    })?;
    Ok(row.first().ok_or(MerchantNotFound)?.get(0))
}

pub async fn set_trader_is_blocked_to_redis(
    conn: &mut MultiplexedConnection,
    trader_id: String,
    is_blocked: bool,
) -> Result<(), LibError> {
    let key = format!("merchant:{}:is_blocked", trader_id);

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
            error!("Error setting merchant is_blocked to redis: {}", e);
            InternalError
        })?;

    Ok(())
}

pub async fn get_public_key_from_db(client: &tokio_postgres::Client, merchant_id: &str)
                                    -> Result<String, LibError>
{
    let rows = map_err_with_log!(client.query_typed("SELECT public_key FROM merchants WHERE id=$1",
        &[(&merchant_id, Type::VARCHAR)]).await,
    "Error getting merchant public key from DB",InternalError, merchant_id)?;
    let row = rows.first().ok_or(LibError::MerchantNotFound)?;
    row.get::<_, Option<String>>(0).ok_or(LibError::NotFound)
}

pub async fn set_public_key_in_db(client: &tokio_postgres::Client, merchant_id: &str, public_key: &str)
                                  -> Result<(), LibError>
{
    let rows = map_err_with_log!(client.query_typed("UPDATE merchants SET public_key=$1 WHERE id=$2 RETURNING id",
        &[(&merchant_id, Type::VARCHAR), (&public_key, Type::TEXT)]).await,
        "Error setting merchant public key from DB",InternalError, merchant_id, public_key)?;
    if rows.is_empty() {
        return Err(LibError::MerchantNotFound);
    }
    Ok(())
}

pub async fn get_public_key_from_redis(conn: &mut MultiplexedConnection, merchant_id: &str)
                                       -> Result<Option<String>, LibError>
{
    let key = format!("merchant:{}:public_key", merchant_id);
    let public_key: Option<String> = map_err_with_log!(conn.get(key).await,
        "Error getting merchant public key from DB",
        InternalError, merchant_id)?;
    Ok(public_key)
}

pub async fn set_public_key_in_redis(conn: &mut MultiplexedConnection, merchant_id: &str, public_key: &str)
                                     -> Result<(), LibError>
{
    let key = format!("merchant:{}:public_key", merchant_id);
    let _: () = map_err_with_log!(conn.set(key, public_key).await,
        "Error setting merchant public key in Redis",
        InternalError, merchant_id)?;
    Ok(())
}