#![allow(warnings)]
pub mod app;
pub mod config;
pub mod files;
pub mod git;
pub mod tui;

// Re-export commonly used items
pub use tui::theme::{AccentColor, Theme};
