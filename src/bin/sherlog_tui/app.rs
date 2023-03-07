use crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind};
use regex::Regex;
use tui::backend::Backend;
use tui::layout::Rect;

use super::filter_list::{FilterList, FilterListReaction};
use super::status_line::{StatusLine, StatusLineContent, StatusLineReaction};
use super::text_area::TextArea;
use crate::ty::{React, Render, RenderCursor, RenderWithState};
use sherlog::{Sherlog, TextLineRef};

pub(crate) struct App {
    core: Sherlog,
    terminal_size: Rect,
    text: TextArea,
    status: StatusLine,
    filters: FilterList,

    focus: Focus,
    pub wants_quit: bool,
    search_issued: bool,
}

impl App {
    pub fn new(core: Sherlog, filename: String, terminal_size: Rect) -> Self {
        let line_count = core.line_count();
        let mut app = App {
            core,
            terminal_size,
            text: TextArea::new(),
            status: StatusLine {
                content: StatusLineContent::Status(String::from("Type `:` to start command")),
                filename,
                line_count,
                line_shown: None,
            },
            filters: FilterList::new(),
            focus: Focus::General,
            wants_quit: false,
            search_issued: false,
        };
        app.update_displayed_lines();
        app
    }

    fn layout(area: Rect) -> Vec<Rect> {
        // Create two chunks for text view and command bar
        tui::layout::Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints(
                [
                    tui::layout::Constraint::Min(1),
                    tui::layout::Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(area)
    }

    fn process_command(&mut self, command: &str) {
        let words: Vec<_> = command.split_whitespace().collect();
        let mut err: Option<String> = None;
        self.focus = Focus::General;

        match words[..] {
            ["q" | "quit"] => self.wants_quit = true,
            ["h" | "highlight", ref rest @ ..] => {
                let value: String = rest.iter().copied().collect();
                self.status.enter_highlight_pattern_mode(value);
                self.focus = Focus::StatusLine;
            }
            ["s" | "search", ref rest @ ..] => {
                let value: String = rest.iter().copied().collect();
                self.status.enter_search_mode(value);
                self.focus = Focus::StatusLine;
            }
            ["w" | "wrap"] => {
                if self.text.toggle_wrap() {
                    self.status.print_info("word wrap on");
                } else {
                    self.status.print_info("word wrap off");
                }
            }
            _ => err = Some(format!("Unknown command: {command}")),
        }
        if let Some(msg) = err {
            self.status.print_error(msg);
        }
    }

    fn display_lines(&mut self, n: usize, dir: DisplayDirection) {
        // We request number of lines equal to terminal height as an upper approximation of what we can render. Text
        // area might be smaller but this seems as a good compromise and we don't have to track exact text area size
        // which depends on current layout or is ultimately determined turing render function execution.
        let request_cnt = self.terminal_size.height as usize;

        let new_lines: Vec<_> = match dir {
            DisplayDirection::Forward => self.core.get_lines(n, Some(request_cnt)),
            DisplayDirection::Reverse if n < request_cnt => {
                self.core.get_lines(0, Some(request_cnt))
            }
            DisplayDirection::Reverse => self.core.get_lines_rev(n, Some(request_cnt)),
        }
        .iter()
        .map(TextLineRef::to_text_line)
        .collect();

        let new_line_idx = new_lines.first().map(|first| first.line_num);
        self.status.line_shown = new_line_idx;
        self.text.lines = new_lines;
    }

    fn update_displayed_lines(&mut self) {
        self.display_lines(self.first_displayed_line_num(), DisplayDirection::Forward);
    }

    fn scroll_up(&mut self, n: usize) {
        self.display_lines(
            self.last_displayed_line_num().saturating_sub(n),
            DisplayDirection::Reverse,
        );
    }

    fn scroll_down(&mut self, n: usize) {
        self.display_lines(
            self.first_displayed_line_num().saturating_add(n),
            DisplayDirection::Forward,
        );
    }

    fn go_top(&mut self) {
        self.display_lines(0, DisplayDirection::Forward);
    }

    fn go_bottom(&mut self) {
        self.display_lines(usize::MAX, DisplayDirection::Reverse);
    }

    fn highlight(&mut self, pattern: &str) {
        if pattern.trim().is_empty() {
            self.core.highlight(None);
            self.status.clear();
        } else {
            match Regex::new(pattern) {
                Ok(re) => {
                    self.core.highlight(Some(re));
                    self.status.clear();
                }
                Err(e) => self.status.print_error(format!("Invalid pattern: {e}")),
            }
        }
        self.focus = Focus::General;
        self.update_displayed_lines();
    }

    fn search(&mut self, pattern: &str) {
        if pattern.is_empty() {
            self.core.search(None);
            self.status.print_info("Search cleared");
        } else {
            match Regex::new(pattern) {
                Ok(re) => {
                    self.core.search(Some(re));
                    self.status.clear();
                }
                Err(e) => self
                    .status
                    .print_error(format!("Invalid search pattern: {e}")),
            }

            match self
                .core
                .next_search_result(self.first_displayed_line_num())
            {
                Some(n) => {
                    self.search_issued = true;
                    self.display_lines(n, DisplayDirection::Forward);
                }
                None => self
                    .status
                    .print_error(format!("Pattern not found: {pattern}")),
            }
        }
    }

    fn first_displayed_line_num(&self) -> usize {
        self.text.first_line().map(|l| l.line_num).unwrap_or(0)
    }

    fn last_displayed_line_num(&self) -> usize {
        self.text.last_line().map(|l| l.line_num).unwrap_or(0)
    }

    fn on_resize(&mut self, x: u16, y: u16) {
        let new_terminal_size = Rect::new(0, 0, x, y);
        if self.terminal_size != new_terminal_size {
            self.terminal_size = new_terminal_size;
            self.display_lines(self.first_displayed_line_num(), DisplayDirection::Forward);
        }
    }

    fn go_to_next_search_result(&mut self) {
        if !self.search_issued {
            self.status
                .print_error("No search issued. Use / or search command.");
        } else {
            match self
                .core
                .next_search_result(self.first_displayed_line_num() + 1)
            {
                Some(n) => {
                    self.display_lines(n, DisplayDirection::Forward);
                }
                None => {
                    self.status.print_info("No more results below");
                }
            }
        }
    }

    fn go_to_prev_search_result(&mut self) {
        if !self.search_issued {
            self.status
                .print_error("No search issued. Use / or search command.");
        } else {
            match self
                .core
                .prev_search_result(self.first_displayed_line_num() - 1)
            {
                Some(n) => {
                    self.display_lines(n, DisplayDirection::Forward);
                }
                None => {
                    self.status.print_info("No more results upwards");
                }
            }
        }
    }

    pub fn render<B: Backend>(&mut self, f: &mut tui::Frame<B>) {
        let area = f.size();
        let layout = App::layout(area);
        let text = self.text.widget();
        let status = self.status.widget();
        let filters_popup = make_filter_popup_area(f);
        let filters = if self.focus == Focus::Filters {
            Some(self.filters.widget())
        } else {
            None
        };

        let cursor = match self.focus {
            Focus::General => None,
            Focus::StatusLine => status.cursor(layout[STATUS_LAYOUT_IDX]),
            Focus::Filters => filters.as_ref().and_then(|f| f.0.cursor(filters_popup)),
        };

        f.render_widget(text, layout[TEXT_LAYOUT_IDX]);
        f.render_widget(status, layout[STATUS_LAYOUT_IDX]);

        if let Some(filters) = filters {
            f.render_stateful_widget(filters.0, filters_popup, filters.1);
        }

        if let Some(c) = cursor {
            f.set_cursor(c.x, c.y)
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.on_key(key),
            Event::Mouse(mouse) => self.on_mouse(mouse),
            Event::Resize(x, y) => self.on_resize(x, y),
            _ => (),
        }
    }
}

const TEXT_LAYOUT_IDX: usize = 0;
const STATUS_LAYOUT_IDX: usize = 1;

enum DisplayDirection {
    Forward,
    Reverse,
}

impl<'a> React<'a> for App {
    type Reaction = ();
    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> Self::Reaction {
        if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
            self.wants_quit = true;
            return;
        }
        match self.focus {
            Focus::StatusLine => match self.status.on_key(key) {
                None => {}
                Some(StatusLineReaction::Defocus) => self.focus = Focus::General,
                Some(StatusLineReaction::ExecuteCommand(c)) => {
                    self.process_command(&c);
                }
                Some(StatusLineReaction::Highlight(s)) => {
                    self.highlight(&s);
                    self.focus = Focus::General;
                }
                Some(StatusLineReaction::Search(s)) => {
                    self.search(&s);
                    self.focus = Focus::General;
                }
            },
            Focus::Filters => match self.filters.on_key(key) {
                FilterListReaction::Nothing => {}
                FilterListReaction::Defocus => {
                    self.focus = Focus::General;
                    let applied_filters = self.filters.make_regex_filter_vec();
                    let applied_filters_len = applied_filters.len();
                    self.core.filter(self.filters.make_regex_filter_vec());
                    match applied_filters_len {
                        0 => self
                            .status
                            .print_info("no filters applied - log unfiltered"),
                        1 => self.status.print_info("one filter applied"),
                        n => self.status.print_info(format!("{n} filters applied")),
                    }
                    self.update_displayed_lines()
                }
            },
            Focus::General => {
                match key.code {
                    KeyCode::Up => self.scroll_up(1),
                    KeyCode::Down => self.scroll_down(1),
                    KeyCode::Left => self.text.scroll_left(),
                    KeyCode::Right => self.text.scroll_right(),
                    KeyCode::Home => self.go_top(),
                    KeyCode::End => self.go_bottom(),
                    KeyCode::Esc => {
                        self.focus = Focus::General;
                        self.status.clear();
                    }
                    KeyCode::Char(':') => {
                        self.focus = Focus::StatusLine;
                        self.status.enter_command_mode();
                    }
                    KeyCode::Char('/') => {
                        self.focus = Focus::StatusLine;
                        self.status.enter_search_mode(String::new());
                    }
                    KeyCode::Char('f') => {
                        self.focus = Focus::Filters;
                        self.status
                            .print_info("<a>add  <e>edit  <d>disable (toggle) <n>negate (toggle)");
                    }
                    KeyCode::Char('n') => self.go_to_next_search_result(),
                    KeyCode::Char('N') => self.go_to_prev_search_result(),
                    _ => {}
                };
            }
        }
    }

    fn on_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> Self::Reaction {
        match mouse.kind {
            MouseEventKind::ScrollDown if mouse.modifiers == KeyModifiers::CONTROL => {
                self.scroll_down(10)
            }
            MouseEventKind::ScrollUp if mouse.modifiers == KeyModifiers::CONTROL => {
                self.scroll_up(10)
            }
            MouseEventKind::ScrollDown => self.scroll_down(3),
            MouseEventKind::ScrollUp => self.scroll_up(3),
            _ => {}
        };
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focus {
    General,
    StatusLine,
    Filters,
}

fn make_filter_popup_area<B: Backend>(f: &tui::Frame<B>) -> Rect {
    tui::layout::Layout::default()
        .horizontal_margin(5)
        .vertical_margin(1)
        .direction(tui::layout::Direction::Vertical)
        .constraints([
            tui::layout::Constraint::Min(0),
            tui::layout::Constraint::Ratio(1, 2),
        ])
        .split(f.size())[1]
}
