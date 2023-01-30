use std::fmt::Display;

use regex::Regex;

use crate::sherlog_core::Sherlog;

pub struct App {
    pub core: Sherlog,
    pub view_offset_y: usize,
    pub view_offset_x: usize,
    pub status: StatusLine,
    pub wants_quit: bool,
    pub filter_is_highlight: bool,
    pub wrap_lines: bool,
    pub filename: String,
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

pub enum StatusLine {
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
    pub fn new(core: Sherlog, filename: String) -> Self {
        App {
            core,
            view_offset_y: 0,
            view_offset_x: 0,
            status: StatusLine::Status(String::from("Type `:` to start command")),
            wants_quit: false,
            filter_is_highlight: false,
            wrap_lines: false,
            filename,
        }
    }

    pub fn scroll_up(&mut self, line_cnt: usize) {
        self.view_offset_y = self.view_offset_y.saturating_sub(line_cnt);
    }

    pub fn scroll_down(&mut self, line_cnt: usize) {
        self.view_offset_y = self.view_offset_y.saturating_add(line_cnt);
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
            "wrap" => {
                if self.wrap_lines {
                    self.print_info("word wrap off");
                    self.wrap_lines = false;
                } else {
                    self.print_info("word wrap on");
                    self.wrap_lines = true;
                }
            }
            other => err = Some(format!("Unknown command: {}", other)),
        }
        if let Some(msg) = err {
            self.print_error(msg);
        }
    }

    pub fn print_error<S: Into<String>>(&mut self, error: S) {
        self.status = StatusLine::Status(error.into());
    }

    pub fn print_info<S: Into<String>>(&mut self, info: S) {
        self.status = StatusLine::Status(info.into());
    }

    pub fn clear_status(&mut self) {
        self.status = StatusLine::with_empty();
    }

    pub fn on_home(&mut self) {
        self.view_offset_y = 0;
    }

    pub fn on_end(&mut self) {
        self.view_offset_y = self.core.line_count() - 1;
    }
}
