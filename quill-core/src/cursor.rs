/// Platform-agnostic cursor state
/// Replaces tui-textarea for WASM compatibility
#[derive(Debug, Clone)]
pub struct CursorState {
    /// Current cursor position (row, col)
    pub row: usize,
    pub col: usize,
    /// Line start offsets for coordinate translation
    line_starts: Vec<usize>,
    /// Lines of content
    lines: Vec<String>,
}

impl CursorState {
    pub fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            line_starts: vec![0],
            lines: Vec::new(),
        }
    }

    /// Load content and compute line offsets
    pub fn set_content(&mut self, content: &str) {
        self.lines = content.lines().map(String::from).collect();
        self.line_starts.clear();
        self.line_starts.push(0);

        for (i, c) in content.char_indices() {
            if c == '\n' {
                self.line_starts.push(i + 1);
            }
        }

        self.row = 0;
        self.col = 0;
    }

    /// Get current cursor position as (row, col)
    pub fn cursor(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    /// Convert (row, col) to character offset
    pub fn cursor_to_offset(&self, row: usize, col: usize) -> usize {
        if row >= self.line_starts.len() {
            // Return end of content
            return self.line_starts.last().copied().unwrap_or(0)
                + self.lines.last().map(|l| l.len()).unwrap_or(0);
        }
        self.line_starts[row] + col
    }

    /// Convert character offset to (row, col)
    pub fn offset_to_cursor(&self, offset: usize) -> (usize, usize) {
        for (i, &start) in self.line_starts.iter().enumerate().rev() {
            if offset >= start {
                return (i, offset - start);
            }
        }
        (0, 0)
    }

    /// Set cursor to character offset
    pub fn set_cursor_offset(&mut self, offset: usize) {
        let (row, col) = self.offset_to_cursor(offset);
        self.row = row;
        self.col = col;
    }

    /// Get the current line content
    pub fn current_line(&self) -> Option<&str> {
        self.lines.get(self.row).map(|s| s.as_str())
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get a specific line
    pub fn line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|s| s.as_str())
    }

    // Cursor movement methods

    pub fn move_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
            // Clamp column to line length
            if let Some(line) = self.lines.get(self.row) {
                self.col = self.col.min(line.chars().count());
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.row + 1 < self.lines.len() {
            self.row += 1;
            // Clamp column to line length
            if let Some(line) = self.lines.get(self.row) {
                self.col = self.col.min(line.chars().count());
            }
        }
    }

    pub fn move_left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.row > 0 {
            // Move to end of previous line
            self.row -= 1;
            self.col = self.lines.get(self.row).map(|l| l.chars().count()).unwrap_or(0);
        }
    }

    pub fn move_right(&mut self) {
        let line_len = self.lines.get(self.row).map(|l| l.chars().count()).unwrap_or(0);
        if self.col < line_len {
            self.col += 1;
        } else if self.row + 1 < self.lines.len() {
            // Move to start of next line
            self.row += 1;
            self.col = 0;
        }
    }

    pub fn move_to_start(&mut self) {
        self.col = 0;
    }

    pub fn move_to_end(&mut self) {
        self.col = self.lines.get(self.row).map(|l| l.chars().count()).unwrap_or(0);
    }

    pub fn move_to_top(&mut self) {
        self.row = 0;
        self.col = 0;
    }

    pub fn move_to_bottom(&mut self) {
        if !self.lines.is_empty() {
            self.row = self.lines.len() - 1;
            self.col = 0;
        }
    }

    pub fn move_word_forward(&mut self) {
        if let Some(line) = self.lines.get(self.row) {
            let chars: Vec<char> = line.chars().collect();
            let mut col = self.col;

            // Skip current word (non-whitespace)
            while col < chars.len() && !chars[col].is_whitespace() {
                col += 1;
            }
            // Skip whitespace
            while col < chars.len() && chars[col].is_whitespace() {
                col += 1;
            }

            if col >= chars.len() && self.row + 1 < self.lines.len() {
                // Move to next line
                self.row += 1;
                self.col = 0;
            } else {
                self.col = col;
            }
        }
    }

    pub fn move_word_back(&mut self) {
        if self.col == 0 {
            if self.row > 0 {
                self.row -= 1;
                self.col = self.lines.get(self.row).map(|l| l.chars().count()).unwrap_or(0);
            }
            return;
        }

        if let Some(line) = self.lines.get(self.row) {
            let chars: Vec<char> = line.chars().collect();
            let mut col = self.col;

            // Skip whitespace backwards
            while col > 0 && chars.get(col - 1).map(|c| c.is_whitespace()).unwrap_or(false) {
                col -= 1;
            }
            // Skip word backwards
            while col > 0 && chars.get(col - 1).map(|c| !c.is_whitespace()).unwrap_or(false) {
                col -= 1;
            }

            self.col = col;
        }
    }
}

impl Default for CursorState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_movement() {
        let mut cursor = CursorState::new();
        cursor.set_content("Hello\nWorld\nTest");

        assert_eq!(cursor.cursor(), (0, 0));

        cursor.move_down();
        assert_eq!(cursor.cursor(), (1, 0));

        cursor.move_right();
        cursor.move_right();
        assert_eq!(cursor.cursor(), (1, 2));

        cursor.move_up();
        assert_eq!(cursor.cursor(), (0, 2));
    }

    #[test]
    fn test_offset_conversion() {
        let mut cursor = CursorState::new();
        cursor.set_content("Hello\nWorld");

        // "Hello\n" = 6 chars, "World" at offset 6
        assert_eq!(cursor.cursor_to_offset(0, 0), 0);
        assert_eq!(cursor.cursor_to_offset(0, 5), 5);
        assert_eq!(cursor.cursor_to_offset(1, 0), 6);
        assert_eq!(cursor.cursor_to_offset(1, 5), 11);

        assert_eq!(cursor.offset_to_cursor(0), (0, 0));
        assert_eq!(cursor.offset_to_cursor(6), (1, 0));
        assert_eq!(cursor.offset_to_cursor(8), (1, 2));
    }
}
