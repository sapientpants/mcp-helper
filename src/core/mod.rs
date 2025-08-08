//! Core business logic module
//!
//! This module contains pure business logic functions that are extracted from I/O operations
//! to make them more testable and reusable. These functions perform data transformations,
//! validations, and calculations without side effects.

pub mod config;
pub mod installation;
pub mod validation;

#[cfg(test)]
mod validation_proptest;

#[cfg(test)]
mod config_proptest;

#[cfg(test)]
mod installation_proptest;
