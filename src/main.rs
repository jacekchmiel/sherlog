mod sherlog_core;

use std::fmt::Display;

use regex::Regex;
use sherlog_core::{Sherlog, TextLine};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::{execute, terminal, Result};
use tui::backend::{Backend, CrosstermBackend};
use tui::style::{Color, Style};
use tui::text::Spans;
use tui::{layout, widgets, Frame, Terminal};

struct App {
    core: Sherlog,
    view_offset_y: usize,
    view_offset_x: usize,
    status: StatusLine,
    wants_quit: bool,
    filter_is_highlight: bool,
}

enum SearchKind {
    Highlight,
    Filter,
}

impl Display for SearchKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SearchKind::Highlight => "highlight",
            SearchKind::Filter => "filter",
        })
    }
}

enum StatusLine {
    Command(String),
    SearchPattern(SearchKind, String), // header, pattern
    Status(String),
}

impl StatusLine {
    pub fn with_empty() -> Self {
        StatusLine::Status(String::new())
    }
}

impl Display for StatusLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusLine::Command(s) => f.write_str(s),
            StatusLine::SearchPattern(h, s) => write!(f, "{} /{}", h, s),
            StatusLine::Status(s) => f.write_str(s),
        }
    }
}

impl App {
    pub fn new(core: Sherlog) -> Self {
        App {
            core,
            view_offset_y: 0,
            view_offset_x: 0,
            status: StatusLine::Status(String::from("Type `:` to start command")),
            wants_quit: false,
            filter_is_highlight: false,
        }
    }

    pub fn scroll_up(&mut self) {
        self.view_offset_y = self.view_offset_y.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.view_offset_y = self.view_offset_y.saturating_add(1);
        let max_offset = self.core.line_count() - 1;
        if self.view_offset_y > max_offset {
            self.view_offset_y = max_offset;
        }
    }

    pub fn scroll_left(&mut self) {
        self.view_offset_x = self.view_offset_x.saturating_sub(1);
    }

    pub fn scroll_right(&mut self) {
        self.view_offset_x = self.view_offset_x.saturating_add(1);
    }

    pub fn on_user_input(&mut self, input: char) {
        match &mut self.status {
            StatusLine::Command(c) => c.push(input),
            StatusLine::SearchPattern(_, p) => p.push(input),
            StatusLine::Status(_) => match input {
                ':' => self.status = StatusLine::Command(String::from(input)),
                _ => {}
            },
        };
    }

    pub fn on_backspace(&mut self) {
        match &mut self.status {
            StatusLine::Command(text) | StatusLine::SearchPattern(_, text) => {
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
            StatusLine::SearchPattern(SearchKind::Highlight, pattern) => {
                if pattern.trim().is_empty() {
                    self.core.highlight = None;
                    self.clear_status();
                } else {
                    match Regex::new(pattern) {
                        Ok(re) => {
                            self.core.highlight = Some(re);
                            self.clear_status();
                        }
                        Err(e) => self.print_error(format!("Invalid pattern: {}", e)),
                    }
                }
            }
            StatusLine::SearchPattern(SearchKind::Filter, pattern) => {
                if pattern.trim().is_empty() {
                    self.core.filter = None;
                    if self.filter_is_highlight {
                        self.filter_is_highlight = false;
                        self.core.highlight = None;
                    }
                    self.clear_status();
                } else {
                    match Regex::new(pattern) {
                        Ok(re) => {
                            if self.core.highlight.is_none() {
                                self.core.highlight = Some(re.clone());
                                self.filter_is_highlight = true;
                            }
                            self.core.filter = Some(re);
                            self.clear_status();
                        }
                        Err(e) => self.print_error(format!("Invalid pattern: {}", e)),
                    }
                }
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
            "h" | "highlight" => {
                self.status = StatusLine::SearchPattern(SearchKind::Highlight, String::new())
            }
            "f" | "filter" => {
                self.status = StatusLine::SearchPattern(SearchKind::Filter, String::new())
            }
            other => err = Some(format!("Unknown command: {}", other)),
        }
        if let Some(msg) = err {
            self.print_error(msg);
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
            KeyCode::Left => app.scroll_left(),
            KeyCode::Right => app.scroll_right(),
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

    let lines = app
        .core
        .get_lines(app.view_offset_y, Some(chunks[0].height as usize));
    let spans: Vec<Spans> = lines
        .into_iter()
        .map(|line| make_spans(line, app.view_offset_x))
        .collect();
    let paragraph = widgets::Paragraph::new(spans);
    f.render_widget(paragraph, chunks[0]);

    let command_line = widgets::Paragraph::new(app.status.to_string());
    f.render_widget(command_line, chunks[1]);
}

fn make_spans<'a>(line: TextLine<'a>, offset: usize) -> tui::text::Spans {
    let mut chars_to_remove = offset;
    let spans = line.spans.into_iter();
    spans
        .filter_map(|s| {
            if chars_to_remove >= s.content.len() {
                chars_to_remove -= s.content.len();
                None
            } else {
                Some(s.remove_left(chars_to_remove))
            }
        })
        .map(|s| make_span(s))
        .collect::<Vec<_>>()
        .into()
}

fn make_span<'a>(span: sherlog_core::Span<'a>) -> tui::text::Span<'a> {
    match span.kind {
        sherlog_core::SpanKind::Raw => tui::text::Span::raw(span.content),
        sherlog_core::SpanKind::Highlight => {
            tui::text::Span::styled(span.content, Style::default().fg(Color::Red))
        }
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

/// Log investigator. Helps analyzing textual log files with rich (not yet) filtering options, search (not yet) and
/// storing investigation session ready to resume after having a cup of coffee (also not yet implemented).
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[arg(value_name = "LOG_FILE")]
    input: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let log_data = std::fs::read_to_string(args.input)?;

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
    let app = App::new(Sherlog::new(&log_data));
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
