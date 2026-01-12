//! Common utilities and test infrastructure for Claude Agent SDK demos.
//!
//! This module provides shared functionality across all demo projects including:
//! - Mock transport implementation for testing
//! - Test helpers and assertion utilities
//! - Common demo patterns and utilities

pub mod test_utils;

pub use test_utils::{MockTransport, StreamCollector, TestFixture};
