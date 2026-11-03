//! PackageDownloader (pdown) — Core Library
//! Cross-platform development environment mirror manager.

pub mod app;
pub mod core;
pub mod infrastructure;
pub mod providers;
pub mod system;

pub use core::errors::{PdownError, Result};
