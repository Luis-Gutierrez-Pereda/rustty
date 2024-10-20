use anyhow::{Result, Context};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::{io, time::{Duration, Instant}};
use sysinfo::{System};

struct App {
    input: String,
    processes: Vec<ProcessInfo>,
    show_processes: bool,
    last_update: Instant,
}

struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory: u64,
}

impl App {
    fn new() -> App {
        App {
            input: String::new(),
            processes: Vec::new(),
            show_processes: false,
            last_update: Instant::now(),
        }
    }

    fn update_processes(&mut self, sys: &mut System) {
        if self.last_update.elapsed() >= Duration::from_secs(1) {
            sys.refresh_all();

            self.processes = sys
                .processes()
                .values()
                .map(|proc| ProcessInfo {
                    pid: proc.pid().as_u32(),
                    name: proc.name().to_str().unwrap().to_string(),
                    cpu_usage: proc.cpu_usage(),
                    memory: proc.memory(),
                })
                .collect();

            self.processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
            self.last_update = Instant::now();
        }
    }

    fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                if self.input.trim() == "ps" {
                    self.show_processes = true;
                } else if self.input.trim() == "clear" {
                    self.show_processes = false;
                }
                self.input.clear();
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    // Terminal setup
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    let mut sys = System::new_all();

    // Main loop
    loop {
        // Update process list if needed and if processes should be shown
        if app.show_processes {
            app.update_processes(&mut sys);
        }

        // Draw UI
        terminal.draw(|f| {
            // Create main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Min(3),     // Process display area
                    Constraint::Length(3),  // Input area
                ])
                .split(f.area());

            // Draw process area
            if app.show_processes {
                let items: Vec<ListItem> = app
                    .processes
                    .iter()
                    .map(|p| {
                        ListItem::new(Line::from(vec![
                            Span::raw(format!("{:<8}", p.pid)),
                            Span::raw(format!("{:<20}", p.name)),
                            Span::raw(format!("{:>6.1}%", p.cpu_usage)),
                            Span::raw(format!("{:>10}KB", p.memory)),
                        ]))
                    })
                    .collect();

                let processes = List::new(items)
                    .block(Block::default()
                        .title("Processes (sorted by CPU usage)")
                        .borders(Borders::ALL))
                    .style(Style::default().fg(Color::White));

                f.render_widget(processes, chunks[0]);
            } else {
                let help = Paragraph::new("Type 'ps' to show processes, 'clear' to hide them")
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(help, chunks[0]);
            }

            // Draw input area
            let input = Paragraph::new(app.input.as_str())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Command"));
            f.render_widget(input, chunks[1]);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c') if key.modifiers == event::KeyModifiers::CONTROL => {
                        break;
                    }
                    code => app.handle_input(code),
                }
            }
        }
    }

    // Cleanup and restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}