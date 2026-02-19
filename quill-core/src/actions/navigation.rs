use crate::model::Document;

/// Navigate to annotation by index in sorted list
pub fn annotation_offset_by_index(doc: &Document, index: usize) -> Option<usize> {
    doc.annotations_sorted()
        .get(index)
        .map(|a| a.range.start_offset)
}
