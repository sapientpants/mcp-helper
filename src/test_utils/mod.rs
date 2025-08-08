//! Common test utilities for MCP Helper tests
//!
//! This module provides shared test utilities including mocks, fixtures, and custom assertions
//! that can be used across different test modules to reduce duplication and improve consistency.

#[cfg(any(test, debug_assertions))]
pub mod mocks;

#[cfg(any(test, debug_assertions))]
pub mod fixtures;

#[cfg(any(test, debug_assertions))]
pub mod assertions;
