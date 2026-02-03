use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

use crate::model::{Annotation, Document};

/// Export format matching macOS Quill app
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportDocument {
    pub filepath: Option<String>,
    pub filename: Option<String>,
    pub title: String,
    pub content: String,
    pub word_count: usize,
    pub annotations: Vec<ExportAnnotation>,
    pub prompt: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAnnotation {
    pub id: String,
    pub text: String,
    pub category: Option<String>,
    pub severity: String,
    pub comment: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

impl From<&Annotation> for ExportAnnotation {
    fn from(ann: &Annotation) -> Self {
        Self {
            id: ann.id.to_string(),
            text: ann.selected_text.clone(),
            category: ann.category.map(|c| format!("{:?}", c).to_uppercase()),
            severity: match ann.severity {
                crate::model::Severity::MustFix => "must-fix",
                crate::model::Severity::ShouldFix => "should-fix",
                crate::model::Severity::Consider => "consider",
            }
            .to_string(),
            comment: ann.comment.clone(),
            start_offset: ann.range.start_offset,
            end_offset: ann.range.end_offset,
        }
    }
}

impl From<&Document> for ExportDocument {
    fn from(doc: &Document) -> Self {
        let prompt = generate_prompt(doc);
        Self {
            filepath: doc.filepath.clone(),
            filename: doc.filename.clone(),
            title: doc.title.clone(),
            content: doc.content.clone(),
            word_count: doc.word_count(),
            annotations: doc.annotations.iter().map(ExportAnnotation::from).collect(),
            prompt,
        }
    }
}

/// Generate a Claude-ready prompt from a document
pub fn generate_prompt(doc: &Document) -> String {
    let mut prompt = String::new();

    prompt.push_str(&format!("## Document: {}\n\n", doc.title));
    prompt.push_str("Please review and edit this document based on the following annotations.\n\n");

    prompt.push_str("### Full Text\n\n");
    prompt.push_str(&doc.content);
    prompt.push_str("\n\n---\n\n");

    let unresolved: Vec<_> = doc
        .annotations
        .iter()
        .filter(|a| !a.is_resolved)
        .collect();

    if unresolved.is_empty() {
        prompt.push_str("No annotations to address.\n");
        return prompt;
    }

    prompt.push_str(&format!("### Annotations ({} items)\n\n", unresolved.len()));

    // Group by severity
    for severity in crate::model::Severity::all() {
        let items: Vec<_> = unresolved
            .iter()
            .filter(|a| a.severity == *severity)
            .collect();

        if items.is_empty() {
            continue;
        }

        prompt.push_str(&format!("#### {} ({})\n\n", severity.as_str(), items.len()));

        for ann in items {
            prompt.push_str(&format!("**\"{}\"**\n", ann.selected_text));
            if let Some(cat) = ann.category {
                prompt.push_str(&format!("- Category: {}\n", cat.as_str()));
            }
            prompt.push_str(&format!("- Feedback: {}\n\n", ann.comment));
        }
    }

    prompt.push_str("---\n\n");
    prompt.push_str("Please provide the revised document with all annotations addressed. ");
    prompt.push_str("For each change, briefly note what was modified and why.");

    prompt
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

    let export_doc = ExportDocument::from(doc);
    let json = serde_json::to_string_pretty(&export_doc)
        .context("Failed to serialize document")?;

    fs::write(&export_path, json)
        .with_context(|| format!("Failed to write {}", export_path.display()))?;

    Ok(export_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Annotation, Category, Severity, TextRange};

    #[test]
    fn test_export_annotation_format() {
        let range = TextRange::new(100, 150);
        let mut ann = Annotation::new(range, "selected text".to_string(), "fix this".to_string());
        ann.category = Some(Category::Rephrase);
        ann.severity = Severity::ShouldFix;

        let export_ann = ExportAnnotation::from(&ann);
        let json = serde_json::to_string(&export_ann).unwrap();

        // Verify camelCase field names
        assert!(json.contains("\"startOffset\":100"));
        assert!(json.contains("\"endOffset\":150"));
        assert!(json.contains("\"severity\":\"should-fix\""));
        assert!(json.contains("\"category\":\"REPHRASE\""));
        assert!(json.contains("\"text\":\"selected text\""));
        assert!(json.contains("\"comment\":\"fix this\""));
    }

    #[test]
    fn test_export_document_format() {
        let mut doc = crate::model::Document::new("Test".to_string(), "Hello world".to_string());
        doc.filepath = Some("/path/to/file.md".to_string());
        doc.filename = Some("file.md".to_string());

        let export_doc = ExportDocument::from(&doc);
        let json = serde_json::to_string(&export_doc).unwrap();

        // Verify camelCase field names
        assert!(json.contains("\"wordCount\":2"));
        assert!(json.contains("\"filepath\":\"/path/to/file.md\""));
    }
}
