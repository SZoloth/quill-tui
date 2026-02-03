use serde::{Deserialize, Serialize};

/// Represents a range of text by character offsets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TextRange {
    pub start_offset: usize,
    pub end_offset: usize,
}

impl TextRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start_offset: start.min(end),
            end_offset: start.max(end),
        }
    }

    /// Check if this range contains the given offset
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start_offset && offset < self.end_offset
    }
}
