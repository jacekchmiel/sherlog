mod sherlog_core;
mod test_data;

use std::collections::HashMap;
use std::fmt::Display;

use regex::Regex;
use sherlog_core::{FindRange, Sherlog};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::{execute, terminal, Result};
use tui::backend::{Backend, CrosstermBackend};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::{layout, widgets, Frame, Terminal};

struct App {
    core: Sherlog,
    view_offset: usize,
    status: StatusLine,
    wants_quit: bool,
    view_height: usize,
    active_highlight_pattern: Option<String>,
}

enum StatusLine {
    Command(String),
    SearchPattern(String),
    Status(String),
}

impl StatusLine {
    pub fn with_empty() -> Self {
        StatusLine::Status(String::new())
    }
}

impl Display for StatusLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            StatusLine::Command(s) => s,
            StatusLine::SearchPattern(s) => s,
            StatusLine::Status(s) => s,
        };
        f.write_str(text)
    }
}

impl App {
    pub fn new(core: Sherlog, terminal_height: usize) -> Self {
        App {
            core,
            view_offset: 0,
            status: StatusLine::Status(String::from("Type `:` to start command")),
            wants_quit: false,
            view_height: terminal_height - 1,
            active_highlight_pattern: None,
        }
    }

    pub fn scroll_up(&mut self) {
        self.view_offset = self.view_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.view_offset = self.view_offset.saturating_add(1);
        let max_offset = self.core.line_count() - 1;
        if self.view_offset > max_offset {
            self.view_offset = max_offset;
        }
    }

    pub fn on_user_input(&mut self, input: char) {
        match &mut self.status {
            StatusLine::Command(c) => c.push(input),
            StatusLine::SearchPattern(p) => p.push(input),
            StatusLine::Status(_) => match input {
                ':' => self.status = StatusLine::Command(String::from(input)),
                '/' => self.status = StatusLine::SearchPattern(String::from(input)),
                _ => {}
            },
        };
    }

    pub fn on_backspace(&mut self) {
        match &mut self.status {
            StatusLine::Command(text) | StatusLine::SearchPattern(text) => {
                text.pop();
            }
            StatusLine::Status(_) => {
                self.status = StatusLine::with_empty();
            }
        }
    }

    pub fn on_enter(&mut self) {
        match &self.status {
            StatusLine::Command(command) => self.process_command(&command[1..].to_owned()),
            StatusLine::SearchPattern(pattern) => {
                self.register_search_pattern(pattern[1..].to_owned());
                self.clear_status();
            }
            StatusLine::Status(_) => self.clear_status(),
        }
    }

    pub fn on_esc(&mut self) {
        self.clear_status()
    }

    pub fn process_command(&mut self, command: &str) {
        let mut err: Option<String> = None;
        match command {
            "q" | "quit" => self.wants_quit = true,
            other => err = Some(format!("Unknown command: {}", other)),
        }
        if let Some(msg) = err {
            self.print_error(msg);
        } else {
            self.clear_status()
        }
    }

    pub fn register_search_pattern(&mut self, pattern: String) {
        self.active_highlight_pattern = Some(pattern)
    }

    pub fn process_search(&mut self) -> HashMap<usize, Vec<FindRange>> {
        match &self.active_highlight_pattern {
            Some(pattern) => match Regex::new(&pattern) {
                Ok(pattern) => self
                    .core
                    .find(&pattern, self.view_offset, Some(self.view_height)),
                Err(e) => {
                    self.print_error(format!("Invalid pattern: {}", e)); // FIXME: handle multiline errors properly
                    self.active_highlight_pattern = None;
                    HashMap::new()
                }
            },
            None => HashMap::new(),
        }
    }

    pub fn print_error(&mut self, error: String) {
        self.status = StatusLine::Status(error);
    }

    pub fn clear_status(&mut self) {
        self.status = StatusLine::with_empty();
    }
}

fn handle_event(app: &mut App, event: Event) {
    if let Event::Key(key) = event {
        if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
            app.wants_quit = true;
            return;
        }

        match key.code {
            KeyCode::Up => app.scroll_up(),
            KeyCode::Down => app.scroll_down(),
            KeyCode::Char(c) => app.on_user_input(c),
            KeyCode::Backspace => app.on_backspace(),
            KeyCode::Enter => app.on_enter(),
            KeyCode::Esc => app.on_esc(),
            _ => {}
        }
    }
}

fn render_ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks for text view and command bar
    let chunks = layout::Layout::default()
        .direction(layout::Direction::Vertical)
        .constraints([layout::Constraint::Min(1), layout::Constraint::Length(1)].as_ref())
        .split(f.size());

    // FIXME: highlights should not be triggered by / (this should be search)
    let highlights = app.process_search();

    let lines = app
        .core
        .get_lines(app.view_offset, Some(chunks[0].height as usize));
    let spans: Vec<Spans<'_>> = lines
        .iter()
        .zip(app.view_offset..app.view_offset + app.view_height)
        .map(|(text, line_num)| Spans::from(highlight(&text, highlights.get(&line_num))))
        .collect();
    let paragraph = widgets::Paragraph::new(spans);
    f.render_widget(paragraph, chunks[0]);

    let command_line = widgets::Paragraph::new(app.status.to_string());
    f.render_widget(command_line, chunks[1]);
}

fn highlight<'a>(text: &'a str, highlights: Option<&Vec<FindRange>>) -> Vec<Span<'a>> {
    if let Some(ranges) = highlights {
        let mut v = Vec::new();
        let mut pos = 0;
        for range in ranges.iter() {
            v.push(Span::raw(&text[pos..range.start]));
            v.push(Span::styled(
                &text[range.start..range.end],
                Style::default().fg(Color::Red),
            ));
            pos = range.end;
        }
        v.push(Span::raw(&text[pos..]));
        v
    } else {
        vec![Span::raw(text)]
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| render_ui(f, &mut app))?;
        handle_event(&mut app, event::read()?);

        if app.wants_quit {
            return Ok(());
        }
    }
}

fn main() -> Result<()> {
    // setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let height = terminal.size()?.height as usize;
    let app = App::new(Sherlog::new(&test_data::SAMPLE_LOG), height);
    let res = run_app(&mut terminal, app);

    // restore terminal
    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
