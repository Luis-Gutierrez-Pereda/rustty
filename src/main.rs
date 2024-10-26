// std
use std::io::{Error, Stdout};
use std::path::{Path, PathBuf};
use std::{collections::HashSet, env, io, process::Command, process::Stdio, result::Result};
// dirs
use dirs::home_dir;

// cross-platform backend
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// frontend
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

struct App {
    input: String,
    output: String,
    fullscreen_commands: HashSet<&'static str>,
}

impl App {
    fn new() -> App {
        App {
            input: String::new(),
            output: String::new(),
            fullscreen_commands: ["htop", "vim", "less", "top"].iter().cloned().collect(),
        }
    }

    /// Returns the current directory
    fn current_dir() -> PathBuf {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        if let Some(home) = home_dir() {
            if current_dir.starts_with(&home) {
                return Path::new("~").join(current_dir.strip_prefix(&home).unwrap());
            }
        }
        current_dir
    }

    /// In charge of running commands that do not involve a full screen
    fn run_cmd(&mut self) {
        let cmd = self.input.trim();
        match Command::new("sh").arg("-c").arg(cmd).output() {
            Ok(value) => {
                if value.status.success() {
                    self.output = String::from_utf8_lossy(&value.stdout).to_string();
                } else {
                    self.output = String::from_utf8_lossy(&value.stderr).to_string();
                }
            }
            Err(_) => {
                self.output = format!("Error: Command '{}' not found", cmd);
            }
        }
    }

    /// Runs fullscreen commands
    fn run_fullscreen_cmd(&mut self) {
        let cmd = self.input.trim();

        disable_raw_mode().expect("Failed to disable raw mode");
        execute!(io::stdout(), LeaveAlternateScreen).expect("Failed to leave alternate screen");

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .spawn()
            .expect("Failed to spawn command");

        let _ = child.wait();

        enable_raw_mode().expect("Failed to enable raw mode");
        execute!(io::stdout(), EnterAlternateScreen).expect("Failed to enter alternate screen");
    }

    /// Reads input commands and modifies the output accordingly
    fn read(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                let cmd = self.input.trim().to_string();
                if cmd.starts_with("cd ") {
                    match env::set_current_dir(cmd[3..].trim()) {
                        Ok(_) => {}
                        Err(e) => self.output = format!("Error changing directory: {}", e),
                    }
                } else if cmd == "clear" {
                    self.output.clear();
                } else if self.fullscreen_commands.contains(cmd.as_str()) {
                    self.run_fullscreen_cmd();
                } else {
                    self.run_cmd();
                }
                self.input.clear();
            }
            _ => {}
        }
    }

    /// Renders the front end of the terminal app
    fn render_ui(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
        terminal
            .draw(|f| {
                // layout
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Min(3),    // Process or Command output display area
                        Constraint::Length(3), // Input area
                    ])
                    .split(f.area());

                // output area
                let command_output = Paragraph::new(self.output.as_str())
                    .style(Style::default().fg(Color::White))
                    .block(Block::default().borders(Borders::ALL).title("Output"));
                f.render_widget(command_output, chunks[0]);

                // input area
                let prompt = format!("{} > {}", App::current_dir().display(), self.input);
                let input = Paragraph::new(Line::from(Span::styled(
                    prompt,
                    Style::default().fg(Color::Blue),
                )))
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL).title("Input"));
                f.render_widget(input, chunks[1]);
            })
            .unwrap();
    }
}

fn main() -> Result<(), Error> {
    // setup
    enable_raw_mode().expect("Failed to enable raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    // main loop
    loop {
        // render
        app.render_ui(&mut terminal);

        // react to input keystrokes
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                // exit the terminal with ctrl + d
                KeyCode::Char('c') if key_event.modifiers == event::KeyModifiers::CONTROL => {
                    break;
                }
                key => app.read(key),
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
