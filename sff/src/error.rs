use std::io;
use thiserror::Error;

use crate::{v1, v2};

#[derive(Debug, Error)]
pub enum LoadingError {
    #[error("Missing signature at start of file")]
    NoSignature,
    #[error("Error loading sprite data")]
    IoError(#[from] io::Error),
    #[error("Unknown sprite file version bytes {0:?}")]
    UnknownVersion([u8; 4]),
}

#[derive(Debug, Error)]
pub enum RenderingError<T> {
    #[error(transparent)]
    SffV1Error(#[from] v1::RenderingError<T>),
    #[error(transparent)]
    SffV2Error(#[from] v2::RenderingError<T>),
}