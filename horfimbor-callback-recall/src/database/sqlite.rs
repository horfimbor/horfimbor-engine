use crate::database::{CallBack, CallBackRow, Pool};
use crate::error::CallbackError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

const MIGRATION: &str = include_str!("./sqlite_migration.sql");

/// # Errors
///
/// This function fail when the db file cannot be opened
///
pub async fn open(database_url: &str) -> Result<SqlitePool, CallbackError> {
    let opts = SqliteConnectOptions::from_str(database_url)
        .map_err(|e| CallbackError::Database(e.to_string()))?
        .create_if_missing(true);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await
        .map_err(|e| CallbackError::Database(e.to_string()))
}

#[async_trait]
impl Pool for SqlitePool {
    async fn migrate(&self) -> Result<(), CallbackError> {
        for stmt in MIGRATION
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            sqlx::query(stmt)
                .execute(self)
                .await
                .map_err(|e| CallbackError::Database(e.to_string()))?;
        }

        Ok(())
    }

    async fn insert_callback(&self, cb: CallBack) -> Result<(), CallbackError> {
        sqlx::query("INSERT INTO callbacks (identifier, payload, due_date) VALUES (?, ?, ?)")
            .bind(cb.identifier)
            .bind(cb.payload)
            .bind(cb.due_date)
            .execute(self)
            .await
            .map_err(|e| CallbackError::Database(e.to_string()))?;

        Ok(())
    }

    async fn fetch_due_soon(
        &self,
        due_before: DateTime<Utc>,
    ) -> Result<Vec<CallBackRow>, CallbackError> {
        let rows = sqlx::query(
            "SELECT id, identifier, payload, due_date, status, created_at, fired_at, failed_at, error_msg
         FROM callbacks
         WHERE status = 'pending' AND due_date <= ?
         ORDER BY due_date ASC",
        )
            .bind(due_before)
            .fetch_all(self)
            .await.map_err(|e| CallbackError::Database(e.to_string()))?;

        let result = rows
            .into_iter()
            .map(|r| CallBackRow {
                id: r.get("id"),
                identifier: r.get("identifier"),
                payload: r.get("payload"),
                due_date: r.get("due_date"),
            })
            .collect();

        Ok(result)
    }

    async fn mark_fired(&self, id: u32) -> Result<(), CallbackError> {
        sqlx::query(
            "UPDATE callbacks SET status = 'fired', fired_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(id)
        .execute(self)
        .await
        .map_err(|e| CallbackError::Database(e.to_string()))?;

        Ok(())
    }

    async fn mark_failed(&self, id: u32, error: &str) -> Result<(), CallbackError> {
        sqlx::query(
            "UPDATE callbacks SET status = 'failed', failed_at = CURRENT_TIMESTAMP, error_msg = ? WHERE id = ?",
        )
            .bind(error)
            .bind(id)
            .execute(self)
            .await.map_err(|e| CallbackError::Database(e.to_string()))?;
        Ok(())
    }
}
