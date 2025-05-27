use std::fmt::Error;

pub mod trader;
pub(crate) mod merchant;
#[macro_export]
macro_rules! retry {
    ($sql_func:expr, $max_retries:expr) => {{
        let mut result;
        let mut attempt = 0;
        loop {
            attempt += 1;
            result = tokio::time::timeout(Duration::from_millis(300), $sql_func).await;
            match result {
                Ok(Ok(r)) => {
                    break Ok(r)
                },
                Ok(Err(ref e)) if attempt < $max_retries && is_connection_err(e) => {
                    warn!(err=e.to_string(), "Error do request. Retrying...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                },
                Ok(Err(e)) => break Err(e.to_string()),
                Err(e)if attempt < $max_retries => {
                    warn!(err=e.to_string(), "timeout do request. Retrying...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
                Err(e) => {
                    break Err(e.to_string());
                }
            }
        }
    }};
}
pub fn is_connection_err<T>(e: &T) -> bool
where T: ToString
{
    e.to_string().contains("connection")
        || e.to_string().contains("broken")
        || e.to_string().contains("time")
        || e.to_string().contains("timed")
        || e.to_string().contains("conn")
        || e.to_string().contains("io")
}