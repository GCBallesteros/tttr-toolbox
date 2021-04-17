use thiserror::Error as ThisError;
use std::io;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("File {0} does not exist.")]
    FileNotAvailable(String),
    #[error("IO error.")]
    IOError(#[from] io::Error),
    //#[error("Failed unicode data conversion.")]
    //InvalidUnicode(#[from] IOError),
    #[error("A different enum variant was expexted.")]
    WrongEnumVariant,
    #[error("{0}")]
    InvalidHeader(String),
}

// todo give more context to WrongEnumVariant
