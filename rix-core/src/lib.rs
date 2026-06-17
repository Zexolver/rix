pub mod compiler;
pub mod errors;
pub mod parser;
pub mod discovery;
pub mod writer;
pub mod system;
pub mod ops;
pub mod verify;
pub mod context;
pub mod hardware;
pub mod git;

pub use errors::RixError;
pub use discovery::FoundPackage;
pub use context::{Package, RixContext};
