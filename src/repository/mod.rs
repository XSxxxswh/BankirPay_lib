pub mod trader;
pub(crate) mod merchant;
#[macro_export]
macro_rules! retry_sql {
    ($sql_func:expr, $max_retries:expr) => {{
        let mut result;
        let mut attempt = 0;
        loop {
            attempt += 1;
            result = $sql_func.await;
            match result {
                Err(ref e) if attempt < $max_retries && is_connection_err(e) => {
                    warn!(err=e.to_string(), "Error do request. Retrying...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                },
               _ => {
                    break;
                }
            }
        }
        result
    }};
}
pub fn is_connection_err<T>(e: &T) -> bool
where T: ToString
{
    e.is_closed()
        || e.to_string().contains("connection")
        || e.to_string().contains("broken")
        || e.to_string().contains("time")
        || e.to_string().contains("timed")
        || e.to_string().contains("conn")
        || e.to_string().contains("io")
}