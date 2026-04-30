//! Command handlers for the CLI.
//!
//! Each subcommand has its own module with a `run()` function.

pub mod init;
pub mod validate_cmd;
pub mod status;
pub mod next;
pub mod task;
pub mod decide;
pub mod discover;
pub mod phase;
pub mod session;
pub mod diff;
pub mod archive;
pub mod view;
pub mod skill;
