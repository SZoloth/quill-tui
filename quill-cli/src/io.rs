//! File I/O for native CLI

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use quill_core::Document;

/// Load a text file and create a Document
pub fn load_file(path: &str) -> Result<Document> {
    let path = Path::new(path);
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", path.display()))?;

    let content = fs::read_to_string(&canonical)
        .with_context(|| format!("Failed to read file: {}", canonical.display()))?;

    let filepath = canonical.to_string_lossy().to_string();
    let filename = canonical
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let title = canonical
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string());

    Ok(Document::with_file_info(title, content, filepath, filename))
}

/// Get the ~/.quill directory path, creating it if needed
pub fn quill_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let quill_dir = home.join(".quill");

    if !quill_dir.exists() {
        fs::create_dir_all(&quill_dir)
            .with_context(|| format!("Failed to create {}", quill_dir.display()))?;
    }

    Ok(quill_dir)
}

/// Export document to ~/.quill/document.json
pub fn export_document(doc: &Document) -> Result<PathBuf> {
    let quill_dir = quill_dir()?;
    let export_path = quill_dir.join("document.json");

    let json = quill_core::to_json(doc)
        .context("Failed to serialize document")?;

    fs::write(&export_path, json)
        .with_context(|| format!("Failed to write {}", export_path.display()))?;

    Ok(export_path)
}
