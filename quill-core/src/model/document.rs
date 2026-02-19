use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Annotation;

/// A document with annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filepath: Option<String>,
    pub annotations: Vec<Annotation>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Document {
    pub fn new(title: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            content,
            filename: None,
            filepath: None,
            annotations: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a document with filename metadata (used by CLI when loading from file)
    pub fn with_file_info(title: String, content: String, filepath: String, filename: String) -> Self {
        let mut doc = Self::new(title, content);
        doc.filepath = Some(filepath);
        doc.filename = Some(filename);
        doc
    }

    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
        self.updated_at = Utc::now();
    }

    pub fn remove_annotation(&mut self, id: Uuid) -> Option<Annotation> {
        if let Some(pos) = self.annotations.iter().position(|a| a.id == id) {
            self.updated_at = Utc::now();
            Some(self.annotations.remove(pos))
        } else {
            None
        }
    }

    pub fn toggle_resolved(&mut self, id: Uuid) -> bool {
        if let Some(ann) = self.annotations.iter_mut().find(|a| a.id == id) {
            ann.is_resolved = !ann.is_resolved;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Get annotations sorted by start offset
    pub fn annotations_sorted(&self) -> Vec<&Annotation> {
        let mut sorted: Vec<_> = self.annotations.iter().collect();
        sorted.sort_by_key(|a| a.range.start_offset);
        sorted
    }
}
