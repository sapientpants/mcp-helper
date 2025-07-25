pub mod client;
pub mod deps;
pub mod install;
pub mod runner;
pub mod server;

// Re-export Platform enum so it can be used in tests
pub use runner::Platform;
