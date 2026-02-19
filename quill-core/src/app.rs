use crate::actions;
use crate::cursor::CursorState;
use crate::model::{Annotation, Category, Document, Severity, TextRange};

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Visual,
    Input,
    CategoryPicker,
    SeverityPicker,
    Help,
}

/// Focus area
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Editor,
    Sidebar,
}

/// Input target for text input mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputTarget {
    Comment,
    FilePath,
}

/// Platform-agnostic application state
pub struct App {
    pub document: Option<Document>,
    pub cursor: CursorState,
    pub mode: Mode,
    pub focus: Focus,
    pub running: bool,

    // Selection state
    pub selection_start: Option<(usize, usize)>, // (row, col)
    pub selection_end: Option<(usize, usize)>,

    // Sidebar state
    pub sidebar_selected: usize,

    // Input state
    pub input_buffer: String,
    pub input_target: InputTarget,

    // Picker state
    pub category_selected: usize,
    pub severity_selected: usize,

    // Pending annotation (during creation workflow)
    pub pending_range: Option<TextRange>,
    pub pending_category: Option<Category>,
    pub pending_severity: Severity,

    // Status message
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            document: None,
            cursor: CursorState::new(),
            mode: Mode::Normal,
            focus: Focus::Editor,
            running: true,

            selection_start: None,
            selection_end: None,

            sidebar_selected: 0,

            input_buffer: String::new(),
            input_target: InputTarget::Comment,

            category_selected: 0,
            severity_selected: 1, // Default to ShouldFix

            pending_range: None,
            pending_category: None,
            pending_severity: Severity::ShouldFix,

            status_message: None,
        }
    }

    pub fn load_document(&mut self, doc: Document) {
        self.cursor.set_content(&doc.content);
        self.document = Some(doc);
        self.sidebar_selected = 0;
    }

    /// Get cursor position as (row, col)
    pub fn cursor_pos(&self) -> (usize, usize) {
        self.cursor.cursor()
    }

    /// Convert (row, col) to character offset
    pub fn cursor_to_offset(&self, row: usize, col: usize) -> usize {
        self.cursor.cursor_to_offset(row, col)
    }

    /// Convert character offset to (row, col)
    pub fn offset_to_cursor(&self, offset: usize) -> (usize, usize) {
        self.cursor.offset_to_cursor(offset)
    }

    /// Set cursor to character offset
    pub fn set_cursor_offset(&mut self, offset: usize) {
        self.cursor.set_cursor_offset(offset);
    }

    // Cursor movement methods
    pub fn move_up(&mut self) {
        self.cursor.move_up();
    }

    pub fn move_down(&mut self) {
        self.cursor.move_down();
    }

    pub fn move_left(&mut self) {
        self.cursor.move_left();
    }

    pub fn move_right(&mut self) {
        self.cursor.move_right();
    }

    pub fn move_to_top(&mut self) {
        self.cursor.move_to_top();
    }

    pub fn move_to_bottom(&mut self) {
        self.cursor.move_to_bottom();
    }

    pub fn move_word_forward(&mut self) {
        self.cursor.move_word_forward();
    }

    pub fn move_word_back(&mut self) {
        self.cursor.move_word_back();
    }

    /// Enter visual/selection mode
    pub fn enter_visual_mode(&mut self) {
        self.mode = Mode::Visual;
        let cursor = self.cursor.cursor();
        self.selection_start = Some(cursor);
        self.selection_end = Some(cursor);
    }

    /// Exit visual mode and get selection range
    pub fn exit_visual_mode(&mut self) -> Option<TextRange> {
        if self.mode != Mode::Visual {
            return None;
        }

        let start = self.selection_start?;
        let end = self.selection_end?;

        let start_offset = self.cursor_to_offset(start.0, start.1);
        let end_offset = self.cursor_to_offset(end.0, end.1);

        self.mode = Mode::Normal;
        self.selection_start = None;
        self.selection_end = None;

        if start_offset != end_offset {
            Some(TextRange::new(start_offset, end_offset))
        } else {
            None
        }
    }

    /// Update selection end position
    pub fn update_selection(&mut self) {
        if self.mode == Mode::Visual {
            self.selection_end = Some(self.cursor.cursor());
        }
    }

    /// Get selection range for highlighting
    pub fn get_selection_range(&self) -> Option<(usize, usize)> {
        if self.mode != Mode::Visual {
            return None;
        }

        let start = self.selection_start?;
        let end = self.selection_end?;

        let start_offset = self.cursor_to_offset(start.0, start.1);
        let end_offset = self.cursor_to_offset(end.0, end.1);

        Some((start_offset.min(end_offset), start_offset.max(end_offset)))
    }

    /// Start annotation creation workflow
    pub fn start_annotation(&mut self) {
        if let Some(range) = self.exit_visual_mode() {
            self.pending_range = Some(range);
            self.mode = Mode::SeverityPicker;
        }
    }

    /// Complete annotation creation
    pub fn complete_annotation(&mut self) -> bool {
        let range = match self.pending_range.take() {
            Some(r) => r,
            None => return false,
        };

        let doc = match self.document.as_mut() {
            Some(d) => d,
            None => return false,
        };

        let selected_text = doc.content[range.start_offset..range.end_offset].to_string();
        let mut annotation = Annotation::new(range, selected_text, self.input_buffer.clone());
        annotation.category = self.pending_category;
        annotation.severity = self.pending_severity;

        doc.add_annotation(annotation);

        // Reset state
        self.input_buffer.clear();
        self.pending_category = None;
        self.pending_severity = Severity::ShouldFix;
        self.mode = Mode::Normal;

        self.set_status("Annotation added");
        true
    }

    /// Get currently selected annotation
    pub fn selected_annotation(&self) -> Option<&Annotation> {
        let doc = self.document.as_ref()?;
        let sorted = doc.annotations_sorted();
        sorted.get(self.sidebar_selected).copied()
    }

    /// Navigate to next annotation
    pub fn next_annotation(&mut self) {
        if let Some(doc) = &self.document {
            let count = doc.annotations.len();
            if count > 0 {
                self.sidebar_selected = (self.sidebar_selected + 1) % count;
                if let Some(offset) = actions::annotation_offset_by_index(doc, self.sidebar_selected) {
                    self.set_cursor_offset(offset);
                }
            }
        }
    }

    /// Navigate to previous annotation
    pub fn prev_annotation(&mut self) {
        if let Some(doc) = &self.document {
            let count = doc.annotations.len();
            if count > 0 {
                self.sidebar_selected = if self.sidebar_selected == 0 {
                    count - 1
                } else {
                    self.sidebar_selected - 1
                };
                if let Some(offset) = actions::annotation_offset_by_index(doc, self.sidebar_selected) {
                    self.set_cursor_offset(offset);
                }
            }
        }
    }

    /// Delete selected annotation
    pub fn delete_selected_annotation(&mut self) -> bool {
        let id = match self.selected_annotation() {
            Some(a) => a.id,
            None => return false,
        };

        if let Some(doc) = self.document.as_mut() {
            if doc.remove_annotation(id).is_some() {
                // Adjust selection if needed
                let count = doc.annotations.len();
                if self.sidebar_selected >= count && count > 0 {
                    self.sidebar_selected = count - 1;
                }
                self.set_status("Annotation deleted");
                return true;
            }
        }
        false
    }

    /// Toggle resolved status of selected annotation
    pub fn toggle_selected_resolved(&mut self) -> bool {
        let id = match self.selected_annotation() {
            Some(a) => a.id,
            None => return false,
        };

        if let Some(doc) = self.document.as_mut() {
            if doc.toggle_resolved(id) {
                self.set_status("Toggled resolved status");
                return true;
            }
        }
        false
    }

    /// Set status message
    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some(msg.to_string());
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Toggle focus between editor and sidebar
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Editor => Focus::Sidebar,
            Focus::Sidebar => Focus::Editor,
        };
    }

    /// Get title for display
    pub fn title(&self) -> String {
        self.document
            .as_ref()
            .and_then(|d| d.filename.clone())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    /// Get content lines for rendering
    pub fn content_lines(&self) -> Vec<&str> {
        self.document
            .as_ref()
            .map(|d| d.content.lines().collect())
            .unwrap_or_default()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
