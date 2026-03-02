use anyhow::Result;
use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
    Terminal,
};
use std::{
    env::current_dir,
    io::stdout,
    process::Command,
    time::{Duration, Instant},
};
use sysinfo::{System, SystemExt, DiskExt, NetworkExt, CpuExt, NetworksExt};

mod config;

#[derive(Debug, Clone)]
struct AppState {
    current_tab: usize,
    command_input: String,
    command_history: Vec<String>,
    output_history: Vec<OutputEntry>,
    settings: config::Settings,
    git_status: GitStatus,
    dropdown_open: Option<DropdownType>,
    selected_setting: usize,
    settings_selected: usize,
    current_dir: String,
    command_filter: String,
    file_filter: String,
    files: Vec<FileInfo>,
    search_results: Vec<SearchResult>,
    history_selected: usize,
    guide_selected: usize,
    files_selected: usize,
    editing_setting: Option<usize>,
    last_update: Instant,
}

#[derive(Debug, Clone)]
struct OutputEntry {
    command: String,
    output: String,
    success: bool,
    timestamp: Instant,
}

#[derive(Debug, Clone)]
struct GitStatus {
    branch: String,
    staged: usize,
    unstaged: usize,
    ahead: usize,
    behind: usize,
}

impl GitStatus {
    fn new() -> Self {
        Self {
            branch: "main".to_string(),
            staged: 0,
            unstaged: 2,
            ahead: 1,
            behind: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct FileInfo {
    name: String,
    is_dir: bool,
    size: u64,
}

#[derive(Debug, Clone)]
struct SearchResult {
    path: String,
    line_num: usize,
    content: String,
}

#[derive(Debug, Clone)]
enum DropdownType {
    Theme,
    AccentColor,
    Background,
}

const QUICK_COMMANDS: &[(&str, &str)] = &[
    ("ls -la", "List all files"),
    ("pwd", "Print working directory"),
    ("git status", "Check git status"),
    ("git log --oneline", "View commit history"),
    ("clear", "Clear output"),
    ("help", "Show help"),
    ("exit", "Exit NyaShell"),
];

const ALL_COMMANDS: &[(&str, &str, &str)] = &[
    ("ls -la", "Navigation", "List all files with details"),
    ("pwd", "Navigation", "Print current working directory"),
    ("cd <dir>", "Navigation", "Change directory"),
    ("clear", "System", "Clear output history"),
    ("help", "System", "Show help information"),
    ("exit", "System", "Exit NyaShell"),
    ("git status", "Git", "Show git repository status"),
    ("git log --oneline", "Git", "Show commit history"),
    ("git diff", "Git", "Show unstaged changes"),
    ("git add <file>", "Git", "Stage file for commit"),
    ("git commit -m \"msg\"", "Git", "Commit staged changes"),
    ("git push", "Git", "Push commits to remote"),
    ("git pull", "Git", "Pull from remote"),
    ("node <file.js>", "Node.js", "Execute JavaScript file"),
    ("node -i", "Node.js", "Open Node.js REPL"),
    ("npm <command>", "Node.js", "Run npm command"),
    ("npx <package>", "Node.js", "Execute npx package"),
    ("python <file.py>", "Python", "Execute Python file"),
    ("python -i", "Python", "Open Python REPL"),
    ("pip <command>", "Python", "Run pip command"),
    ("py <file.py>", "Python", "Execute Python (Windows)"),
    ("dir", "File Operations", "List files (Windows)"),
    ("type <file>", "File Operations", "Display file contents"),
    ("find <pattern>", "File Operations", "Search for files"),
    ("grep <pattern> <file>", "File Operations", "Search in files"),
    ("cat <file>", "File Operations", "Display file contents"),
    ("mkdir <dir>", "File Operations", "Create directory"),
    ("rm <file>", "File Operations", "Remove file"),
    ("rmdir <dir>", "File Operations", "Remove directory"),
    ("echo <text>", "System", "Print text"),
    ("date", "System", "Show current date/time"),
    ("whoami", "System", "Show current user"),
    ("hostname", "System", "Show computer name"),
];

const THEMES: &[(&str, &str)] = &[
    ("catboy", "Catboy Pink/Purple"),
    ("neon", "Neon Cyberpunk"),
    ("monokai", "Monokai Pro"),
    ("dracula", "Dracula Dark"),
    ("pastel", "Pastel Gradient"),
    ("sunset", "Sunset Orange"),
    ("ocean", "Ocean Blue"),
    ("forest", "Forest Green"),
];

const ACCENT_COLORS: &[(&str, &str)] = &[
    ("#ff6b9d", "Pink"),
    ("#00ff00", "Neon Green"),
    ("#00ffff", "Cyan"),
    ("#ff00ff", "Magenta"),
    ("#ffff00", "Yellow"),
    ("#ff6b6b", "Coral Red"),
    ("#4ecdc4", "Turquoise"),
    ("#a8e6cf", "Mint Green"),
];

const BACKGROUNDS: &[(&str, &str)] = &[
    ("#0d0d0d", "Deep Black"),
    ("#1a1a2e", "Midnight Blue"),
    ("#16213e", "Dark Navy"),
    ("#000033", "Deep Blue"),
    ("#2d1b2e", "Dark Purple"),
    ("#1b2d2e", "Dark Teal"),
    ("#2e1b1b", "Dark Red"),
    ("#0f0f0f", "Soft Black"),
];

fn main() -> Result<()> {
    let current_dir = match current_dir() {
        Ok(dir) => dir.to_string_lossy().to_string(),
        Err(_) => "C:\\".to_string(),
    };

    let mut app_state = AppState {
        current_tab: 0,
        command_input: String::new(),
        command_history: Vec::new(),
        output_history: Vec::new(),
        settings: config::Settings::load(),
        git_status: GitStatus::new(),
        dropdown_open: None,
        selected_setting: 0,
        settings_selected: 0,
        current_dir,
        command_filter: String::new(),
        file_filter: String::new(),
        files: Vec::new(),
        search_results: Vec::new(),
        history_selected: 0,
        guide_selected: 0,
        files_selected: 0,
        editing_setting: None,
        last_update: Instant::now(),
    };

    app_state.files = scan_directory(&app_state.current_dir);
    app_state.output_history.push(OutputEntry {
        command: "Welcome to NyaShell TUI!".to_string(),
        output: "✨ Enhanced terminal with Node.js & Python support\n📚 6 tabs: Commands, History, Guide, Files, Settings, System Monitor\n🔍 Search commands and files with arrow keys\n🎨 Customize with gradient pastel themes\n💻 Execute JavaScript and Python files!\n📋 Ctrl+V or Shift+Insert to paste\n📊 Real-time system monitoring".to_string(),
        success: true,
        timestamp: Instant::now(),
    });

    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    loop {
        terminal.draw(|f| render_ui(f, &app_state))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => {
                            if app_state.dropdown_open.is_some() {
                                app_state.dropdown_open = None;
                            } else if app_state.editing_setting.is_some() {
                                app_state.editing_setting = None;
                            } else {
                                // Exit application when no dropdown is open
                                break;
                            }
                        }
                        KeyCode::Tab => {
                            if app_state.dropdown_open.is_none() {
                                app_state.current_tab = (app_state.current_tab + 1) % 6;
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(dropdown) = app_state.dropdown_open.take() {
                                match dropdown {
                                    DropdownType::Theme => {
                                        app_state.settings.theme = THEMES[app_state.selected_setting].0.to_string();
                                        let _ = app_state.settings.save();
                                    }
                                    DropdownType::AccentColor => {
                                        app_state.settings.accent_color = ACCENT_COLORS[app_state.selected_setting].0.to_string();
                                        let _ = app_state.settings.save();
                                    }
                                    DropdownType::Background => {
                                        app_state.settings.background = BACKGROUNDS[app_state.selected_setting].0.to_string();
                                        let _ = app_state.settings.save();
                                    }
                                }
                            } else if app_state.current_tab == 4 && app_state.dropdown_open.is_none() {
                                match app_state.settings_selected {
                                    0 => app_state.dropdown_open = Some(DropdownType::Theme),
                                    1 => app_state.dropdown_open = Some(DropdownType::AccentColor),
                                    2 => app_state.dropdown_open = Some(DropdownType::Background),
                                    _ => {}
                                }
                            } else if !app_state.command_input.is_empty() && app_state.dropdown_open.is_none() {
                                let cmd = app_state.command_input.clone();
                                if cmd == "exit" {
                                    break;
                                } else if cmd == "settings" {
                                    app_state.current_tab = 4;
                                } else if cmd.starts_with("cd ") {
                                    let new_dir = cmd[3..].trim();
                                    if let Ok(_) = std::fs::canonicalize(new_dir) {
                                        app_state.current_dir = new_dir.to_string();
                                        app_state.files = scan_directory(&app_state.current_dir);
                                    }
                                } else if cmd == "clear" {
                                    app_state.output_history.clear();
                                } else {
                                    app_state.command_history.push(cmd.clone());
                                    let (output, success) = execute_command(&cmd, &app_state.current_dir);
                                    app_state.output_history.push(OutputEntry {
                                        command: cmd,
                                        output: output,
                                        success,
                                        timestamp: Instant::now(),
                                    });
                                }
                                app_state.command_input.clear();
                            }
                        }
                        KeyCode::Up => {
                            if let Some(dropdown) = &mut app_state.dropdown_open {
                                match dropdown {
                                    DropdownType::Theme => {
                                        app_state.selected_setting = (app_state.selected_setting + THEMES.len() - 1) % THEMES.len()
                                    }
                                    DropdownType::AccentColor => {
                                        app_state.selected_setting = (app_state.selected_setting + ACCENT_COLORS.len() - 1) % ACCENT_COLORS.len()
                                    }
                                    DropdownType::Background => {
                                        app_state.selected_setting = (app_state.selected_setting + BACKGROUNDS.len() - 1) % BACKGROUNDS.len()
                                    }
                                }
                            } else {
                                match app_state.current_tab {
                                    1 => {
                                        if app_state.history_selected > 0 {
                                            app_state.history_selected -= 1;
                                        }
                                    }
                                    2 => {
                                        if app_state.guide_selected > 0 {
                                            app_state.guide_selected -= 1;
                                        }
                                    }
                                    3 => {
                                        if app_state.files_selected > 0 {
                                            app_state.files_selected -= 1;
                                        }
                                    }
                                    4 => {
                                        if app_state.settings_selected > 0 {
                                            app_state.settings_selected -= 1;
                                        } else {
                                            app_state.settings_selected = 6;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        KeyCode::Down => {
                            if let Some(dropdown) = &mut app_state.dropdown_open {
                                match dropdown {
                                    DropdownType::Theme => {
                                        app_state.selected_setting = (app_state.selected_setting + 1) % THEMES.len()
                                    }
                                    DropdownType::AccentColor => {
                                        app_state.selected_setting = (app_state.selected_setting + 1) % ACCENT_COLORS.len()
                                    }
                                    DropdownType::Background => {
                                        app_state.selected_setting = (app_state.selected_setting + 1) % BACKGROUNDS.len()
                                    }
                                }
                            } else {
                                match app_state.current_tab {
                                    1 => {
                                        let max = app_state.command_history.len().saturating_sub(1);
                                        if app_state.history_selected < max {
                                            app_state.history_selected += 1;
                                        }
                                    }
                                    2 => {
                                        let filtered_count = if app_state.command_filter.is_empty() {
                                            ALL_COMMANDS.len()
                                        } else {
                                            ALL_COMMANDS.iter().filter(|(cmd, _, _)| {
                                                cmd.to_lowercase().contains(&app_state.command_filter.to_lowercase())
                                            }).count()
                                        };
                                        if app_state.guide_selected < filtered_count.saturating_sub(1) {
                                            app_state.guide_selected += 1;
                                        }
                                    }
                                    3 => {
                                        let displayed_count = if app_state.file_filter.is_empty() {
                                            app_state.files.len()
                                        } else {
                                            search_files(&app_state.current_dir, &app_state.file_filter).len()
                                        };
                                        if app_state.files_selected < displayed_count.saturating_sub(1) {
                                            app_state.files_selected += 1;
                                        }
                                    }
                                    4 => {
                                        app_state.settings_selected = (app_state.settings_selected + 1) % 7;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        KeyCode::Char('v') if key_event.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            if app_state.dropdown_open.is_none() {
                                if let Ok(mut ctx) = ClipboardContext::new() {
                                    if let Ok(text) = ctx.get_contents() {
                                        if app_state.current_tab == 0 {
                                            app_state.command_input.push_str(&text);
                                        } else if app_state.current_tab == 2 {
                                            app_state.command_filter.push_str(&text);
                                        } else if app_state.current_tab == 3 {
                                            app_state.file_filter.push_str(&text);
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Insert if key_event.modifiers.contains(event::KeyModifiers::SHIFT) => {
                            if app_state.dropdown_open.is_none() {
                                if let Ok(mut ctx) = ClipboardContext::new() {
                                    if let Ok(text) = ctx.get_contents() {
                                        if app_state.current_tab == 0 {
                                            app_state.command_input.push_str(&text);
                                        } else if app_state.current_tab == 2 {
                                            app_state.command_filter.push_str(&text);
                                        } else if app_state.current_tab == 3 {
                                            app_state.file_filter.push_str(&text);
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            if app_state.dropdown_open.is_none() {
                                if app_state.current_tab == 0 || app_state.current_tab == 4 {
                                    app_state.command_input.push(c);
                                }
                                if app_state.current_tab == 2 {
                                    app_state.command_filter.push(c);
                                }
                                if app_state.current_tab == 3 {
                                    app_state.file_filter.push(c);
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            if app_state.dropdown_open.is_none() {
                                if !app_state.command_input.is_empty() {
                                    app_state.command_input.pop();
                                }
                                if app_state.current_tab == 2 && !app_state.command_filter.is_empty() {
                                    app_state.command_filter.pop();
                                }
                                if app_state.current_tab == 3 && !app_state.file_filter.is_empty() {
                                    app_state.file_filter.pop();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn execute_command(cmd: &str, cwd: &str) -> (String, bool) {
    if cmd.trim() == "clear" {
        return ("Output cleared!".to_string(), true);
    }
    if cmd.trim() == "help" {
        let help_text = "NyaShell TUI Commands:\n\nNavigation:\n  pwd - Print working directory\n  cd <dir> - Change directory\n\nFile Operations:\n  ls -la - List all files\n  dir - List files (Windows)\n  type <file> - Display file\n  cat <file> - Display file\n  find <pattern> - Search files\n  grep <pattern> <file> - Search in file\n\nGit:\n  git status - Repository status\n  git log --oneline - Commit history\n  git diff - Show changes\n  git commit -m \"msg\" - Commit\n  git push/pull - Sync remote\n\nNode.js:\n  node <file.js> - Execute JS\n  node -i - Open REPL\n  npm <cmd> - Run npm\n  npx <pkg> - Execute package\n\nPython:\n  python <file.py> - Execute Python\n  python -i - Open REPL\n  pip <cmd> - Run pip\n  py <file.py> - Execute (Windows)\n\nSystem:\n  clear - Clear output\n  exit - Quit NyaShell";
        return (help_text.to_string(), true);
    }

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return ("".to_string(), true);
    }

    let executable = parts[0].to_lowercase();
    let args = &parts[1..];

    let output = if executable == "node" || executable == "npm" || executable == "npx" {
        Command::new(executable)
            .args(args)
            .current_dir(cwd)
            .output()
    } else if executable == "python" || executable == "pip" || executable == "py" {
        let python_cmd = if executable == "py" { "python" } else { &executable };
        Command::new(python_cmd)
            .args(args)
            .current_dir(cwd)
            .output()
    } else {
        Command::new("powershell")
            .args(["-NoProfile", "-Command", cmd])
            .current_dir(cwd)
            .output()
    };

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout).to_string();
            let stderr = String::from_utf8_lossy(&result.stderr).to_string();
            let success = result.status.success();
            let mut full_output = stdout;
            if !stderr.is_empty() {
                if !full_output.is_empty() {
                    full_output.push('\n');
                }
                full_output.push_str(&stderr);
            }
            if full_output.is_empty() {
                full_output = if success {
                    "Command executed successfully (no output).".to_string()
                } else {
                    "Command failed with no output.".to_string()
                };
            }
            (full_output, success)
        }
        Err(e) => (format!("Failed to execute command: {}", e), false),
    }
}

fn scan_directory(path: &str) -> Vec<FileInfo> {
    let mut files = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let name = entry.file_name().to_string_lossy().to_string();
                files.push(FileInfo {
                    name: name.clone(),
                    is_dir: metadata.is_dir(),
                    size: metadata.len(),
                });
            }
        }
    }
    
    files.sort_by(|a, b| {
        let a_is_dir = a.is_dir as i32;
        let b_is_dir = b.is_dir as i32;
        b_is_dir.cmp(&a_is_dir).then_with(|| a.name.cmp(&b.name))
    });
    
    files
}

fn search_files(dir: &str, query: &str) -> Vec<FileInfo> {
    let all = scan_directory(dir);
    if query.is_empty() {
        return all;
    }
    all.into_iter()
        .filter(|f| f.name.to_lowercase().contains(&query.to_lowercase()))
        .collect()
}

fn render_ui(f: &mut ratatui::Frame, state: &AppState) {
    let size = f.size();

    let (pink, purple, cyan, dark_bg, highlight) = get_theme_colors(&state.settings);

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(size);

    render_header(f, main_chunks[0], state, pink, purple, cyan, dark_bg);
    render_content(f, main_chunks[1], state, pink, purple, cyan, dark_bg, highlight);
    render_input(f, main_chunks[2], state, pink, cyan, dark_bg);
}

fn get_theme_colors(settings: &config::Settings) -> (Color, Color, Color, Color, Color) {
    let pink = Color::Rgb(255, 107, 157);
    let purple = Color::Rgb(123, 104, 238);
    let cyan = Color::Rgb(86, 182, 194);
    let dark_bg = Color::Rgb(13, 13, 13);
    let highlight = Color::Rgb(30, 30, 40);

    match settings.theme.as_str() {
        "neon" => {
            let neon_pink = Color::Rgb(255, 0, 128);
            let neon_purple = Color::Rgb(189, 0, 255);
            let neon_cyan = Color::Rgb(0, 255, 255);
            let neon_dark = Color::Rgb(10, 10, 30);
            let neon_highlight = Color::Rgb(40, 0, 60);
            (neon_pink, neon_purple, neon_cyan, neon_dark, neon_highlight)
        }
        "monokai" => {
            let mono_pink = Color::Rgb(249, 38, 114);
            let mono_purple = Color::Rgb(166, 227, 121);
            let mono_cyan = Color::Rgb(117, 113, 94);
            let mono_dark = Color::Rgb(39, 40, 34);
            let mono_highlight = Color::Rgb(98, 100, 84);
            (mono_pink, mono_purple, mono_cyan, mono_dark, mono_highlight)
        }
        "dracula" => {
            let drag_pink = Color::Rgb(255, 121, 198);
            let drag_purple = Color::Rgb(189, 147, 249);
            let drag_cyan = Color::Rgb(139, 233, 253);
            let drag_dark = Color::Rgb(40, 42, 54);
            let drag_highlight = Color::Rgb(68, 71, 90);
            (drag_pink, drag_purple, drag_cyan, drag_dark, drag_highlight)
        }
        "pastel" => {
            let pastel_pink = Color::Rgb(255, 183, 178);
            let pastel_purple = Color::Rgb(199, 144, 255);
            let pastel_cyan = Color::Rgb(178, 235, 242);
            let pastel_dark = Color::Rgb(30, 30, 50);
            let pastel_highlight = Color::Rgb(60, 60, 90);
            (pastel_pink, pastel_purple, pastel_cyan, pastel_dark, pastel_highlight)
        }
        "sunset" => {
            let sunset_pink = Color::Rgb(255, 154, 0);
            let sunset_purple = Color::Rgb(237, 117, 57);
            let sunset_cyan = Color::Rgb(255, 206, 84);
            let sunset_dark = Color::Rgb(20, 15, 30);
            let sunset_highlight = Color::Rgb(50, 35, 60);
            (sunset_pink, sunset_purple, sunset_cyan, sunset_dark, sunset_highlight)
        }
        "ocean" => {
            let ocean_pink = Color::Rgb(0, 229, 255);
            let ocean_purple = Color::Rgb(72, 219, 251);
            let ocean_cyan = Color::Rgb(144, 224, 239);
            let ocean_dark = Color::Rgb(0, 20, 40);
            let ocean_highlight = Color::Rgb(0, 60, 100);
            (ocean_pink, ocean_purple, ocean_cyan, ocean_dark, ocean_highlight)
        }
        "forest" => {
            let forest_pink = Color::Rgb(0, 255, 127);
            let forest_purple = Color::Rgb(60, 179, 113);
            let forest_cyan = Color::Rgb(152, 251, 152);
            let forest_dark = Color::Rgb(0, 30, 15);
            let forest_highlight = Color::Rgb(0, 60, 30);
            (forest_pink, forest_purple, forest_cyan, forest_dark, forest_highlight)
        }
        _ => (pink, purple, cyan, dark_bg, highlight),
    }
}

fn render_system_monitor_tab(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    pink: Color,
    purple: Color,
    cyan: Color,
    dark_bg: Color,
    highlight: Color,
) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled("📊 ", Style::default().fg(pink)),
            Span::styled("System Monitor", Style::default().fg(cyan).add_modifier(ratatui::style::Modifier::BOLD)),
        ]))
        .borders(Borders::ALL)
        .style(Style::default().bg(dark_bg))
        .border_style(Style::default().fg(purple));

    let mut sys = System::new_all();
    sys.refresh_all();

    // CPU info
    let cpu_usage = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
    let cpu_count = sys.cpus().len();
    
    // Memory info
    let total_mem = sys.total_memory();
    let used_mem = total_mem - sys.free_memory();
    let mem_percent = (used_mem as f64 / total_mem as f64) * 100.0;

    // Disk info (first disk)
    let disks = sys.disks();
    let (disk_name, disk_total, disk_used, disk_percent) = if let Some(disk) = disks.first() {
        let total = disk.total_space() as u64;
        let used = total - disk.available_space() as u64;
        let percent = (used as f64 / total as f64) * 100.0;
        (disk.name().to_string_lossy().to_string(), total, used, percent)
    } else {
        ("None".to_string(), 0, 0, 0.0)
    };

    // Network info - get first network interface
    let networks = sys.networks();
    let (net_name, net_rx, net_tx) = if let Some((name, data)) = networks.iter().next() {
        (name.clone(), data.received() as u64, data.transmitted() as u64)
    } else {
        ("None".to_string(), 0, 0)
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("CPU", Style::default().fg(purple).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::raw(": "),
            Span::styled(format!("{:.1}%", cpu_usage), Style::default().fg(if cpu_usage > 80.0 { Color::Red } else if cpu_usage > 50.0 { Color::Yellow } else { Color::Green })),
            Span::raw(" ("),
            Span::styled(format!("{} cores", cpu_count), Style::default().fg(cyan)),
            Span::raw(")"),
        ]),
        Line::from(vec![
            Span::styled("Memory", Style::default().fg(purple).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::raw(": "),
            Span::styled(format!("{:.1}%", mem_percent), Style::default().fg(if mem_percent > 80.0 { Color::Red } else if mem_percent > 50.0 { Color::Yellow } else { Color::Green })),
            Span::raw(" ("),
            Span::styled(format!("{} / {}", format_file_size(used_mem), format_file_size(total_mem)), Style::default().fg(cyan)),
            Span::raw(")"),
        ]),
        Line::from(vec![
            Span::styled("Disk", Style::default().fg(purple).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::raw(": "),
            Span::styled(format!("{:.1}%", disk_percent), Style::default().fg(if disk_percent > 80.0 { Color::Red } else if disk_percent > 50.0 { Color::Yellow } else { Color::Green })),
            Span::raw(" ("),
            Span::styled(&disk_name, Style::default().fg(cyan)),
            Span::raw(")"),
        ]),
        Line::from(vec![
            Span::styled("Network", Style::default().fg(purple).add_modifier(ratatui::style::Modifier::BOLD)),
            Span::raw(": "),
            Span::styled(format!("↓ {} ↑ {}", format_file_size(net_rx), format_file_size(net_tx)), Style::default().fg(cyan)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}


fn render_header(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    pink: Color,
    purple: Color,
    cyan: Color,
    dark_bg: Color,
) {
    let title = Line::from(vec![
        Span::styled("✧ NyaShell TUI ✧", Style::default().fg(pink).add_modifier(ratatui::style::Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled("Tab: Switch", Style::default().fg(cyan)),
        Span::raw(" | "),
        Span::styled("Type 'settings' for config", Style::default().fg(purple)),
        Span::raw(" | "),
        Span::styled("Esc: Close", Style::default().fg(Color::Yellow)),
    ]);

    let git_info = if state.settings.show_git {
        let branch_color = if state.git_status.ahead > 0 { Color::Green } else { cyan };
        let status_color = if state.git_status.unstaged > 0 { Color::Red } else { Color::Green };
        Line::from(vec![
            Span::raw(" ⎇ "),
            Span::styled(&state.git_status.branch, Style::default().fg(branch_color)),
            Span::raw(" "),
            Span::styled(
                format!("✗{}", state.git_status.unstaged),
                Style::default().fg(status_color),
            ),
            Span::raw(" "),
            Span::styled(
                format!("+{}", state.git_status.staged),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" "),
            Span::styled(
                format!("↑{}", state.git_status.ahead),
                Style::default().fg(Color::Blue),
            ),
        ])
    } else {
        Line::from("")
    };

    let dir_info = Line::from(vec![
        Span::raw(" 📁 "),
        Span::styled(&state.current_dir, Style::default().fg(cyan)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(dark_bg))
        .border_style(Style::default().fg(purple));

    f.render_widget(block, area);

    let title_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width - 2,
        height: 1,
    };
    f.render_widget(Paragraph::new(title), title_area);

    let git_area = Rect {
        x: area.x + area.width / 2,
        y: area.y,
        width: area.width / 2 - 1,
        height: 1,
    };
    f.render_widget(Paragraph::new(git_info), git_area);

    let dir_area = Rect {
        x: area.x,
        y: area.y + 2,
        width: area.width,
        height: 1,
    };
    f.render_widget(Paragraph::new(dir_info), dir_area);

    let tabs = Tabs::new(vec![
        Line::from(vec![Span::styled("⚡ Commands", Style::default().fg(if state.current_tab == 0 { pink } else { cyan }))]),
        Line::from(vec![Span::styled("📜 History", Style::default().fg(if state.current_tab == 1 { pink } else { cyan }))]),
        Line::from(vec![Span::styled("🔍 Command Guide", Style::default().fg(if state.current_tab == 2 { pink } else { cyan }))]),
        Line::from(vec![Span::styled("📁 Files", Style::default().fg(if state.current_tab == 3 { pink } else { cyan }))]),
        Line::from(vec![Span::styled("⚙️ Settings", Style::default().fg(if state.current_tab == 4 { pink } else { cyan }))]),
        Line::from(vec![Span::styled("📊 System Monitor", Style::default().fg(if state.current_tab == 5 { pink } else { cyan }))]),
    ])
    .select(state.current_tab)
    .style(Style::default().fg(cyan))
    .highlight_style(Style::default().fg(pink).add_modifier(ratatui::style::Modifier::BOLD))
    .divider(Span::raw("│"));

    let tabs_area = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: 1,
    };
    f.render_widget(tabs, tabs_area);
}

fn render_content(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    pink: Color,
    purple: Color,
    cyan: Color,
    dark_bg: Color,
    highlight: Color,
) {
    match state.current_tab {
        0 => render_commands_tab(f, area, state, pink, purple, cyan, dark_bg, highlight),
        1 => render_history_tab(f, area, state, pink, purple, cyan, dark_bg, highlight),
        2 => render_guide_tab(f, area, state, pink, purple, cyan, dark_bg, highlight),
        3 => render_files_tab(f, area, state, pink, purple, cyan, dark_bg, highlight),
        4 => render_settings_tab(f, area, state, pink, purple, cyan, dark_bg, highlight),
        5 => render_system_monitor_tab(f, area, state, pink, purple, cyan, dark_bg, highlight),
        _ => {}
    }
}

fn render_commands_tab(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    _pink: Color,
    purple: Color,
    cyan: Color,
    dark_bg: Color,
    highlight: Color,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(1)])
        .split(area);

    let commands_block = Block::default()
        .title(Line::from(vec![
            Span::styled("⚡ ", Style::default().fg(purple)),
            Span::styled("Quick Commands", Style::default().fg(cyan).add_modifier(ratatui::style::Modifier::BOLD)),
        ]))
        .borders(Borders::ALL)
        .style(Style::default().bg(dark_bg))
        .border_style(Style::default().fg(purple));

    let command_items: Vec<ListItem> = QUICK_COMMANDS
        .iter()
        .map(|(cmd, desc)| {
            ListItem::new(Line::from(vec![
                Span::styled(*cmd, Style::default().fg(cyan).add_modifier(ratatui::style::Modifier::BOLD)),
                Span::raw(" → "),
                Span::styled(*desc, Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();

    let commands_list = List::new(command_items)
        .block(commands_block)
        .highlight_style(Style::default().bg(highlight).fg(Color::White))
        .highlight_symbol("▶ ");

    f.render_widget(commands_list, chunks[0]);

    if !state.output_history.is_empty() {
        let output_block = Block::default()
            .title(Line::from(vec![
                Span::styled("📤 ", Style::default().fg(purple)),
                Span::styled("Output", Style::default().fg(cyan)),
            ]))
            .borders(Borders::ALL)
            .style(Style::default().bg(dark_bg))
            .border_style(Style::default().fg(purple));

        let mut output_lines = Vec::new();
        for entry in state.output_history.iter().rev().take(8) {
            let status_icon = if entry.success { "✓" } else { "✗" };
            let status_color = if entry.success { Color::Green } else { Color::Red };
            output_lines.push(Line::from(vec![
                Span::styled(format!("{} ", status_icon), Style::default().fg(status_color)),
                Span::styled(&entry.command, Style::default().fg(purple)),
            ]));
            for line in entry.output.lines().take(3) {
                output_lines.push(Line::from(Span::raw(line)));
            }
            output_lines.push(Line::from(""));
        }

        let output_paragraph = Paragraph::new(output_lines)
            .block(output_block)
            .wrap(Wrap { trim: true });

        f.render_widget(output_paragraph, chunks[1]);
    }
}

fn render_history_tab(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    pink: Color,
    purple: Color,
    cyan: Color,
    dark_bg: Color,
    highlight: Color,
) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled("📜 ", Style::default().fg(purple)),
            Span::styled("Command History", Style::default().fg(cyan).add_modifier(ratatui::style::Modifier::BOLD)),
        ]))
        .borders(Borders::ALL)
        .style(Style::default().bg(dark_bg))
        .border_style(Style::default().fg(purple));

    let items: Vec<ListItem> = state
        .command_history
        .iter()
        .rev()
        .take(50)
        .enumerate()
        .map(|(i, cmd)| {
            let is_selected = i == state.history_selected;
            let style = if is_selected {
                Style::default().bg(highlight).fg(Color::White)
            } else {
                Style::default().fg(cyan)
            };
            ListItem::new(Line::from(vec![
                Span::raw("$ "),
                Span::styled(cmd, style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(highlight).fg(Color::White))
        .highlight_symbol("▶ ");

    f.render_widget(list, area);
}

fn render_guide_tab(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    pink: Color,
    purple: Color,
    cyan: Color,
    dark_bg: Color,
    highlight: Color,
) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled("🔍 ", Style::default().fg(pink)),
            Span::styled("Command Guide", Style::default().fg(cyan).add_modifier(ratatui::style::Modifier::BOLD)),
        ]))
        .borders(Borders::ALL)
        .style(Style::default().bg(dark_bg))
        .border_style(Style::default().fg(purple));

    let filter_text = if state.command_filter.is_empty() {
        "Type to filter commands...".to_string()
    } else {
        format!("Filter: {}", state.command_filter)
    };

    let filtered: Vec<&(&str, &str, &str)> = ALL_COMMANDS
        .iter()
        .filter(|(cmd, _, _)| {
            cmd.to_lowercase().contains(&state.command_filter.to_lowercase()) ||
            state.command_filter.is_empty()
        })
        .collect();

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, (cmd, category, desc))| {
            let is_selected = i == state.guide_selected;
            let category_color = match *category {
                "Navigation" => Color::Green,
                "File Operations" => Color::Yellow,
                "Git" => Color::Red,
                "Node.js" => Color::Rgb(255, 202, 0),
                "Python" => Color::Rgb(53, 114, 165),
                "System" => Color::Gray,
                _ => cyan,
            };
            let style = if is_selected {
                Style::default().bg(highlight).fg(Color::White)
            } else {
                Style::default().fg(cyan)
            };
            ListItem::new(Line::from(vec![
                Span::styled(*cmd, style.add_modifier(ratatui::style::Modifier::BOLD)),
                Span::raw("  ["),
                Span::styled(*category, Style::default().fg(category_color)),
                Span::raw("]  "),
                Span::styled(*desc, Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(highlight).fg(Color::White))
        .highlight_symbol("▶ ");

    f.render_widget(list, area);

    let filter_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    let filter_span = Span::styled(&filter_text, Style::default().fg(if state.command_filter.is_empty() { Color::Gray } else { cyan }));
    f.render_widget(Paragraph::new(Line::from(filter_span)), filter_area);
}

fn render_files_tab(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    pink: Color,
    purple: Color,
    cyan: Color,
    dark_bg: Color,
    highlight: Color,
) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled("📁 ", Style::default().fg(pink)),
            Span::styled("File Explorer", Style::default().fg(cyan).add_modifier(ratatui::style::Modifier::BOLD)),
        ]))
        .borders(Borders::ALL)
        .style(Style::default().bg(dark_bg))
        .border_style(Style::default().fg(purple));

    let filter_text = if state.file_filter.is_empty() {
        "Filter files...".to_string()
    } else {
        format!("Filter: {}", state.file_filter)
    };

    let displayed_files = if state.file_filter.is_empty() {
        state.files.clone()
    } else {
        search_files(&state.current_dir, &state.file_filter)
    };

    let items: Vec<ListItem> = displayed_files
        .iter()
        .take(30)
        .enumerate()
        .map(|(i, file)| {
            let is_selected = i == state.files_selected;
            let icon = if file.is_dir { "📁" } else { "📄" };
            let size_str = if file.is_dir {
                "<DIR>".to_string()
            } else {
                format_file_size(file.size)
            };
            let style = if is_selected {
                Style::default().bg(highlight).fg(Color::White)
            } else {
                Style::default().fg(cyan)
            };
            ListItem::new(Line::from(vec![
                Span::styled(icon, Style::default().fg(purple)),
                Span::raw(" "),
                Span::styled(&file.name, style),
                Span::raw("  "),
                Span::styled(size_str, Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(highlight).fg(Color::White))
        .highlight_symbol("▶ ");

    f.render_widget(list, area);

    let filter_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    let filter_span = Span::styled(&filter_text, Style::default().fg(if state.file_filter.is_empty() { Color::Gray } else { cyan }));
    f.render_widget(Paragraph::new(Line::from(filter_span)), filter_area);
}

fn render_settings_tab(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    pink: Color,
    purple: Color,
    cyan: Color,
    dark_bg: Color,
    highlight: Color,
) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled("⚙️ ", Style::default().fg(pink)),
            Span::styled("Settings", Style::default().fg(cyan).add_modifier(ratatui::style::Modifier::BOLD)),
        ]))
        .borders(Borders::ALL)
        .style(Style::default().bg(dark_bg))
        .border_style(Style::default().fg(purple));

    let settings_text = vec![
        format!("Theme: {}", state.settings.theme),
        format!("Accent: {}", state.settings.accent_color),
        format!("Background: {}", state.settings.background),
        format!("Cursor Style: {}", state.settings.cursor_style),
        format!("Animation Speed: {}", state.settings.animation_speed),
        format!("Font Size: {}px", state.settings.font_size),
        format!("Show Git: {}", if state.settings.show_git { "Yes" } else { "No" }),
        "".to_string(),
        "💡 Tips:".to_string(),
        "• Press Enter on a setting to change it".to_string(),
        "• Use ↑↓ to navigate settings".to_string(),
        "• Type 'settings' + Enter to open settings".to_string(),
        "• Arrow keys navigate all lists".to_string(),
        "• Ctrl+V or Shift+Insert to paste".to_string(),
    ];

    let settings_items: Vec<ListItem> = settings_text.iter().enumerate().map(|(i, text)| {
        let is_selectable = (0..7).contains(&i) || i == 8;
        let is_selected = is_selectable && i == state.settings_selected && state.dropdown_open.is_none();
        let style = if is_selected {
            Style::default().bg(highlight).fg(Color::White)
        } else {
            Style::default().fg(cyan)
        };
        ListItem::new(Line::from(Span::raw(text.clone()))).style(style)
    }).collect();

    let settings_list = List::new(settings_items)
        .block(block)
        .highlight_style(Style::default().bg(highlight).fg(Color::White))
        .highlight_symbol("▶ ");

    f.render_widget(settings_list, area);

    if let Some(dropdown) = &state.dropdown_open {
        match dropdown {
            DropdownType::Theme => {
                let items = THEMES.to_vec();
                let dropdown_items: Vec<String> = items.iter().map(|(k, v)| format!("{} - {}", k, v)).collect();
                let dropdown_list: Vec<ListItem> = dropdown_items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let is_selected = i == state.selected_setting;
                        let style = if is_selected {
                            Style::default().bg(highlight).fg(Color::White)
                        } else {
                            Style::default().fg(cyan)
                        };
                        ListItem::new(Line::from(Span::raw(item.clone()))).style(style)
                    })
                    .collect();
                let dropdown_area = Rect {
                    x: area.x + 2,
                    y: area.y + 8,
                    width: area.width - 4,
                    height: (dropdown_items.len() as u16).min(10) + 2,
                };
                let dropdown_block = Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().bg(dark_bg))
                    .border_style(Style::default().fg(pink));
                let list = List::new(dropdown_list)
                    .block(dropdown_block)
                    .highlight_style(Style::default().bg(highlight).fg(Color::White));
                f.render_widget(list, dropdown_area);
            }
            DropdownType::AccentColor => {
                let items = ACCENT_COLORS.to_vec();
                let dropdown_items: Vec<String> = items.iter().map(|(k, v)| format!("{} - {}", k, v)).collect();
                let dropdown_list: Vec<ListItem> = dropdown_items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let is_selected = i == state.selected_setting;
                        let style = if is_selected {
                            Style::default().bg(highlight).fg(Color::White)
                        } else {
                            Style::default().fg(cyan)
                        };
                        ListItem::new(Line::from(Span::raw(item.clone()))).style(style)
                    })
                    .collect();
                let dropdown_area = Rect {
                    x: area.x + 2,
                    y: area.y + 8,
                    width: area.width - 4,
                    height: (dropdown_items.len() as u16).min(10) + 2,
                };
                let dropdown_block = Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().bg(dark_bg))
                    .border_style(Style::default().fg(pink));
                let list = List::new(dropdown_list)
                    .block(dropdown_block)
                    .highlight_style(Style::default().bg(highlight).fg(Color::White));
                f.render_widget(list, dropdown_area);
            }
            DropdownType::Background => {
                let items = BACKGROUNDS.to_vec();
                let dropdown_items: Vec<String> = items.iter().map(|(k, v)| format!("{} - {}", k, v)).collect();
                let dropdown_list: Vec<ListItem> = dropdown_items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let is_selected = i == state.selected_setting;
                        let style = if is_selected {
                            Style::default().bg(highlight).fg(Color::White)
                        } else {
                            Style::default().fg(cyan)
                        };
                        ListItem::new(Line::from(Span::raw(item.clone()))).style(style)
                    })
                    .collect();
                let dropdown_area = Rect {
                    x: area.x + 2,
                    y: area.y + 8,
                    width: area.width - 4,
                    height: (dropdown_items.len() as u16).min(10) + 2,
                };
                let dropdown_block = Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().bg(dark_bg))
                    .border_style(Style::default().fg(pink));
                let list = List::new(dropdown_list)
                    .block(dropdown_block)
                    .highlight_style(Style::default().bg(highlight).fg(Color::White));
                f.render_widget(list, dropdown_area);
            }
        }
    }
}

fn render_input(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    pink: Color,
    cyan: Color,
    dark_bg: Color,
) {
    let prompt = Span::styled("❯", Style::default().fg(pink).add_modifier(ratatui::style::Modifier::BOLD));
    let input = Span::raw(&state.command_input);
    
    let cursor = if area.width > (state.command_input.len() as u16 + 3) {
        let (cursor_char, cursor_style) = match state.settings.cursor_style.as_str() {
            "underline" => ("‗", Style::default().fg(cyan).add_modifier(ratatui::style::Modifier::UNDERLINED)),
            "bar" => ("│", Style::default().fg(cyan)),
            _ => ("█", Style::default().bg(cyan).fg(Color::White)), // block (default)
        };
        let visible_char = if Instant::now().elapsed().as_millis() % 500 < 250 {
            cursor_char
        } else {
            " "
        };
        Span::styled(visible_char, cursor_style)
    } else {
        Span::raw("")
    };

    let text = Line::from(vec![prompt, input, cursor]);

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(dark_bg))
        .border_style(Style::default().fg(cyan));

    f.render_widget(block, area);
    let input_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width - 2,
        height: 1,
    };
    f.render_widget(Paragraph::new(text), input_area);

    let hint = if state.dropdown_open.is_some() {
        "↑↓ Navigate | Enter Apply | Esc Cancel"
    } else {
        "Type command → Enter | Tab: Switch tabs | ↑↓: Lists | Ctrl+V: Paste"
    };
    let hint_span = Span::styled(hint, Style::default().fg(Color::Gray));
    let hint_area = Rect {
        x: area.x,
        y: area.y + 2,
        width: area.width,
        height: 1,
    };
    f.render_widget(Paragraph::new(Line::from(hint_span)), hint_area);
}

fn format_file_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

