use std::fmt::Display;

use crossterm::event::KeyCode;

use crate::tui_widgets;

pub struct StatusLine {
    pub content: StatusLineContent,
    pub filename: String,
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

    pub fn on_key(&mut self, key: crossterm::event::KeyEvent) {
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
