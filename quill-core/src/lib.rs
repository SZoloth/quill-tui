//! Quill Core - Platform-agnostic text annotation library
//!
//! This crate provides the core data structures and logic for the Quill
//! text annotation tool. It's designed to work both in native CLI and
//! WASM environments.

pub mod actions;
pub mod app;
pub mod cursor;
pub mod export;
pub mod model;

pub use app::{App, Focus, InputTarget, Mode};
pub use cursor::CursorState;
pub use export::{generate_prompt, to_json, ExportAnnotation, ExportDocument};
pub use model::{Annotation, Category, Document, Severity, TextRange};
