use std::{io, process};

use crossterm::event::{Event::Key, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Modifier, Style, Stylize},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget,
    },
};

pub struct App {
    exit: bool,
    entries: Vec<String>,
    state: ListState,
}

impl App {
    pub fn new(entries: Vec<String>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        App {
            exit: false,
            entries,
            state,
        }
    }

    pub fn next(&mut self) {
        let selected = self.state.selected().unwrap_or(0);

        let next = if selected >= self.entries.len() - 1 {
            0
        } else {
            selected + 1
        };

        self.state.select(Some(next));
    }

    pub fn previous(&mut self) {
        let selected = self.state.selected().unwrap_or(0);

        let previous = if selected == 0 {
            self.entries.len() - 1
        } else {
            selected - 1
        };

        self.state.select(Some(previous));
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            let input = match crossterm::event::read() {
                Ok(k) => k,
                Err(e) => {
                    let msg = format!("Failed to read TUI input: {e}").red();
                    println!("{msg}");
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

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let horizontal_area =
            Layout::horizontal([Constraint::Percentage(10), Constraint::Percentage(90)])
                .areas(area);

        let [list_area, view_area] = horizontal_area;

        let items = self.entries.iter().map(|item| ListItem::new(item.clone())); // Ensure items map cleanly
        let list =
            List::new(items).block(Block::default().title("Passwords").borders(Borders::ALL));

        frame.render_stateful_widget(list, list_area, &mut self.state);
    }

    fn handle_key(&mut self, key_event: KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') => self.exit = true,
                _ => {}
            }
        }

        Ok(())
    }
}
