use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::TextRange;

/// Annotation category for classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Category {
    Voice,
    Clarity,
    Structure,
    Expand,
    Condense,
    Rephrase,
}

impl Category {
    pub fn all() -> &'static [Category] {
        &[
            Category::Voice,
            Category::Clarity,
            Category::Structure,
            Category::Expand,
            Category::Condense,
            Category::Rephrase,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Voice => "Voice",
            Category::Clarity => "Clarity",
            Category::Structure => "Structure",
            Category::Expand => "Expand",
            Category::Condense => "Condense",
            Category::Rephrase => "Rephrase",
        }
    }
}

/// Severity level for annotations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    MustFix,
    ShouldFix,
    Consider,
}

impl Severity {
    pub fn all() -> &'static [Severity] {
        &[Severity::MustFix, Severity::ShouldFix, Severity::Consider]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::MustFix => "Must Fix",
            Severity::ShouldFix => "Should Fix",
            Severity::Consider => "Consider",
        }
    }

    pub fn short(&self) -> &'static str {
        match self {
            Severity::MustFix => "MUST",
            Severity::ShouldFix => "SHOULD",
            Severity::Consider => "CONSIDER",
        }
    }
}

impl Default for Severity {
    fn default() -> Self {
        Severity::ShouldFix
    }
}

/// An annotation attached to a text range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: Uuid,
    #[serde(flatten)]
    pub range: TextRange,
    #[serde(rename = "text")]
    pub selected_text: String,
    pub category: Option<Category>,
    pub severity: Severity,
    pub comment: String,
    #[serde(default)]
    pub is_resolved: bool,
}

impl Annotation {
    pub fn new(range: TextRange, selected_text: String, comment: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            range,
            selected_text,
            category: None,
            severity: Severity::default(),
            comment,
            is_resolved: false,
        }
    }
}
