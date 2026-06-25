use std::{fs, io, process};

use crossterm::event::{Event::Key, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, Wrap},
};
use silicate::SilicateError;

pub struct App {
    exit: bool,
    entries: Vec<String>,
    state: ListState,
    key: [u8; 32],
    search_query: String,
    is_searching: bool,
}

impl App {
    pub fn new(entries: Vec<String>, key: [u8; 32]) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        App {
            exit: false,
            entries,
            state,
            key,
            search_query: String::new(),
            is_searching: false,
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

        let main_layout = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(if self.is_searching { 3 } else { 0 }),
        ])
        .split(area);

        let main_area = main_layout[0];
        let search_area = main_layout[1];

        let horizontal_area =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
                .areas(main_area);

        let [list_area, view_area] = horizontal_area;

        let filtered = self.filtered_entries();
        let items = filtered.iter().map(|item| ListItem::new((*item).clone()));

        let list = List::new(items)
            .block(Block::default().title("Passwords").borders(Borders::ALL))
            .highlight_symbol("> ")
            .highlight_style(
                Style::default()
                    .fg(ratatui::style::Color::Green)
                    .add_modifier(Modifier::REVERSED),
            )
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(list, list_area, &mut self.state);

        // 3. Determine the decrypted content for the right pane
        let details_content = if let Some(selected_idx) = self.state.selected() {
            if let Some(entry_name) = self.entries.get(selected_idx) {
                match self.get_decrypted_password(entry_name) {
                    Ok(password) => vec![
                        Line::from(vec![
                            Span::raw("Account/Site: "),
                            Span::styled(
                                entry_name,
                                Style::default()
                                    .fg(ratatui::style::Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]),
                        Line::from(""),
                        Line::from(vec![
                            Span::raw("Password:     "),
                            Span::styled(
                                password,
                                Style::default().fg(ratatui::style::Color::Green),
                            ),
                        ]),
                    ],
                    Err(e) => vec![Line::from(Span::styled(
                        format!("Decryption Error: {}", e),
                        Style::default().fg(ratatui::style::Color::Red),
                    ))],
                }
            } else {
                vec![Line::from("No entry found.")]
            }
        } else {
            vec![Line::from("Select an entry to view details.")]
        };

        let details_block = Paragraph::new(details_content)
            .block(Block::default().title("Details").borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        frame.render_widget(details_block, view_area);
    }

    /// Helper method to fetch and decrypt the file for the TUI view pane
    fn get_decrypted_password(&self, entry_name: &str) -> Result<String, SilicateError> {
        // Construct the expected file path inside ~/.silicate/
        let home = std::env::var("HOME").unwrap();
        let full_path = format!("{}/.silicate/{}.bin", home, entry_name);

        // Read the encrypted file binary chunk
        let data = fs::read(full_path)?;
        if data.len() < 12 {
            return Err(SilicateError::Plain(
                "Invalid password file format (too short)",
            ));
        }

        let (nonce_bytes, cipher_bytes) = data.split_at(12);

        let decrypted =
            silicate::decrypt_passwd(&self.key, cipher_bytes.to_vec(), nonce_bytes.try_into()?)?;

        Ok(decrypted)
    }

    /// Helper to get entries filtered by the current search query
    fn filtered_entries(&self) -> Vec<&String> {
        if self.search_query.is_empty() {
            self.entries.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.entries
                .iter()
                .filter(|entry| entry.to_lowercase().contains(&query))
                .collect()
        }
    }

    fn handle_key(&mut self, key_event: KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                _ => {}
            }
        }

        Ok(())
    }
}
