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
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use std::io::{Error, Stdout};
use std::{io, process::Command, result::Result};

struct App {
    input: String,
    output: String,
}

impl App {
    fn new() -> App {
        App {
            input: String::new(),
            output: String::new(),
        }
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
                let input_command = self.input.trim();
                let result = Command::new("sh").arg("-c").arg(input_command).output();
                match result {
                    Ok(value) => {
                        if value.status.success() {
                            self.output = String::from_utf8_lossy(&value.stdout).to_string();
                        } else {
                            self.output = String::from_utf8_lossy(&value.stderr).to_string();
                        }
                    }
                    Err(_) => {
                        self.output = format!("Error: Command '{}' not found", input_command);
                    }
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
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(command_output, chunks[0]);

                // input area
                let input = Paragraph::new(self.input.as_str())
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL));
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
