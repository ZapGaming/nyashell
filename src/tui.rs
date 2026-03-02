use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{stdout, Write};

use crate::NyaShellConfig;

pub fn run_tui(config: NyaShellConfig) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut current_tab = 0;
    let mut command_input = String::new();
    let mut command_history: Vec<String> = Vec::new();
    let mut git_status = GitStatus::default();

    // Main loop
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Top bar
                    Constraint::Min(1),     // Main content
                    Constraint::Length(3),  // Command input
                ])
                .split(f.size());

            // Top bar with tabs and Git status
            render_top_bar(f, chunks[0], current_tab, &git_status, config.show_git);

            // Main content area
            match current_tab {
                0 => render_commands(f, chunks[1]),
                1 => render_history(f, chunks[1], &command_history),
                2 => render_settings(f, chunks[1], &config),
                _ => {}
            }

            // Command input
            render_command_input(f, chunks[2], &command_input);
        })?;

        // Handle input
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char(c) => {
                    command_input.push(c);
                }
                KeyCode::Backspace => {
                    command_input.pop();
                }
                KeyCode::Enter => {
                    if !command_input.is_empty() {
                        command_history.push(command_input.clone());
                        // Here you would execute the command via Nushell
                        // For now, just clear
                        command_input.clear();
                    }
                }
                KeyCode::Tab => {
                    current_tab = (current_tab + 1) % 3;
                }
                KeyCode::Esc => {
                    break;
                }
                _ => {}
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn render_top_bar<B: ratatui::backend::Backend>(
    f: &mut ratatui::Frame<B>,
    area: Rect,
    current_tab: usize,
    git_status: &GitStatus,
    show_git: bool,
) {
    let tabs = ["Commands", "History", "Settings"];
    let tab_spans: Vec<Spans> = tabs
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            if i == current_tab {
                Spans::from(vec![Span::styled(
                    format!(" {} ", name),
                    Style::default().fg(Color::Black).bg(Color::White),
                )])
            } else {
                Spans::from(vec![Span::raw(format!(" {} ", name))])
            }
        })
        .collect();

    let mut text = Text::from(tab_spans);

    // Add Git status if enabled and available
    if show_git && !git_status.branch.is_empty() {
        text.lines.push(Spans::from(vec![
            Span::raw(" | "),
            Span::styled(
                format!(" {}", git_status.branch),
                Style::default().fg(Color::Green),
            ),
            Span::raw(format!(" | +{} -{} {}", git_status.changes, git_status.deletions, git_status.untracked)),
        ]));
    }

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan))
    );
    f.render_widget(paragraph, area);
}

fn render_commands<B: ratatui::backend::Backend>(f: &mut ratatui::Frame<B>, area: Rect) {
    let commands = [
        ("ls", "List files"),
        ("pwd", "Print working directory"),
        ("git status", "Show git status"),
        ("sys", "System info"),
        ("help", "Show help"),
        ("echo", "Print text"),
        ("cd", "Change directory"),
        ("mkdir", "Create directory"),
        ("rm", "Remove file"),
        ("cp", "Copy file"),
    ];

    let items: Vec<ListItem> = commands
        .iter()
        .map(|(cmd, desc)| {
            ListItem::new(Spans::from(vec![
                Span::styled(format!("{:<12}", cmd), Style::default().fg(Color::Yellow)),
                Span::raw(format!(" - {}", desc)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Quick Commands"))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list, area);
}

fn render_history<B: ratatui::backend::Backend>(
    f: &mut ratatui::Frame<B>,
    area: Rect,
    history: &[String],
) {
    let items: Vec<ListItem> = history
        .iter()
        .rev()
        .take(20)  // Show last 20 commands
        .map(|cmd| ListItem::new(Spans::from(Span::raw(cmd))))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Command History"))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list, area);
}

fn render_settings<B: ratatui::backend::Backend>(
    f: &mut ratatui::Frame<B>,
    area: Rect,
    config: &NyaShellConfig,
) {
    let settings = [
        ("Theme", &config.theme),
        ("Show Git Status", &if config.show_git { "Yes" } else { "No" }),
        ("Font Size", &config.font_size.to_string()),
    ];

    let rows: Vec<Row> = settings
        .iter()
        .map(|(key, value)| {
            Row::new(vec![
                Span::styled(*key, Style::default().fg(Color::Cyan)),
                Span::raw(": "),
                Span::styled(*value, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let table = Table::new(rows)
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .column_spacing(1);

    f.render_widget(table, area);
}

fn render_command_input<B: ratatui::backend::Backend>(
    f: &mut ratatui::Frame<B>,
    area: Rect,
    input: &str,
) {
    let prompt = Span::styled("❯", Style::default().fg(Color::Green));
    let input_span = Span::raw(input);
    let cursor = if input.is_empty() {
        Span::styled(" ", Style::default().bg(Color::White))
    } else {
        Span::styled(" ", Style::default().bg(Color::White))
    };

    let text = Spans::from(vec![prompt, Span::raw(" "), input_span, cursor]);

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default());

    f.render_widget(paragraph, area);
}

#[derive(Default, Debug)]
struct GitStatus {
    branch: String,
    changes: usize,
    deletions: usize,
    untracked: usize,
}