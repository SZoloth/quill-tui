//! Quill Web - WebAssembly version of the text annotation tool
//!
//! This crate provides a browser-based version of Quill using Ratzilla
//! for terminal rendering in the DOM.

use std::cell::RefCell;
use std::rc::Rc;

use ratzilla::ratatui::Terminal;
use ratzilla::{event::KeyCode, DomBackend, WebRenderer};
use wasm_bindgen::prelude::*;

use quill_core::{App, Category, Focus, InputTarget, Mode, Severity};

pub mod io;
mod ui;

/// Sample document content for demo
const SAMPLE_CONTENT: &str = r#"# Welcome to Quill TUI

This is a demonstration of Quill running in your browser via WebAssembly.

## Introduction

Claude is an AI assistant made by Anthropic. It is designed to be helpful, harmless, and honest. This paragraph could use some clarity improvements.

## Main Content

Here is some content that might need editing:

- First point that could be expanded
- Second point with unclear wording
- Third point to consider rephrasing

The voice in this section doesn't quite match the rest of the document. It feels too formal and could be more conversational.

## Conclusion

In conclusion, this document demonstrates the annotation capabilities of Quill TUI. Users can select text, add annotations with severity levels and categories, then export for Claude to process.

Try it out:
1. Press 'v' to enter visual mode
2. Use j/k to select text
3. Press 'a' to add an annotation
4. Use 'e' to export your annotations
"#;

/// Initialize the Quill web application
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    // Create app with sample document
    let mut app = App::new();
    let doc = quill_core::Document::new("Demo Document".to_string(), SAMPLE_CONTENT.to_string());
    app.load_document(doc);
    app.set_status("Welcome to Quill! Press 'v' to start selecting, '?' for help");

    // Wrap in Rc<RefCell> for shared state
    let app_state = Rc::new(RefCell::new(app));

    // Create terminal with DOM backend
    let backend = DomBackend::new()
        .map_err(|e| JsValue::from_str(&format!("Failed to create backend: {:?}", e)))?;
    let terminal = Terminal::new(backend)
        .map_err(|e| JsValue::from_str(&format!("Failed to create terminal: {:?}", e)))?;

    // Set up keyboard handler
    terminal.on_key_event({
        let app_state_cloned = app_state.clone();
        move |event| {
            let mut app = app_state_cloned.borrow_mut();
            app.clear_status();

            match app.mode {
                Mode::Normal => handle_normal_mode(&mut app, event.code),
                Mode::Visual => handle_visual_mode(&mut app, event.code),
                Mode::Input => handle_input_mode(&mut app, event.code),
                Mode::SeverityPicker => handle_severity_picker(&mut app, event.code),
                Mode::CategoryPicker => handle_category_picker(&mut app, event.code),
                Mode::Help => {
                    app.mode = Mode::Normal;
                }
            }
        }
    });

    // Draw loop
    terminal.draw_web(move |frame| {
        let app = app_state.borrow();
        ui::draw(frame, &app);
    });

    web_sys::console::log_1(&"Quill WASM initialized".into());

    Ok(())
}

fn handle_normal_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('?') => app.mode = Mode::Help,

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            if app.focus == Focus::Editor {
                app.move_down();
            } else {
                app.next_annotation();
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.focus == Focus::Editor {
                app.move_up();
            } else {
                app.prev_annotation();
            }
        }
        KeyCode::Char('h') | KeyCode::Left => {
            app.move_left();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.move_right();
        }
        KeyCode::Char('g') => {
            app.move_to_top();
        }
        KeyCode::Char('G') => {
            app.move_to_bottom();
        }

        // Annotation navigation
        KeyCode::Char(']') => app.next_annotation(),
        KeyCode::Char('[') => app.prev_annotation(),

        // Visual mode
        KeyCode::Char('v') => app.enter_visual_mode(),

        // Annotation actions
        KeyCode::Char('d') => {
            app.delete_selected_annotation();
        }
        KeyCode::Char('r') => {
            app.toggle_selected_resolved();
        }

        // Focus toggle
        KeyCode::Tab => app.toggle_focus(),

        // Export
        KeyCode::Char('e') => {
            if let Some(doc) = &app.document {
                match quill_core::to_json(doc) {
                    Ok(json) => {
                        if let Err(e) = io::download_json("quill-export.json", &json) {
                            app.set_status(&format!("Export failed: {:?}", e));
                        } else {
                            app.set_status("Exported to quill-export.json");
                        }
                    }
                    Err(e) => app.set_status(&format!("Serialization failed: {}", e)),
                }
            }
        }

        _ => {}
    }
}

fn handle_visual_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.selection_start = None;
            app.selection_end = None;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_down();
            app.update_selection();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_up();
            app.update_selection();
        }
        KeyCode::Char('h') | KeyCode::Left => {
            app.move_left();
            app.update_selection();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.move_right();
            app.update_selection();
        }
        KeyCode::Char('w') => {
            app.move_word_forward();
            app.update_selection();
        }
        KeyCode::Char('b') => {
            app.move_word_back();
            app.update_selection();
        }
        KeyCode::Char('a') => {
            app.start_annotation();
        }
        _ => {}
    }
}

fn handle_input_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.input_buffer.clear();
            app.pending_range = None;
        }
        KeyCode::Enter => {
            if app.input_target == InputTarget::Comment {
                app.complete_annotation();
            }
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        _ => {}
    }
}

fn handle_severity_picker(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.pending_range = None;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.severity_selected = (app.severity_selected + 1) % Severity::all().len();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let len = Severity::all().len();
            app.severity_selected = if app.severity_selected == 0 {
                len - 1
            } else {
                app.severity_selected - 1
            };
        }
        KeyCode::Enter => {
            app.pending_severity = Severity::all()[app.severity_selected];
            app.mode = Mode::CategoryPicker;
        }
        KeyCode::Char('1') => {
            app.pending_severity = Severity::MustFix;
            app.mode = Mode::CategoryPicker;
        }
        KeyCode::Char('2') => {
            app.pending_severity = Severity::ShouldFix;
            app.mode = Mode::CategoryPicker;
        }
        KeyCode::Char('3') => {
            app.pending_severity = Severity::Consider;
            app.mode = Mode::CategoryPicker;
        }
        _ => {}
    }
}

fn handle_category_picker(app: &mut App, code: KeyCode) {
    let total = Category::all().len() + 1;

    match code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.pending_range = None;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.category_selected = (app.category_selected + 1) % total;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.category_selected = if app.category_selected == 0 {
                total - 1
            } else {
                app.category_selected - 1
            };
        }
        KeyCode::Enter => {
            app.pending_category = if app.category_selected == 0 {
                None
            } else {
                Some(Category::all()[app.category_selected - 1])
            };
            app.input_buffer.clear();
            app.input_target = InputTarget::Comment;
            app.mode = Mode::Input;
        }
        KeyCode::Char('0') => {
            app.pending_category = None;
            app.input_buffer.clear();
            app.input_target = InputTarget::Comment;
            app.mode = Mode::Input;
        }
        _ => {}
    }
}
