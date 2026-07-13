use crate::app::{App, EditorAction, Mode, StatusKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Clear, Paragraph, Row, Table, Wrap},
};
use unicode_width::UnicodeWidthStr;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [header_area, content_area, status_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(2),
    ])
    .areas(frame.area());

    draw_header(frame, header_area, app);
    draw_todos(frame, content_area, app);
    draw_status(frame, status_area, app);
    draw_help(frame, help_area, app);

    match app.mode() {
        Mode::Normal => {}

        Mode::Editing(action) => {
            draw_editor_popup(frame, app, action);
        }

        Mode::ConfirmDelete => {
            draw_delete_popup(frame, app);
        }
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = Line::from(vec![
        Span::styled(
            " TODO TUI ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            "  {} total  •  {} pending  •  {} completed",
            app.todos().len(),
            app.pending_count(),
            app.completed_count(),
        )),
    ]);

    let header = Paragraph::new(title)
        .centered()
        .block(Block::bordered().title(" Dashboard "));

    frame.render_widget(header, area);
}

fn draw_todos(frame: &mut Frame, area: Rect, app: &mut App) {
    if app.todos().is_empty() {
        let empty_state = Paragraph::new("No tasks yet.\n\nPress A to create your first task.")
            .centered()
            .block(Block::bordered().title(" Tasks "));

        frame.render_widget(empty_state, area);
        return;
    }

    let (todos, table_state) = app.table_parts();

    let rows = todos.iter().map(|todo| {
        let status_symbol = if todo.finished { "✓" } else { "○" };

        let status_style = if todo.finished {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let title_style = if todo.finished {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::CROSSED_OUT)
        } else {
            Style::default()
        };

        Row::new([
            Cell::from(status_symbol).style(status_style),
            Cell::from(todo.id.to_string()),
            Cell::from(todo.title.clone()).style(title_style),
        ])
    });

    let header = Row::new(["", "ID", "Task"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .column_spacing(1)
    .block(Block::bordered().title(" Tasks "))
    .row_highlight_style(
        Style::default()
            .fg(Color::White)
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, area, table_state);
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let mode = match app.mode() {
        Mode::Normal => "NORMAL",
        Mode::Editing(EditorAction::Add) => "ADD",
        Mode::Editing(EditorAction::Rename) => "RENAME",
        Mode::ConfirmDelete => "DELETE",
    };

    let message_style = match app.status().kind() {
        StatusKind::Info => Style::default().fg(Color::White),

        StatusKind::Success => Style::default().fg(Color::Green),

        StatusKind::Error => Style::default().fg(Color::Red),
    };

    let status = Line::from(vec![
        Span::styled(
            format!(" {mode} "),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(app.status().message().to_string(), message_style),
    ]);

    let paragraph = Paragraph::new(status).block(Block::bordered().title(" Status "));

    frame.render_widget(paragraph, area);
}

fn draw_help(frame: &mut Frame, area: Rect, app: &App) {
    let help = match app.mode() {
        Mode::Normal => Line::from(vec![
            key("↑/↓"),
            Span::raw(" move  "),
            key("A"),
            Span::raw(" add  "),
            key("E"),
            Span::raw(" rename  "),
            key("Space"),
            Span::raw(" toggle  "),
            key("D"),
            Span::raw(" delete  "),
            key("Q"),
            Span::raw(" quit"),
        ]),

        Mode::Editing(_) => Line::from(vec![
            key("Enter"),
            Span::raw(" save  "),
            key("Esc"),
            Span::raw(" cancel"),
        ]),

        Mode::ConfirmDelete => Line::from(vec![
            key("Y"),
            Span::raw(" confirm  "),
            key("N"),
            Span::raw(" cancel"),
        ]),
    };

    frame.render_widget(
        Paragraph::new(help).centered().wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_editor_popup(frame: &mut Frame, app: &App, action: EditorAction) {
    let area = centered_rect(70, 3, frame.area());

    let popup_title = match action {
        EditorAction::Add => " Add task ",
        EditorAction::Rename => " Rename task ",
    };

    let input = app.input();
    let input_width = UnicodeWidthStr::width(input);
    let inner_width = usize::from(area.width.saturating_sub(2));

    let horizontal_scroll = input_width.saturating_sub(inner_width.saturating_sub(1));

    let paragraph = Paragraph::new(input)
        .scroll((0, horizontal_scroll.min(u16::MAX as usize) as u16))
        .block(
            Block::bordered()
                .title(popup_title)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);

    if area.width > 2 {
        let cursor_offset = input_width
            .saturating_sub(horizontal_scroll)
            .min(inner_width.saturating_sub(1));

        frame.set_cursor_position((area.x + 1 + cursor_offset as u16, area.y + 1));
    }
}

fn draw_delete_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 7, frame.area());

    let task_title = app
        .selected_todo()
        .map(|todo| todo.title.as_str())
        .unwrap_or("this task");

    let text = format!("Delete \"{task_title}\"?\n\nThis action cannot be undone.");

    let popup = Paragraph::new(text)
        .centered()
        .wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .title(" Confirm deletion ")
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(Clear, area);
    frame.render_widget(popup, area);
}

fn key(label: &'static str) -> Span<'static> {
    Span::styled(
        format!(" {label} "),
        Style::default()
            .fg(Color::Black)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
}

fn centered_rect(width_percentage: u16, requested_height: u16, area: Rect) -> Rect {
    let available_width = area.width.saturating_sub(2);
    let available_height = area.height.saturating_sub(2);

    let minimum_width = 20.min(available_width);

    let width = available_width.saturating_mul(width_percentage) / 100;

    let width = width.max(minimum_width).min(available_width);

    let height = requested_height.min(available_height);

    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}
