use thiserror::Error;

#[derive(Error, Debug)]
pub enum GError {
    #[error("{0}")]
    Unknown(String),
    #[error("{0}")]
    Timeout(String),
}

pub type GResult<T> = core::result::Result<T, GError>;
