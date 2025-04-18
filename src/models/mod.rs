use serde::{Deserialize, Serialize};

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct AuthState {
    pub pool: deadpool_postgres::Pool,
    pub rdb: deadpool_redis::Pool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    #[serde(skip_serializing_if="Option::is_none")]
    pub impersonated_by: Option<String>,
}


type Job = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;
#[derive(Clone)]
pub struct WorkerPool {
    tx: tokio::sync::mpsc::Sender<Job>,
}

impl WorkerPool {
    pub fn new(worker_count: usize) -> Self
    {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Job>(100);
        let semaphore = Arc::new(Semaphore::new(worker_count));
        tokio::spawn(async move {
            while let Some(job) = rx.recv().await {
                let permit = semaphore.clone().acquire_owned().await.unwrap();

                tokio::spawn(async move {
                    job().await;
                    drop(permit);
                });
            }
        });
        Self { tx }
    }
    pub async fn execute<F, Fut>(&self, job: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {

        let job = Box::new(move || -> Pin<Box<dyn Future<Output = ()> + Send>> {
            Box::pin(job())
        });
        self.tx.send(job).await.expect("Failed to send job");
    }
}


#[macro_export]
macro_rules! map_err_with_log {
    ($res:expr, $msg:literal, $error:ident, $($name:ident),+) => {
        $res.map_err(|e| {
            error!(
                $(
                    $name = &$name,
                )+
                err = e.to_string(),
                $msg
            );
            $error
        })
    };
}