pub mod compiler;
pub mod context;
pub mod discovery;
pub mod errors;
pub mod git;
pub mod hardware;
pub mod ops;
pub mod parser;
pub mod system;
pub mod verify;
pub mod writer;

pub use context::{Package, RixContext};
pub use discovery::FoundPackage;
pub use errors::RixError;
