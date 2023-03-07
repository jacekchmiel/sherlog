use std::fmt::Display;

use crossterm::event::KeyCode;

use crate::ty::{React, Render};
use crate::widgets;

pub(crate) struct StatusLine {
    pub content: StatusLineContent,
    pub filename: String,
    pub line_count: usize,
    pub line_shown: Option<usize>,
}

impl StatusLine {
    pub fn print_error<S: Into<String>>(&mut self, error: S) {
        self.content = StatusLineContent::Status(error.into());
    }

    pub fn print_info<S: Into<String>>(&mut self, info: S) {
        self.content = StatusLineContent::Status(info.into());
    }

    pub fn clear(&mut self) {
        self.content = StatusLineContent::with_empty();
    }

    pub fn enter_command_mode(&mut self) {
        self.content = StatusLineContent::Command(String::new());
    }

    pub fn enter_highlight_pattern_mode(&mut self, value: String) {
        self.content = StatusLineContent::SearchPattern(SearchKind::Highlight, value);
    }

    pub fn enter_search_mode(&mut self, value: String) {
        self.content = StatusLineContent::SearchPattern(SearchKind::Search, value);
    }
}

impl Render for StatusLine {
    type Widget<'a> = widgets::StatusLine<'a>;

    fn widget(&self) -> Self::Widget<'_> {
        widgets::StatusLine::new()
            .left_maybe(self.content.header())
            .left_maybe(self.content.editable())
            .cursor_maybe(self.content.editable().is_some())
            .right_maybe(
                self.line_shown
                    .map(|line| format!("{}/{}", line, self.line_count)),
            )
            .right(self.filename.as_ref())
    }
}

impl<'a> React<'a> for StatusLine {
    type Reaction = Option<StatusLineReaction>;
    fn on_key(&'a mut self, key: crossterm::event::KeyEvent) -> Self::Reaction {
        match key.code {
            KeyCode::Char(input) => {
                match &mut self.content {
                    StatusLineContent::Command(s) | StatusLineContent::SearchPattern(_, s) => {
                        s.push(input);
                    }
                    _ => {}
                };
                None
            }
            KeyCode::Backspace => {
                match &mut self.content {
                    StatusLineContent::Command(s) | StatusLineContent::SearchPattern(_, s) => {
                        s.pop();
                    }
                    _ => {}
                }
                None
            }
            KeyCode::Esc => {
                self.clear();
                Some(StatusLineReaction::Defocus)
            }
            KeyCode::Enter => match &self.content {
                StatusLineContent::Command(s) => {
                    Some(StatusLineReaction::ExecuteCommand(s.clone()))
                }
                StatusLineContent::SearchPattern(SearchKind::Highlight, s) => {
                    Some(StatusLineReaction::Highlight(s.clone()))
                }
                StatusLineContent::SearchPattern(SearchKind::Search, s) => {
                    Some(StatusLineReaction::Search(s.clone()))
                }
                _ => Some(StatusLineReaction::Defocus),
            },
            _ => None,
        }
    }

    fn on_mouse(&'a mut self, _mouse: crossterm::event::MouseEvent) -> Self::Reaction {
        None
    }
}

#[derive(Debug)]
pub enum StatusLineReaction {
    Defocus,
    ExecuteCommand(String),
    Highlight(String),
    Search(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SearchKind {
    Highlight,
    Search,
}

impl SearchKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchKind::Highlight => "highlight",
            SearchKind::Search => "search",
        }
    }
}

impl Display for SearchKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug)]
pub enum StatusLineContent {
    Command(String),
    SearchPattern(SearchKind, String), // header, pattern
    Status(String),
}

impl StatusLineContent {
    pub fn with_empty() -> Self {
        StatusLineContent::Status(String::new())
    }

    pub fn header(&self) -> Option<&str> {
        match self {
            StatusLineContent::Command(_) => None,
            StatusLineContent::SearchPattern(h, _) => Some(h.as_str()),
            StatusLineContent::Status(s) => Some(s),
        }
    }

    pub fn editable(&self) -> Option<String> {
        match self {
            StatusLineContent::Command(s) => Some(format!(":{s}")),
            StatusLineContent::SearchPattern(_, s) => Some(s.to_string()),
            _ => None,
        }
    }
}
