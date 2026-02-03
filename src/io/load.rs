use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::model::Document;

/// Load a text file and create a Document
pub fn load_file(path: &str) -> Result<Document> {
    let path = Path::new(path);
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", path.display()))?;

    let content = fs::read_to_string(&canonical)
        .with_context(|| format!("Failed to read file: {}", canonical.display()))?;

    Ok(Document::from_file(
        canonical.to_string_lossy().as_ref(),
        content,
    ))
}
