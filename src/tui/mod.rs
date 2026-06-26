use std::{fs, io, process};

use crossterm::event::{Event::Key, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, HighlightSpacing, List, ListItem, ListState, Paragraph, Wrap,
    },
};
use silicate::SilicateError;

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchTarget {
    Name,
    Tag,
}

pub struct App {
    exit: bool,
    entries: Vec<String>,
    state: ListState,
    key: [u8; 32],
    search_query: String,
    is_searching: bool,
    search_target: SearchTarget,
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
            search_target: SearchTarget::Name,
        }
    }

    pub fn next(&mut self) {
        let selected = self.state.selected().unwrap_or(0);

        let next = if selected >= self.filtered_entries().len() - 1 {
            0
        } else {
            selected + 1
        };

        self.state.select(Some(next));
    }

    pub fn previous(&mut self) {
        let selected = self.state.selected().unwrap_or(0);

        let previous = if selected == 0 {
            self.filtered_entries().len() - 1
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

            //match input {
            //    Key(key_event) => self.handle_key(key_event)?,
            //    _ => {}
            //}

            if let Key(key_event) = input {
                self.handle_key(key_event)?;
            }
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let horizontal_area =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
                .areas(area);

        let [list_area, view_area] = horizontal_area;

        let filtered = self.filtered_entries();

        let items: Vec<ListItem> = filtered
            .iter()
            .map(|item_str| {
                if let Some((name, tag)) = item_str.split_once('-') {
                    let name_span = ratatui::text::Span::raw(format!("{} ", name.trim_end()));
                    let tag_span = ratatui::text::Span::raw(format!("({})", tag.trim())).dim();
                    let line = ratatui::text::Line::from(vec![name_span, tag_span]);
                    ListItem::new(line)
                } else {
                    // item_str is now &String, so we can clone it directly without dereferencing
                    ListItem::new(item_str.clone())
                }
            })
            .collect();

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

        let details_content = if let Some(selected_idx) = self.state.selected() {
            if let Some(entry_name) = filtered.get(selected_idx) {
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

        if self.is_searching {
            let popup = centered_rect(50, 5, area);

            frame.render_widget(Clear, popup);

            // Construct the visual toggle selectors with custom highlights
            let name_style = if self.search_target == SearchTarget::Name {
                Style::default()
                    .fg(ratatui::style::Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(ratatui::style::Color::DarkGray)
            };

            let tag_style = if self.search_target == SearchTarget::Tag {
                Style::default()
                    .fg(ratatui::style::Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(ratatui::style::Color::DarkGray)
            };

            let name_icon = if self.search_target == SearchTarget::Name {
                "[● Name]  "
            } else {
                "[○ Name]  "
            };
            let tag_icon = if self.search_target == SearchTarget::Tag {
                "[● Tag]"
            } else {
                "[○ Tag]"
            };

            let popup_content = vec![
                Line::from(self.search_query.as_str()),
                Line::from("─".repeat((popup.width as usize).saturating_sub(2))).dim(),
                Line::from(vec![
                    Span::styled(name_icon, name_style),
                    Span::styled(tag_icon, tag_style),
                ]),
            ];

            let input = Paragraph::new(popup_content)
                .block(Block::default().title(" Search ").borders(Borders::ALL))
                .alignment(Alignment::Left);

            frame.render_widget(&input, popup);

            // Pin the cursor strictly to the query string line (top row of popup payload area)
            frame.set_cursor_position((popup.x + 1 + self.search_query.len() as u16, popup.y + 1));

            frame.render_widget(input, popup);

            // Cursor goes inside the box
            frame.set_cursor_position((popup.x + 1 + self.search_query.len() as u16, popup.y + 1));
        }
    }

    /// Helper method to toggle between name and tag
    fn toggle_search(&mut self) {
        match self.search_target {
            SearchTarget::Name => self.search_target = SearchTarget::Tag,
            SearchTarget::Tag => self.search_target = SearchTarget::Name,
        }
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
    fn filtered_entries(&self) -> Vec<String> {
        if self.search_query.is_empty() {
            self.entries.to_vec()
        } else {
            let query = self.search_query.to_lowercase();
            self.entries
                .iter()
                .filter(|entry| entry.to_lowercase().contains(&query))
                .cloned()
                .collect()
        }
    }

    fn handle_key(&mut self, key_event: KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            if self.is_searching {
                match key_event.code {
                    KeyCode::Esc => self.is_searching = false,
                    KeyCode::Enter => {
                        self.is_searching = false;
                    }
                    KeyCode::Tab => {
                        self.toggle_search();
                        self.state.select(Some(0));
                    }
                    KeyCode::Backspace => {
                        self.search_query.pop();
                        self.state.select(Some(0));
                    }
                    KeyCode::Char(c) => {
                        self.search_query.push(c);
                        self.state.select(Some(0));
                    }
                    _ => {}
                }
            } else {
                match key_event.code {
                    KeyCode::Char('q') => self.exit = true,
                    KeyCode::Char('/') => self.is_searching = true,
                    KeyCode::Up => self.previous(),
                    KeyCode::Down => self.next(),
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
