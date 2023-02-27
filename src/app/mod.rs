use std::str::FromStr;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use regex::Regex;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::{layout, Frame};

use crate::sherlog_core::{Sherlog, TextLineRef};

use self::components::filters::{Filter, FilterList};
use self::components::status_line::{SearchKind, StatusLine, StatusLineContent};
use self::components::text_area::TextArea;

mod components;

pub struct App {
    pub core: Sherlog,

    pub status_line: StatusLine,
    pub filter_list: FilterList,
    pub text_area: TextArea,

    pub focus: Focus,

    pub wants_quit: bool,
    pub filter_is_highlight: bool,

    pub y: usize,
    pub max_y: usize,
}

impl App {
    pub fn new(core: Sherlog, filename: String, height: u16) -> Self {
        let line_count = core.line_count();

        let mut app = App {
            core,

            status_line: StatusLine {
                content: StatusLineContent::Status(String::from("Type `:` to start command")),
                filename,
                line_count,
            },
            filter_list: FilterList {
                entries: vec![
                    Filter::from_str("Filter 1").unwrap(),
                    Filter::from_str("Filter 2").unwrap(),
                    Filter::from_str("Filter 3").unwrap(),
                ],
                visible: false,
                selected: 0,
            },
            text_area: TextArea {
                x: 0,
                height,
                wrap: false,
                lines: vec![],
            },

            focus: Focus::General,

            wants_quit: false,
            filter_is_highlight: false,

            y: 0,
            max_y: line_count - 1,
        };
        app.update_presented_lines();
        app
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                self.wants_quit = true;
                return;
            }
            KeyCode::Char(':') if self.focus == Focus::General => {
                self.focus = Focus::StatusLine;
                self.status_line.content = StatusLineContent::Command(String::from(':'));
                return;
            }
            KeyCode::Char('f') if self.focus == Focus::General => {
                self.focus = Focus::Filters;
                self.filter_list.visible = true;
                return;
            }
            KeyCode::Enter if self.focus == Focus::StatusLine => match &self.status_line.content {
                StatusLineContent::Command(command) => {
                    self.process_command(&command[1..].to_owned())
                }
                StatusLineContent::SearchPattern(SearchKind::Highlight, pattern) => {
                    let pattern = pattern.clone();
                    self.highlight(&pattern);
                    self.focus = Focus::General;
                }
                StatusLineContent::SearchPattern(SearchKind::Filter, pattern) => {
                    let pattern = pattern.clone();
                    self.filter(&pattern);
                    self.focus = Focus::General;
                }
                StatusLineContent::Status(_) => {}
            },
            _ => {}
        }

        match self.focus {
            Focus::General => self.on_key_no_focus(key),
            Focus::StatusLine => self.status_line.on_key(key),
            Focus::Filters => self.filter_list.on_key(key),
        }

        if key.code == KeyCode::Esc {
            self.focus = Focus::General
        }
    }

    pub fn on_key_no_focus(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Up => self.scroll_up(1),
            KeyCode::Down => self.scroll_down(1),
            KeyCode::Left => self.scroll_left(),
            KeyCode::Right => self.scroll_right(),
            KeyCode::Home => self.go_top(),
            KeyCode::End => self.go_bottom(),
            _ => {}
        }
        self.update_presented_lines()
    }

    fn on_resize(&mut self, x: u16, y: u16) {
        let chunks = self.layout(Rect::new(0, 0, x, y));
        let new_height = chunks[0].height;
        if self.text_area.height != new_height {
            self.text_area.height = new_height;
            self.update_presented_lines();
        }
    }

    fn update_presented_lines(&mut self) {
        self.text_area.lines = self
            .core
            .get_lines(self.y, Some(self.text_area.height as usize))
            .iter()
            .map(TextLineRef::to_text_line)
            .collect();
    }

    fn last_line_shown(&self) -> Option<usize> {
        self.text_area.lines.last().map(|l| l.line_num)
    }

    fn scroll_up(&mut self, line_cnt: usize) {
        self.y = self.y.saturating_sub(line_cnt);
    }

    fn scroll_down(&mut self, line_cnt: usize) {
        self.y = self.y.saturating_add(line_cnt);
        if self.y > self.max_y {
            self.y = self.max_y;
        }
    }

    fn scroll_left(&mut self) {
        self.text_area.x = self.text_area.x.saturating_sub(1);
    }

    fn scroll_right(&mut self) {
        self.text_area.x = self.text_area.x.saturating_add(1);
    }

    fn go_top(&mut self) {
        self.y = 0;
    }

    fn go_bottom(&mut self) {
        self.y = self.max_y;
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

    fn process_command(&mut self, command: &str) {
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
                if self.text_area.wrap {
                    self.status_line.print_info("word wrap off");
                    self.text_area.wrap = false;
                } else {
                    self.status_line.print_info("word wrap on");
                    self.text_area.wrap = true;
                }
            }
            _ => err = Some(format!("Unknown command: {}", command)),
        }
        if let Some(msg) = err {
            self.status_line.print_error(msg);
        }
    }

    fn layout(&self, area: Rect) -> Vec<Rect> {
        // Create two chunks for text view and command bar
        layout::Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints([layout::Constraint::Min(1), layout::Constraint::Length(1)].as_ref())
            .split(area)
    }
}

pub fn render_ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = app.layout(f.size());
    f.render_widget(app.text_area.widget(), chunks[0]);
    f.render_widget(app.status_line.widget(app.last_line_shown()), chunks[1]);

    if let Some((widget, mut state)) = app.filter_list.widget() {
        let overlay_area = layout::Layout::default()
            .horizontal_margin(5)
            .vertical_margin(1)
            .direction(layout::Direction::Vertical)
            .constraints([layout::Constraint::Min(0), layout::Constraint::Ratio(1, 2)])
            .split(f.size())[1];

        f.render_stateful_widget(widget, overlay_area, &mut state);
    }

    if let Some(x) = app.status_line.cursor_x() {
        f.set_cursor(x, chunks[1].y);
    }
}

pub fn handle_event(app: &mut App, event: Event) {
    match event {
        Event::Key(key) => {
            app.on_key(key);
        }
        Event::Mouse(mouse) => match mouse.kind {
            event::MouseEventKind::ScrollDown if mouse.modifiers == KeyModifiers::CONTROL => {
                app.scroll_down(10);
            }
            event::MouseEventKind::ScrollUp if mouse.modifiers == KeyModifiers::CONTROL => {
                app.scroll_up(10);
            }
            event::MouseEventKind::ScrollDown => app.scroll_down(1),
            event::MouseEventKind::ScrollUp => app.scroll_up(1),
            _ => {}
        },
        Event::Resize(x, y) => app.on_resize(x, y),
        _ => (),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focus {
    General,
    StatusLine,
    Filters,
}
