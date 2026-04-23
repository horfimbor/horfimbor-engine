#[derive(Debug, thiserror::Error)]
pub enum CallbackError {
    #[error("database error: {0}")]
    Database(String),

    #[error("no handler registered for name: {0}")]
    UnknownHandler(String),
}
