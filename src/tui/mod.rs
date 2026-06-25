use std::{io, process};

use crossterm::event::{Event::Key, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, style::Stylize, text::Line, widgets::Widget};

pub struct App {
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            let input = match crossterm::event::read() {
                Ok(k) => k,
                Err(e) => {
                    println!("Failed to read TUI input.");
                    process::exit(1);
                }
            };

            match input {
                Key(key_event) => self.handle_key(key_event)?,
                _ => {}
            }
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key(&mut self, key_event: KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') => self.exit = false,
                _ => {}
            }
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Line::from("Silicate - Password Manager")
            .bold()
            .green()
            .render(area, buf);
    }
}

impl Default for App {
    fn default() -> App {
        App { exit: false }
    }
}
