use std::io;

/// Custom error types for Rix engine operations
#[derive(Debug)]
pub enum RixError {
    Io(io::Error),
    ParseError(String),
    PackageNotFound(String),
}

impl From<io::Error> for RixError {
    fn from(err: io::Error) -> Self {
        RixError::Io(err)
    }
}
