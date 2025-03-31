use anyhow::Result;
use ratatui::{
    Frame, Terminal,
    backend::Backend,
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Color,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table},
};
use std::time::Duration;

use crate::app::{App, InputMode, SortColumn};
use crate::utils::centered_rect;

/// Main app logic
pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        app.refresh();
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::Char('k') => app.input_mode = InputMode::ConfirmKill,
                        KeyCode::Char('/') => {
                            app.input_mode = InputMode::Search;
                            app.search_query.clear();
                        }
                        KeyCode::Char('p') => app.toggle_sort(SortColumn::Pid),
                        KeyCode::Char('n') => app.toggle_sort(SortColumn::Name),
                        KeyCode::Char('m') => app.toggle_sort(SortColumn::Memory),
                        _ => {}
                    },
                    InputMode::Search => match key.code {
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.search_query.clear();
                            app.apply_filters();
                        }
                        KeyCode::Enter => {
                            app.input_mode = InputMode::Normal;
                            app.apply_filters();
                        }
                        KeyCode::Backspace => {
                            app.search_query.pop();
                            app.apply_filters();
                        }
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.apply_filters();
                        }
                        _ => {}
                    },
                    InputMode::ConfirmKill => match key.code {
                        /// Is this better than 'n' for "No"?
                        KeyCode::Char('y') => app.kill_selected_process(),
                        _ => app.input_mode = InputMode::Normal,
                    },
                }
            }
        }
    }
}

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),     // Process table
            Constraint::Length(10), // Process details
            Constraint::Length(3),  // Help bar
        ])
        .margin(1)
        .split(f.area());

    // Process table
    render_process_table(f, app, chunks[0]);

    // Process details panel
    render_process_details(f, app, chunks[1]);

    // Help section
    render_help_bar(f, app, chunks[2]);

    // Render popups
    match app.input_mode {
        InputMode::Search => render_search_popup(f, app),
        InputMode::ConfirmKill => render_kill_confirmation(f, app),
        _ => {}
    }

    // Show message if any
    if let Some((message, color)) = &app.message {
        render_message(f, message, *color);
    }
}

fn render_process_table(f: &mut Frame, app: &mut App, area: Rect) {
    // Get sort indicators
    let pid_sort = if app.sort_column == SortColumn::Pid {
        if app.sort_ascending { " ↑" } else { " ↓" }
    } else {
        ""
    };

    let name_sort = if app.sort_column == SortColumn::Name {
        if app.sort_ascending { " ↑" } else { " ↓" }
    } else {
        ""
    };

    let mem_sort = if app.sort_column == SortColumn::Memory {
        if app.sort_ascending { " ↑" } else { " ↓" }
    } else {
        ""
    };

    let header_cells = [
        Cell::from(format!("PID{}", pid_sort)).style(Style::default().fg(Color::Green)),
        Cell::from(format!("Name{}", name_sort)).style(Style::default().fg(Color::Green)),
        Cell::from(format!("Memory (MB){}", mem_sort)).style(Style::default().fg(Color::Green)),
    ];

    let header = Row::new(header_cells)
        .style(Style::default())
        .height(1)
        .bottom_margin(1);

    let rows = app.filtered_processes.iter().map(|&i| {
        let process = &app.processes[i];
        let mem_color = if process.memory_mb > 500.0 {
            Color::Red
        } else if process.memory_mb > 100.0 {
            Color::Yellow
        } else {
            Color::White
        };

        let cells = [
            Cell::from(process.pid.clone()),
            Cell::from(process.name.clone()),
            Cell::from(format!("{:.2}", process.memory_mb)).style(Style::default().fg(mem_color)),
        ];
        Row::new(cells).height(1)
    });

    let title = format!(
        "Process Information ({} processes)",
        app.filtered_processes.len()
    );

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(55),
            Constraint::Percentage(30),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title),
    )
    .row_highlight_style(Style::default().fg(Color::Yellow).bold())
    .highlight_symbol("> ");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_process_details(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Process Details");

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if let Some(process) = app.selected_process() {
        let details = vec![
            Line::from(vec!["PID: ".into(), process.pid.clone().yellow()]),
            Line::from(vec!["Name: ".into(), process.name.clone().yellow()]),
            Line::from(vec![
                "Memory: ".into(),
                format!("{:.2} MB", process.memory_mb).yellow(),
            ]),
            // Add more details here as needed
        ];

        let text = Paragraph::new(details).alignment(Alignment::Left);

        f.render_widget(text, inner_area);
    }
}

fn render_help_bar(f: &mut Frame, app: &App, area: Rect) {
    let mut help_text = vec![
        "↑/↓".fg(Color::Yellow),
        " Navigate   ".into(),
        "p/n/m".fg(Color::Yellow),
        " Sort by PID/Name/Memory   ".into(),
        "/".fg(Color::Yellow),
        " Search   ".into(),
        "k".fg(Color::Yellow),
        " Kill Process   ".into(),
        "q".fg(Color::Yellow),
        " Quit".into(),
    ];

    if !app.search_query.is_empty() {
        help_text.push("   Filter: ".into());
        help_text.push(app.search_query.clone().blue());
    }

    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL).title("Controls"));

    f.render_widget(help, area);
}

fn render_search_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 3, f.area());
    let popup_block = Block::default()
        .title("Search")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::DarkGray));

    f.render_widget(Clear, area); // Clear the area
    f.render_widget(popup_block, area);

    let text = Paragraph::new(format!("> {}", app.search_query))
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::NONE));

    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width - 2,
        height: 1,
    };

    f.render_widget(text, inner_area);

    // Set cursor position
    f.set_cursor_position((
        inner_area.x + app.search_query.len() as u16 + 2,
        inner_area.y,
    ));
}
fn render_kill_confirmation(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 5, f.area());
    let popup_block = Block::default()
        .title("Confirm Kill Process")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::DarkGray));

    f.render_widget(Clear, area); // Clear the area
    f.render_widget(popup_block, area);

    let process_name = app
        .selected_process()
        .map(|p| &p.name as &str)
        .unwrap_or("");

    let text = Paragraph::new(vec![
        Line::from(format!(
            "Are you sure you want to kill process: {}?",
            process_name
        ))
        .style(Style::default().fg(Color::Red)),
        Line::from(""),
        Line::from("Press (Y) to confirm, any other key to cancel."),
    ])
    .alignment(Alignment::Center);

    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width - 2,
        height: area.height - 2,
    };

    f.render_widget(text, inner_area);
}

fn render_message(f: &mut Frame, message: &str, color: Color) {
    let area = centered_rect(50, 3, f.area());

    // Clear the area first
    f.render_widget(Clear, area);

    // Calculate inner area manually instead of using popup_block.inner()
    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width - 2,
        height: area.height - 2,
    };

    // Create and render the block
    let popup_block = Block::default()
        .title("Message")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    f.render_widget(popup_block, area);

    // Create and render the text in the inner area
    let text = Paragraph::new(message)
        .style(Style::default().fg(color))
        .alignment(Alignment::Center);

    f.render_widget(text, inner_area);
}
