pub mod client;
pub mod config;
pub mod deps;
pub mod error;
pub mod install;
pub mod runner;
pub mod server;

// Re-export Platform enum so it can be used in tests
pub use runner::Platform;

// Re-export error types for external use
pub use error::{McpError, Result};
