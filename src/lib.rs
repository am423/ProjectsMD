//! projectsmd library.
//!
//! Public API for parsing, validating, and rendering project.md files.

pub mod frontmatter;
pub mod sections;
pub mod project;
pub mod tasks;
pub mod requirements;
pub mod decisions;
pub mod state;
pub mod session_log;
pub mod validate;
pub mod render;
pub mod template;
pub mod skill;
pub mod commands;

pub use project::Project;
