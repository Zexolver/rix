#[derive(Debug)]
pub enum RixError {
    IOError(std::io::Error),
    ParseError(String),
    /// Host is missing critical Nix tooling dependencies
    MissingSystemDependency(String),
    /// The generated file contains syntax that failed validation
    InvalidNixSyntax(String),
}

impl From<std::io::Error> for RixError {
    fn from(error: std::io::Error) -> Self {
        RixError::IOError(error)
    }
}
