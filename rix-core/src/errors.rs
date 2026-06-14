use std::fmt;

#[derive(Debug)]
pub enum RixError {
    IOError(std::io::Error),
    ParseError(String),
    /// Host is missing critical Nix tooling dependencies
    MissingSystemDependency(String),
    /// The generated file contains syntax that failed validation
    InvalidNixSyntax(String),
}

// 1. Implement Display to format the errors cleanly for the end-user
impl fmt::Display for RixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RixError::IOError(err) => write!(f, "I/O Error: {}", err),
            RixError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            RixError::MissingSystemDependency(msg) => write!(f, "Missing System Dependency: {}", msg),
            RixError::InvalidNixSyntax(msg) => write!(f, "Invalid Nix Syntax: {}", msg),
        }
    }
}

// 2. Implement standard Error trait so it plays nice with the Rust ecosystem
impl std::error::Error for RixError {}

impl From<std::io::Error> for RixError {
    fn from(error: std::io::Error) -> Self {
        RixError::IOError(error)
    }
}
