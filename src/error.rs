use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Snow(#[from] snow::error::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
