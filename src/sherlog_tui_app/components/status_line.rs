use std::fmt::Display;

use crossterm::event::KeyCode;

use crate::sherlog_tui_app::{React, Render};
use crate::tui_widgets;

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
        self.content = StatusLineContent::Command(String::from(':'));
    }

    pub fn enter_highlight_pattern_mode(&mut self, value: String) {
        self.content = StatusLineContent::SearchPattern(SearchKind::Highlight, value);
    }

    fn cursor_x(&self) -> Option<u16> {
        match &self.content {
            s @ (StatusLineContent::Command(_) | StatusLineContent::SearchPattern(_, _)) => {
                Some(s.to_string().len() as u16)
            }
            StatusLineContent::Status(_) => None,
        }
    }
}

impl<'a> Render<'a> for StatusLine {
    type Widget = tui_widgets::StatusLine<'a>;

    fn widget(&'a self) -> Self::Widget {
        tui_widgets::StatusLine::new()
            .left(self.content.to_string())
            .right_maybe(
                self.line_shown
                    .map(|line| format!("{}/{}", line, self.line_count)),
            )
            .right(self.filename.as_ref())
            .with_cursor_maybe(self.cursor_x())
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
                    Some(StatusLineReaction::ExecuteCommand(String::from(&s[1..])))
                }
                StatusLineContent::SearchPattern(SearchKind::Highlight, s) => {
                    Some(StatusLineReaction::Highlight(s.clone()))
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SearchKind {
    Highlight,
    // More to follow
}

impl Display for SearchKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SearchKind::Highlight => "highlight",
        })
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
