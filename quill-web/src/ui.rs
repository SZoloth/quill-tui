//! Terminal UI rendering for Quill Web
//!
//! This module mirrors quill-cli's UI but uses ratzilla's rendering.

use ratzilla::ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use quill_core::{App, Category, Focus, InputTarget, Mode, Severity};

// Catppuccin Mocha colors
const SURFACE0: Color = Color::Rgb(49, 50, 68);
const SURFACE1: Color = Color::Rgb(69, 71, 90);
const TEXT: Color = Color::Rgb(205, 214, 244);
const SUBTEXT0: Color = Color::Rgb(166, 173, 200);
const RED: Color = Color::Rgb(243, 139, 168);
const YELLOW: Color = Color::Rgb(249, 226, 175);
const GREEN: Color = Color::Rgb(166, 227, 161);
const BLUE: Color = Color::Rgb(137, 180, 250);
const MAUVE: Color = Color::Rgb(203, 166, 247);
const TEAL: Color = Color::Rgb(148, 226, 213);

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    draw_title_bar(frame, app, chunks[0]);
    draw_main_area(frame, app, chunks[1]);
    draw_status_bar(frame, app, chunks[2]);

    // Draw popups/overlays
    match app.mode {
        Mode::SeverityPicker => draw_severity_picker(frame, app),
        Mode::CategoryPicker => draw_category_picker(frame, app),
        Mode::Input => draw_input_dialog(frame, app),
        Mode::Help => draw_help(frame),
        _ => {}
    }
}

fn draw_title_bar(frame: &mut Frame, app: &App, area: Rect) {
    let title = app.title();
    let ann_count = app
        .document
        .as_ref()
        .map(|d| d.annotations.len())
        .unwrap_or(0);

    let current = if ann_count > 0 {
        app.sidebar_selected + 1
    } else {
        0
    };

    let title_text = format!(
        " Quill TUI (Web) - {} [{}/{}]",
        title, current, ann_count
    );

    let title_bar = Paragraph::new(title_text)
        .style(Style::default().fg(TEXT).bg(SURFACE0));

    frame.render_widget(title_bar, area);
}

fn draw_main_area(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),        // Editor
            Constraint::Length(30),    // Sidebar
        ])
        .split(area);

    draw_editor(frame, app, chunks[0]);
    draw_sidebar(frame, app, chunks[1]);
}

fn draw_editor(frame: &mut Frame, app: &App, area: Rect) {
    let editor_style = if app.focus == Focus::Editor {
        Style::default().fg(BLUE)
    } else {
        Style::default().fg(SUBTEXT0)
    };

    let mode_indicator = match app.mode {
        Mode::Visual => " [VISUAL]",
        _ => "",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(editor_style)
        .title(format!("Editor{}", mode_indicator));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render document content with annotations highlighted
    if let Some(doc) = &app.document {
        let content = &doc.content;
        let annotations = doc.annotations_sorted();

        // Build styled lines
        let mut lines: Vec<Line> = Vec::new();
        let mut current_line_spans: Vec<Span> = Vec::new();
        let mut char_idx = 0;

        // Get selection range if in visual mode
        let selection = app.get_selection_range();

        for (_line_idx, line_text) in content.lines().enumerate() {
            current_line_spans.clear();
            let line_start = char_idx;

            let mut col = 0;
            for ch in line_text.chars() {
                let offset = line_start + col;

                // Determine styling for this character
                let mut style = Style::default().fg(TEXT);

                // Check if in selection
                if let Some((sel_start, sel_end)) = selection {
                    if offset >= sel_start && offset < sel_end {
                        style = style.bg(SURFACE1).add_modifier(Modifier::BOLD);
                    }
                }

                // Check if in an annotation
                for ann in &annotations {
                    if ann.range.contains(offset) {
                        let color = severity_color(ann.severity);
                        style = style.fg(color).add_modifier(Modifier::UNDERLINED);
                        break;
                    }
                }

                current_line_spans.push(Span::styled(ch.to_string(), style));
                col += ch.len_utf8();
            }

            lines.push(Line::from(current_line_spans.clone()));
            char_idx = line_start + line_text.len() + 1; // +1 for newline
        }

        // Calculate scroll offset based on cursor
        let cursor = app.cursor_pos();
        let visible_height = inner.height as usize;
        let scroll_offset = if cursor.0 >= visible_height {
            cursor.0 - visible_height + 1
        } else {
            0
        };

        let paragraph = Paragraph::new(lines)
            .scroll((scroll_offset as u16, 0))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, inner);

        // Draw cursor (in web, we don't set terminal cursor, but could highlight)
        // For now, the selection/visual mode provides visual feedback
    }
}

fn draw_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let sidebar_style = if app.focus == Focus::Sidebar {
        Style::default().fg(BLUE)
    } else {
        Style::default().fg(SUBTEXT0)
    };

    let ann_count = app
        .document
        .as_ref()
        .map(|d| d.annotations.len())
        .unwrap_or(0);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(sidebar_style)
        .title(format!("Annotations ({})", ann_count));

    if let Some(doc) = &app.document {
        let items: Vec<ListItem> = doc
            .annotations_sorted()
            .iter()
            .enumerate()
            .map(|(i, ann)| {
                let selected = i == app.sidebar_selected;
                let marker = if selected { ">" } else { " " };
                let resolved = if ann.is_resolved { "~" } else { "" };

                let severity_str = ann.severity.short();
                let text_preview: String = ann
                    .selected_text
                    .chars()
                    .take(15)
                    .collect::<String>()
                    .replace('\n', " ");

                let line1 = format!(
                    "{} [{}]{} \"{}...\"",
                    marker, severity_str, resolved, text_preview
                );
                let line2 = format!(
                    "   {}",
                    ann.comment.chars().take(20).collect::<String>()
                );

                let style = if selected {
                    Style::default().fg(TEXT).bg(SURFACE1)
                } else if ann.is_resolved {
                    Style::default().fg(SUBTEXT0)
                } else {
                    Style::default().fg(TEXT)
                };

                ListItem::new(vec![
                    Line::from(Span::styled(line1, style)),
                    Line::from(Span::styled(line2, style.fg(SUBTEXT0))),
                ])
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    } else {
        frame.render_widget(block, area);
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mode_str = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Visual => "VISUAL",
        Mode::Input => "INPUT",
        Mode::CategoryPicker => "CATEGORY",
        Mode::SeverityPicker => "SEVERITY",
        Mode::Help => "HELP",
    };

    let status = app
        .status_message
        .as_deref()
        .unwrap_or("");

    let help_hint = "j/k scroll | v select | a add | e export | ? help";

    let status_text = format!(
        " {} | {}",
        mode_str,
        if status.is_empty() { help_hint } else { status },
    );

    let status_bar = Paragraph::new(status_text)
        .style(Style::default().fg(SUBTEXT0).bg(SURFACE0));

    frame.render_widget(status_bar, area);
}

fn draw_severity_picker(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 10, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MAUVE))
        .title("Select Severity (1-3 or j/k)");

    let items: Vec<ListItem> = Severity::all()
        .iter()
        .enumerate()
        .map(|(i, sev)| {
            let selected = i == app.severity_selected;
            let marker = if selected { ">" } else { " " };
            let color = severity_color(*sev);
            let style = if selected {
                Style::default().fg(color).bg(SURFACE1)
            } else {
                Style::default().fg(color)
            };
            ListItem::new(format!("{} {} {}", i + 1, marker, sev.as_str())).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_category_picker(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 12, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MAUVE))
        .title("Select Category (0=None, j/k)");

    let mut items: Vec<ListItem> = vec![
        ListItem::new(if app.category_selected == 0 {
            "> None"
        } else {
            "  None"
        })
        .style(if app.category_selected == 0 {
            Style::default().fg(TEXT).bg(SURFACE1)
        } else {
            Style::default().fg(SUBTEXT0)
        }),
    ];

    for (i, cat) in Category::all().iter().enumerate() {
        let idx = i + 1;
        let selected = idx == app.category_selected;
        let marker = if selected { ">" } else { " " };
        let style = if selected {
            Style::default().fg(TEAL).bg(SURFACE1)
        } else {
            Style::default().fg(TEAL)
        };
        items.push(ListItem::new(format!("{} {}", marker, cat.as_str())).style(style));
    }

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_input_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 5, frame.area());
    frame.render_widget(Clear, area);

    let title = match app.input_target {
        InputTarget::Comment => "Enter comment (then press Enter)",
        InputTarget::FilePath => "Enter file path",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GREEN))
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input = Paragraph::new(format!("{}_", app.input_buffer))
        .style(Style::default().fg(TEXT));
    frame.render_widget(input, inner);
}

fn draw_help(frame: &mut Frame) {
    let area = centered_rect(60, 18, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BLUE))
        .title("Help (press any key to close)");

    let help_text = vec![
        Line::from(Span::styled("Navigation", Style::default().fg(MAUVE).add_modifier(Modifier::BOLD))),
        Line::from("  j/k      Scroll down/up"),
        Line::from("  g/G      Go to top/bottom"),
        Line::from("  ]/[      Next/prev annotation"),
        Line::from("  Tab      Toggle editor/sidebar"),
        Line::from(""),
        Line::from(Span::styled("Annotations", Style::default().fg(MAUVE).add_modifier(Modifier::BOLD))),
        Line::from("  v        Enter visual mode"),
        Line::from("  a        Add annotation (after selection)"),
        Line::from("  d        Delete annotation"),
        Line::from("  r        Toggle resolved"),
        Line::from(""),
        Line::from(Span::styled("File", Style::default().fg(MAUVE).add_modifier(Modifier::BOLD))),
        Line::from("  e        Export annotations as JSON"),
        Line::from(""),
        Line::from(Span::styled("Press any key to close", Style::default().fg(SUBTEXT0))),
    ];

    let paragraph = Paragraph::new(help_text).block(block);
    frame.render_widget(paragraph, area);
}

fn severity_color(severity: Severity) -> Color {
    match severity {
        Severity::MustFix => RED,
        Severity::ShouldFix => YELLOW,
        Severity::Consider => GREEN,
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
