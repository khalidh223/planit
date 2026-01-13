pub mod arg;
pub mod command;
pub mod config;
pub mod core;
pub mod errors;
pub mod extensions;
pub mod prompter;
pub mod logging;
mod scheduler;
pub mod ui;

// Re-export main entry helpers if needed in future integration tests.
