use crate::database::Pool;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub mod database;
pub mod error;
mod runner;

/// A registered async handler. Takes owned payload bytes and returns a future.
type HandlerFn = Arc<dyn Fn(Vec<u8>) -> BoxFuture<'static, ()> + Send + Sync + 'static>;

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
    pub async fn new(pool: P, duration: Duration) -> Self {
        Self {
            inner: Inner {
                pool,
                handlers: HashMap::new(),
            },
            duration,
        }
    }

    pub fn register<F, Fut>(&mut self, name: &str, handler: F)
    where
        F: Fn(Vec<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let boxed: HandlerFn = Arc::new(move |payload| Box::pin(handler(payload)));
        self.inner.handlers.insert(name.to_string(), boxed);
    }

    pub fn start(self) -> SchedulerHandle {
        let task = tokio::spawn(runner::run(self.inner, self.duration));
        SchedulerHandle { task }
    }
}

/// A handle to the running scheduler background task.
pub struct SchedulerHandle {
    task: tokio::task::JoinHandle<()>,
}

impl SchedulerHandle {
    /// Wait until the poller task finishes (only happens on abort or panic).
    pub async fn join(self) {
        let _ = self.task.await;
    }

    /// Stop the poller task.
    pub fn shutdown(&self) {
        self.task.abort();
    }
}
