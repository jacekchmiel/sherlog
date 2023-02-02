use std::fmt::Display;

use crossterm::event::{KeyCode, KeyModifiers};
use regex::Regex;
use tui::style::{Color, Style};
use tui::text::Spans;
use tui::widgets::{BorderType, List, ListItem, ListState, Paragraph, Wrap};

use crate::sherlog_core::{self, Sherlog, TextLine};
use crate::tui_widgets::{self, OverlayBlock};

pub struct App {
    pub core: Sherlog,

    pub status_line: StatusLine,
    pub filters: Filters,
    pub view: ViewArea,

    pub focus: Focus,

    pub wants_quit: bool,
    pub filter_is_highlight: bool,
}

impl App {
    pub fn new(core: Sherlog, filename: String) -> Self {
        let line_count = core.line_count();
        App {
            core,

            status_line: StatusLine {
                content: StatusLineContent::Status(String::from("Type `:` to start command")),
                filename,
                last_line_shown: 0,
                line_count,
            },
            filters: Filters {
                entries: vec![
                    String::from("Filter 1"),
                    String::from("Filter 2"),
                    String::from("Filter 3"),
                ],
                visible: false,
                selected: 0,
            },
            view: ViewArea {
                x: 0,
                y: 0,
                max_y: line_count - 1,
                wrap: false,
            },

            focus: Focus::View,

            wants_quit: false,
            filter_is_highlight: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    View,
    StatusLine,
    Filters,
}

pub struct Filters {
    pub entries: Vec<String>,
    pub visible: bool,
    pub selected: usize,
}

impl Filters {
    pub fn select_next(&mut self) {
        self.selected += 1;
        if self.selected > self.entries.len() - 1 {
            self.selected = self.entries.len() - 1;
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn widget(&self) -> Option<(OverlayBlock<List>, ListState)> {
        if self.visible {
            let mut list_state = ListState::default();
            list_state.select(Some(self.selected));
            let overlayed_list = OverlayBlock::new(
                //TODO: populate with real filters
                //TODO: add/edit/delete/move support
                //TODO: apply filters on close
                List::new(
                    self.entries
                        .iter()
                        .map(|s| ListItem::new(s.as_ref()))
                        .collect::<Vec<_>>(),
                )
                .highlight_style(Style::default().fg(Color::Yellow)),
            )
            .border(BorderType::Double, Style::default().fg(Color::Green));
            Some((overlayed_list, list_state))
        } else {
            None
        }
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Up => self.select_prev(),
            KeyCode::Down => self.select_next(),
            KeyCode::Esc => self.visible = false,
            _ => {}
        }
    }
}

pub struct Filter {
    pub value: Regex,
    pub negate: bool,
    pub active: bool,
}

pub enum SearchKind {
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

pub struct StatusLine {
    pub content: StatusLineContent,
    pub filename: String,
    pub last_line_shown: usize,
    pub line_count: usize,
}

impl StatusLine {
    pub fn cursor_x(&self) -> Option<u16> {
        match &self.content {
            s @ (StatusLineContent::Command(_) | StatusLineContent::SearchPattern(_, _)) => {
                Some(s.to_string().len() as u16)
            }
            StatusLineContent::Status(_) => None,
        }
    }

    pub fn print_error<S: Into<String>>(&mut self, error: S) {
        self.content = StatusLineContent::Status(error.into());
    }

    pub fn print_info<S: Into<String>>(&mut self, info: S) {
        self.content = StatusLineContent::Status(info.into());
    }

    pub fn clear(&mut self) {
        self.content = StatusLineContent::with_empty();
    }

    pub fn widget(&self, last_line_shown: Option<usize>) -> tui_widgets::StatusLine {
        tui_widgets::StatusLine::new()
            .left(self.content.to_string())
            .right_maybe(last_line_shown.map(|line| format!("{}/{}", line, self.line_count)))
            .right(self.filename.as_ref())
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        match (key.code, &mut self.content) {
            (
                KeyCode::Char(input),
                StatusLineContent::Command(s) | StatusLineContent::SearchPattern(_, s),
            ) => {
                s.push(input);
            }
            (
                KeyCode::Backspace,
                StatusLineContent::Command(s) | StatusLineContent::SearchPattern(_, s),
            ) => {
                s.pop();
            }
            (KeyCode::Esc, _) => {
                self.clear();
            }
            _ => {}
        }
    }
}

pub enum StatusLineContent {
    Command(String),
    SearchPattern(SearchKind, String), // header, pattern
    Status(String),
}

impl StatusLineContent {
    pub fn with_empty() -> Self {
        StatusLineContent::Status(String::new())
    }
}

impl Display for StatusLineContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusLineContent::Command(s) => f.write_str(s),
            StatusLineContent::SearchPattern(h, s) => write!(f, "{} /{}", h, s),
            StatusLineContent::Status(s) => f.write_str(s),
        }
    }
}

pub struct ViewArea {
    pub y: usize,
    pub x: usize,
    pub wrap: bool,
    pub max_y: usize,
}

impl ViewArea {
    pub fn scroll_up(&mut self, line_cnt: usize) {
        self.y = self.y.saturating_sub(line_cnt);
    }

    pub fn scroll_down(&mut self, line_cnt: usize) {
        self.y = self.y.saturating_add(line_cnt);
        if self.y > self.max_y {
            self.y = self.max_y;
        }
    }

    pub fn scroll_left(&mut self) {
        self.x = self.x.saturating_sub(1);
    }

    pub fn scroll_right(&mut self) {
        self.x = self.x.saturating_add(1);
    }

    pub fn go_top(&mut self) {
        self.y = 0;
    }

    pub fn go_bottom(&mut self) {
        self.y = self.max_y;
    }

    pub fn widget<'a>(&self, lines: Vec<TextLine<'a>>) -> Paragraph<'a> {
        let spans: Vec<Spans> = lines
            .into_iter()
            .map(|line| Self::make_spans(line, self.x))
            .collect();
        let mut paragraph = Paragraph::new(spans);
        if self.wrap {
            paragraph = paragraph.wrap(Wrap { trim: false })
        }
        paragraph
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
                    let remaining = s.remove_left(chars_to_remove);
                    chars_to_remove = 0;
                    Some(remaining)
                }
            })
            .map(|s| Self::make_span(s))
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

    fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Up => self.scroll_up(1),
            KeyCode::Down => self.scroll_down(1),
            KeyCode::Left => self.scroll_left(),
            KeyCode::Right => self.scroll_right(),
            KeyCode::Home => self.go_top(),
            KeyCode::End => self.go_bottom(),
            _ => {}
        }
    }
}

impl App {
    pub fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                self.wants_quit = true;
                return;
            }
            KeyCode::Char(':') if self.focus == Focus::View => {
                self.focus = Focus::StatusLine;
                self.status_line.content = StatusLineContent::Command(String::from(':'));
                return;
            }
            KeyCode::Char('f') if self.focus == Focus::View => {
                self.focus = Focus::Filters;
                self.filters.visible = true;
                return;
            }
            KeyCode::Enter if self.focus == Focus::StatusLine => match &self.status_line.content {
                StatusLineContent::Command(command) => {
                    self.process_command(&command[1..].to_owned())
                }
                StatusLineContent::SearchPattern(SearchKind::Highlight, pattern) => {
                    let pattern = pattern.clone();
                    self.highlight(&pattern);
                    self.focus = Focus::View;
                }
                StatusLineContent::SearchPattern(SearchKind::Filter, pattern) => {
                    let pattern = pattern.clone();
                    self.filter(&pattern);
                    self.focus = Focus::View;
                }
                StatusLineContent::Status(_) => {}
            },
            _ => {}
        }

        match self.focus {
            Focus::View => self.view.on_key(key),
            Focus::StatusLine => self.status_line.on_key(key),
            Focus::Filters => self.filters.on_key(key),
        }

        if key.code == KeyCode::Esc {
            self.focus = Focus::View
        }
    }

    fn highlight(&mut self, pattern: &str) {
        if pattern.trim().is_empty() {
            self.core.highlight = None;
            self.status_line.clear();
        } else {
            match Regex::new(pattern) {
                Ok(re) => {
                    self.core.highlight = Some(re);
                    self.status_line.clear();
                }
                Err(e) => self
                    .status_line
                    .print_error(format!("Invalid pattern: {}", e)),
            }
        }
    }

    fn filter(&mut self, pattern: &str) {
        if pattern.trim().is_empty() {
            self.core.filter = None;
            if self.filter_is_highlight {
                self.filter_is_highlight = false;
                self.core.highlight = None;
            }
            self.status_line.clear();
        } else {
            match Regex::new(pattern) {
                Ok(re) => {
                    if self.core.highlight.is_none() {
                        self.core.highlight = Some(re.clone());
                        self.filter_is_highlight = true;
                    }
                    self.core.filter = Some(re);
                    self.status_line.clear();
                }
                Err(e) => self
                    .status_line
                    .print_error(format!("Invalid pattern: {}", e)),
            }
        }
    }

    pub fn process_command(&mut self, command: &str) {
        let words: Vec<_> = command.split_whitespace().collect();
        let mut err: Option<String> = None;
        match &words[..] {
            &["q" | "quit"] => self.wants_quit = true,
            &["h" | "highlight", ref rest @ ..] => {
                let value: String = rest.iter().copied().collect();
                self.status_line.content =
                    StatusLineContent::SearchPattern(SearchKind::Highlight, value)
            }
            &["f" | "filter", ref rest @ ..] => {
                let value: String = rest.iter().copied().collect();
                self.status_line.content =
                    StatusLineContent::SearchPattern(SearchKind::Filter, value)
            }
            &["w" | "wrap"] => {
                if self.view.wrap {
                    self.status_line.print_info("word wrap off");
                    self.view.wrap = false;
                } else {
                    self.status_line.print_info("word wrap on");
                    self.view.wrap = true;
                }
            }
            _ => err = Some(format!("Unknown command: {}", command)),
        }
        if let Some(msg) = err {
            self.status_line.print_error(msg);
        }
    }
}
