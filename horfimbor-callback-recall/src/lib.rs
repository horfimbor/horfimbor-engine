use crate::database::{CallBack, Pool};
use crate::error::CallbackError;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub mod database;
pub mod error;
pub(crate) mod runner;

/// A registered async handler. Takes owned payload bytes and returns a future.
type HandlerFn =
    Arc<dyn Fn(Vec<u8>) -> BoxFuture<'static, Result<(), String>> + Send + Sync + 'static>;

pub(crate) struct Inner<P: Pool> {
    pub pool: P,
    pub handlers: HashMap<String, HandlerFn>,
}

pub struct SchedulerBuilder<P: Pool> {
    inner: Inner<P>,
    duration: Duration,
}

impl<P> SchedulerBuilder<P>
where
    P: Pool,
{
    /// # Errors
    ///
    /// This function will fail if migrate failed, to ensure the server doesn't boot
    /// when there is no change that it will run correctly.
    pub async fn new(pool: P, duration: Duration) -> Result<Self, CallbackError> {
        pool.migrate().await?;

        Ok(Self {
            inner: Inner {
                pool,
                handlers: HashMap::new(),
            },
            duration,
        })
    }

    pub fn register<F, Fut>(&mut self, name: &str, handler: F)
    where
        F: Fn(Vec<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        let boxed: HandlerFn = Arc::new(move |payload| Box::pin(handler(payload)));
        self.inner.handlers.insert(name.to_string(), boxed);
    }

    pub fn start(self) -> (SchedulerEmitter<P>, SchedulerListener) {
        let pool = self.inner.pool.clone();
        let task = tokio::spawn(runner::run(self.inner, self.duration));
        (SchedulerEmitter { pool }, SchedulerListener { task })
    }
}

/// A handle to the running scheduler background task.
pub struct SchedulerEmitter<P> {
    pool: P,
}

/// A handle to the running scheduler background task.
pub struct SchedulerListener {
    task: tokio::task::JoinHandle<()>,
}

impl<P> SchedulerEmitter<P>
where
    P: Pool,
{
    /// # Errors
    ///
    /// This function will fail if the callback cannot be registered into the DB.
    /// Note: it doesn't mean it will work.
    pub async fn schedule(&self, call_back: CallBack) -> Result<(), CallbackError> {
        self.pool.insert_callback(call_back).await
    }
}

impl SchedulerListener {
    /// Wait until the poller task finishes (only happens on abort or panic).
    pub async fn join(self) {
        let _ = self.task.await;
    }

    /// Stop the poller task.
    pub fn shutdown(&self) {
        self.task.abort();
    }
}
