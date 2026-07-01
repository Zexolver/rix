use std::fmt;

#[derive(Debug)]
pub enum RixError {
    IOError(std::io::Error),
    ParseError(String),
    MissingSystemDependency(String),
    InvalidNixSyntax(String),
    // Add the new Git error variant
    GitError(git2::Error),
}

impl fmt::Display for RixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RixError::IOError(err) => write!(f, "I/O Error: {}", err),
            RixError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            RixError::MissingSystemDependency(msg) => {
                write!(f, "Missing System Dependency: {}", msg)
            }
            RixError::InvalidNixSyntax(msg) => write!(f, "Invalid Nix Syntax: {}", msg),
            // Add display mapping for Git errors
            RixError::GitError(err) => write!(f, "Git Repository Error: {}", err),
        }
    }
}

impl std::error::Error for RixError {}

impl From<std::io::Error> for RixError {
    fn from(error: std::io::Error) -> Self {
        RixError::IOError(error)
    }
}

// Add the automatic conversion trait for git2::Error
impl From<git2::Error> for RixError {
    fn from(error: git2::Error) -> Self {
        RixError::GitError(error)
    }
}
