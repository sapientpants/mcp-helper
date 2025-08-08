//! End-to-end test framework for MCP Helper
//!
//! This module provides utilities for testing the actual CLI binary
//! with real commands and filesystem operations.

pub mod assertions;
pub mod common;

pub use assertions::*;
pub use common::*;
