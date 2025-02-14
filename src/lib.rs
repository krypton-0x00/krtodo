use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use csv::{Reader, Writer};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Todo {
    pub id: u32,
    pub title: String,
    pub is_completed: bool,
}

pub struct App {
    pub items: Vec<Todo>,
    pub db: String,
    pub state: ListState,
    pub input_mode: bool,
    pub input_text: String,
}

impl App {
    pub fn new(db_path: &str) -> Self {
        let items = Self::read_db(db_path);
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            items,
            db: db_path.to_string(),
            state,
            input_mode: false,
            input_text: String::new(),
        }
    }

    fn read_db(db_path: &str) -> Vec<Todo> {
        let mut todos = Vec::new();
        if Path::new(db_path).exists() {
            let mut rdr = Reader::from_path(db_path).expect("Failed to open DB");
            for result in rdr.deserialize() {
                if let Ok(todo) = result {
                    todos.push(todo);
                }
            }
        }
        todos
    }

    pub fn toggle_complete(&mut self) {
        if let Some(selected) = self.state.selected() {
            if let Some(todo) = self.items.get_mut(selected) {
                todo.is_completed = !todo.is_completed;
                self.write_db();
            }
        }
    }

    pub fn add_task(&mut self) {
        if !self.input_text.is_empty() {
            let new_id = (self.items.len() as u32) + 1;
            let new_todo = Todo {
                id: new_id,
                title: self.input_text.clone(),
                is_completed: false,
            };
            self.items.push(new_todo);
            self.write_db();
            self.input_text.clear();
        }
        self.input_mode = false;
    }

    pub fn delete_task(&mut self) {
        if let Some(selected) = self.state.selected() {
            if selected < self.items.len() {
                self.items.remove(selected);
                self.write_db();
                let new_index = if selected >= self.items.len() {
                    0
                } else {
                    selected
                };
                self.state.select(Some(new_index));
            }
        }
    }

    fn write_db(&self) {
        let mut wtr = Writer::from_path(&self.db).expect("Failed to write DB");
        wtr.write_record(&["id", "title", "is_completed"]).unwrap();
        for todo in &self.items {
            wtr.write_record(&[
                todo.id.to_string(),
                todo.title.clone(),
                todo.is_completed.to_string(),
            ])
            .unwrap();
        }
        wtr.flush().unwrap();
    }

    pub fn next(&mut self) {
        if let Some(selected) = self.state.selected() {
            let next = if selected >= self.items.len() - 1 {
                0
            } else {
                selected + 1
            };
            self.state.select(Some(next));
        }
    }

    pub fn previous(&mut self) {
        if let Some(selected) = self.state.selected() {
            let prev = if selected == 0 {
                self.items.len() - 1
            } else {
                selected - 1
            };
            self.state.select(Some(prev));
        }
    }
}

fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(3),
                    if app.input_mode {
                        Constraint::Length(3)
                    } else {
                        Constraint::Length(0)
                    },
                ])
                .split(size);

            let block = Block::default().title(" TODO List ").borders(Borders::ALL);
            let items: Vec<ListItem> = app
                .items
                .iter()
                .map(|todo| {
                    let status = if todo.is_completed { "[✔] " } else { "[ ] " };
                    ListItem::new(format!("{status}{}", todo.title)).style(if todo.is_completed {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::CROSSED_OUT)
                    } else {
                        Style::default()
                    })
                })
                .collect();

            let list = List::new(items).block(block).highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

            f.render_stateful_widget(list, layout[0], &mut app.state);

            let help_text = Paragraph::new(
                "↑ ↓ OR j k: Move | SpaceBar: Toggle | a: Add | d: Delete | q: Quit",
            )
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::TOP))
            .wrap(Wrap { trim: false });

            f.render_widget(help_text, layout[1]);

            if app.input_mode {
                let input_block = Block::default().title(" Add Task ").borders(Borders::ALL);
                let input_text = Paragraph::new(app.input_text.clone()).block(input_block);
                f.render_widget(input_text, layout[2]);
            }
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let event::Event::Key(key) = event::read()? {
                if app.input_mode {
                    match key.code {
                        KeyCode::Enter => app.add_task(),
                        KeyCode::Esc => {
                            app.input_text.clear();
                            app.input_mode = false;
                        }
                        KeyCode::Backspace => {
                            app.input_text.pop();
                        }
                        KeyCode::Char(c) => {
                            app.input_text.push(c);
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Enter | KeyCode::Char(' ') => app.toggle_complete(),
                        KeyCode::Char('a') => app.input_mode = true,
                        KeyCode::Char('d') => app.delete_task(),
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn start_tui(db_path: &str) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(db_path);
    let result = run(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
