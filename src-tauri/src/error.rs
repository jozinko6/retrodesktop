use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Neplatný vstup: {0}")]
    InvalidInput(String),
    #[error("Súbor alebo priečinok neexistuje: {0}")]
    NotFound(String),
    #[error("Operácia nie je povolená: {0}")]
    Forbidden(String),
    #[error("Databázová chyba: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("I/O chyba: {0}")]
    Io(#[from] std::io::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
