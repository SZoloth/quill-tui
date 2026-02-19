//! Quill CLI - Terminal-based text annotation tool

mod io;
mod ui;

use std::io::stdout;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use quill_core::{generate_prompt, App, Category, Focus, InputTarget, Mode, Severity};

fn main() -> Result<()> {
    // Get file path from args
    let args: Vec<String> = std::env::args().collect();
    let file_path = args.get(1);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Load file if provided
    if let Some(path) = file_path {
        match io::load_file(path) {
            Ok(doc) => {
                app.load_document(doc);
                app.set_status(&format!("Loaded {}", path));
            }
            Err(e) => {
                app.set_status(&format!("Error: {}", e));
            }
        }
    } else {
        app.set_status("No file loaded. Pass a file path as argument.");
    }

    // Main loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    while app.running {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            // Clear status on any key
            app.clear_status();

            match app.mode {
                Mode::Normal => handle_normal_mode(app, key.code, key.modifiers),
                Mode::Visual => handle_visual_mode(app, key.code),
                Mode::Input => handle_input_mode(app, key.code),
                Mode::SeverityPicker => handle_severity_picker(app, key.code),
                Mode::CategoryPicker => handle_category_picker(app, key.code),
                Mode::Help => {
                    app.mode = Mode::Normal;
                }
            }
        }
    }
    Ok(())
}

fn handle_normal_mode(app: &mut App, code: KeyCode, _modifiers: KeyModifiers) {
    match code {
        KeyCode::Char('q') => app.running = false,
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
                match io::export_document(doc) {
                    Ok(path) => app.set_status(&format!("Exported to {}", path.display())),
                    Err(e) => app.set_status(&format!("Export failed: {}", e)),
                }
            }
        }
        KeyCode::Char('E') => {
            if let Some(doc) = &app.document {
                let prompt = generate_prompt(doc);
                // In a real app, we'd copy to clipboard or show in a pane
                app.set_status(&format!("Prompt generated ({} chars)", prompt.len()));
            }
        }

        // Open file
        KeyCode::Char('o') => {
            app.input_buffer.clear();
            app.input_target = InputTarget::FilePath;
            app.mode = Mode::Input;
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
            match app.input_target {
                InputTarget::Comment => {
                    app.complete_annotation();
                }
                InputTarget::FilePath => {
                    let path = app.input_buffer.clone();
                    match io::load_file(&path) {
                        Ok(doc) => {
                            app.load_document(doc);
                            app.set_status(&format!("Loaded {}", path));
                        }
                        Err(e) => {
                            app.set_status(&format!("Error: {}", e));
                        }
                    }
                    app.input_buffer.clear();
                    app.mode = Mode::Normal;
                }
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
        // Quick select
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
    let total = Category::all().len() + 1; // +1 for "None"

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
        // Quick select
        KeyCode::Char('0') => {
            app.pending_category = None;
            app.input_buffer.clear();
            app.input_target = InputTarget::Comment;
            app.mode = Mode::Input;
        }
        _ => {}
    }
}
