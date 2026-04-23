use crate::error::CallbackError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[cfg(feature = "sqlx_sqlite")]
pub mod sqlite;

pub struct CallBack {
    identifier: String,
    payload: Vec<u8>,
    due_date: DateTime<Utc>,
}

pub struct CallBackRow {
    pub id: u32,
    pub identifier: String,
    pub payload: Vec<u8>,
    pub due_date: DateTime<Utc>,
}

impl CallBack {
    #[must_use]
    pub const fn new(identifier: String, payload: Vec<u8>, due_date: DateTime<Utc>) -> Self {
        Self {
            identifier,
            payload,
            due_date,
        }
    }
}

#[async_trait]
pub trait Pool: Clone + Send + Sync + 'static {
    async fn migrate(&self) -> Result<(), CallbackError>;

    async fn insert_callback(&self, cb: CallBack) -> Result<(), CallbackError>;

    async fn fetch_due_soon(
        &self,
        due_before: DateTime<Utc>,
    ) -> Result<Vec<CallBackRow>, CallbackError>;

    async fn mark_fired(&self, id: u32) -> Result<(), CallbackError>;

    async fn mark_failed(&self, id: u32, error: &str) -> Result<(), CallbackError>;
}
