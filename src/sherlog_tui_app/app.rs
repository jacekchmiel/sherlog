use crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind};
use regex::Regex;
use tui;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::widgets::{ListState, StatefulWidget, Widget};

use crate::sherlog_core::{Sherlog, TextLineRef};
use crate::tui_widgets::{ListWithCursor, OverlayBlock};

use super::filter_list::{FilterList, FilterListReaction};
use super::status_line::{StatusLine, StatusLineContent, StatusLineReaction};
use super::text_area::TextArea;
use super::{React, Render, RenderCursor, RenderWithState};

pub(crate) struct App {
    core: Sherlog,
    text: TextArea,
    status: StatusLine,
    filters: FilterList,

    focus: Focus,
    pub wants_quit: bool,
}

impl App {
    pub fn new(core: Sherlog, filename: String, terminal_size: Rect) -> Self {
        let line_count = core.line_count();
        let mut app = App {
            core,
            text: TextArea::new(terminal_size.height - 1, line_count - 1),
            status: StatusLine {
                content: StatusLineContent::Status(String::from("Type `:` to start command")),
                filename,
                line_count,
                line_shown: None,
            },
            filters: FilterList::new(),
            focus: Focus::General,
            wants_quit: false,
        };
        app.update_presented_lines();
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

        match &words[..] {
            &["q" | "quit"] => self.wants_quit = true,
            &["h" | "highlight", ref rest @ ..] => {
                let value: String = rest.iter().copied().collect();
                self.status.enter_highlight_pattern_mode(value);
                self.focus = Focus::StatusLine;
            }
            &["w" | "wrap"] => {
                if self.text.toggle_wrap() {
                    self.status.print_info("word wrap on");
                } else {
                    self.status.print_info("word wrap off");
                }
            }
            _ => err = Some(format!("Unknown command: {}", command)),
        }
        if let Some(msg) = err {
            self.status.print_error(msg);
        }
    }

    fn update_presented_lines(&mut self) {
        self.text.lines = self
            .core
            .get_lines(self.text.y, Some(self.text.height as usize))
            .iter()
            .map(TextLineRef::to_text_line)
            .collect();
        self.status.line_shown = Some(self.text.y + 1);
    }

    fn highlight(&mut self, pattern: &str) {
        if pattern.trim().is_empty() {
            self.core.highlight = None;
            self.status.clear();
        } else {
            match Regex::new(pattern) {
                Ok(re) => {
                    self.core.highlight = Some(re);
                    self.status.clear();
                }
                Err(e) => self.status.print_error(format!("Invalid pattern: {}", e)),
            }
        }
        self.focus = Focus::General;
    }

    fn on_resize(&mut self, x: u16, y: u16) {
        let terminal_size = Rect::new(0, 0, x, y);
        let chunks = Self::layout(terminal_size);
        let new_height = chunks[0].height;
        if self.text.height != new_height {
            self.text.height = new_height;
            self.update_presented_lines();
        }
    }
}

impl<'a> RenderWithState<'a> for App {
    type Widget = AppWidget<'a>;

    fn widget(&'a mut self) -> (Self::Widget, &mut ListState) {
        let (filters_widget, filters_state) = self.filters.widget();
        (
            AppWidget {
                focus: self.focus,
                text: self.text.widget(),
                status: self.status.widget(),
                filters: filters_widget,
            },
            filters_state,
        )
    }
}
const TEXT_LAYOUT_IDX: usize = 0;
const STATUS_LAYOUT_IDX: usize = 1;

pub(crate) struct AppWidget<'a> {
    focus: Focus,
    text: <TextArea as Render<'a>>::Widget,
    status: <StatusLine as Render<'a>>::Widget,
    filters: OverlayBlock<ListWithCursor<'a>>,
}

impl AppWidget<'_> {
    fn make_filter_popup_area(&self, area: Rect) -> Rect {
        tui::layout::Layout::default()
            .horizontal_margin(5)
            .vertical_margin(1)
            .direction(tui::layout::Direction::Vertical)
            .constraints([
                tui::layout::Constraint::Min(0),
                tui::layout::Constraint::Ratio(1, 2),
            ])
            .split(area)[1]
    }
}

impl<'a> StatefulWidget for AppWidget<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer, state: &mut Self::State) {
        let layout = App::layout(area);
        let filter_popup_area = self.make_filter_popup_area(area);
        self.text.render(layout[TEXT_LAYOUT_IDX], buf);
        self.status.render(layout[STATUS_LAYOUT_IDX], buf);

        if self.focus == Focus::Filters {
            // This is ugly :/
            <OverlayBlock<ListWithCursor<'a>> as StatefulWidget>::render(
                self.filters,
                filter_popup_area,
                buf,
                state,
            );
        }
    }
}

impl<'a> RenderCursor for AppWidget<'a> {
    fn cursor(&self, area: Rect) -> Option<super::Cursor> {
        let layout = App::layout(area);
        match self.focus {
            Focus::General => None,
            Focus::StatusLine => self.status.cursor(layout[STATUS_LAYOUT_IDX]),
            Focus::Filters => self.filters.cursor(self.make_filter_popup_area(area)),
        }
    }
}

impl<'a> React<'a> for App {
    type Reaction = ();
    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> Self::Reaction {
        if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
            self.wants_quit = true;
            return;
        }
        let mut update_needed = false;
        match self.focus {
            Focus::StatusLine => match self.status.on_key(key) {
                None => {}
                Some(StatusLineReaction::Defocus) => self.focus = Focus::General,
                Some(StatusLineReaction::ExecuteCommand(c)) => {
                    self.process_command(&c);
                }
                Some(StatusLineReaction::Highlight(s)) => {
                    self.highlight(&s);
                    update_needed = true;
                }
            },
            Focus::Filters => match self.filters.on_key(key) {
                FilterListReaction::Nothing => {}
                FilterListReaction::Defocus => {
                    self.focus = Focus::General;
                    self.core.filters = self.filters.make_regex_filter_vec();
                    match self.core.filters.len() {
                        0 => self
                            .status
                            .print_info("no filters applied - log unfiltered"),
                        1 => self.status.print_info("one filter applied"),
                        n => self.status.print_info(format!("{} filters applied", n)),
                    }
                    self.update_presented_lines()
                }
            },
            Focus::General => {
                update_needed = match key.code {
                    KeyCode::Up => self.text.scroll_up(1),
                    KeyCode::Down => self.text.scroll_down(1),
                    KeyCode::Left => self.text.scroll_left(),
                    KeyCode::Right => self.text.scroll_right(),
                    KeyCode::Home => self.text.go_top(),
                    KeyCode::End => self.text.go_bottom(),
                    KeyCode::Esc => {
                        self.focus = Focus::General;
                        self.status.clear();
                        false
                    }
                    KeyCode::Char(':') if self.focus == Focus::General => {
                        self.focus = Focus::StatusLine;
                        self.status.enter_command_mode();
                        false
                    }
                    KeyCode::Char('f') if self.focus == Focus::General => {
                        self.focus = Focus::Filters;
                        self.status
                            .print_info("<a>add  <e>edit  <d>disable (toggle) <n>negate (toggle)");
                        false
                    }
                    _ => false,
                };
            }
        }
        if update_needed {
            self.update_presented_lines();
        }
    }

    fn on_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> Self::Reaction {
        let update_needed = match mouse.kind {
            MouseEventKind::ScrollDown if mouse.modifiers == KeyModifiers::CONTROL => {
                self.text.scroll_down(10)
            }
            MouseEventKind::ScrollUp if mouse.modifiers == KeyModifiers::CONTROL => {
                self.text.scroll_up(10)
            }
            MouseEventKind::ScrollDown => self.text.scroll_down(3),
            MouseEventKind::ScrollUp => self.text.scroll_up(3),
            _ => false,
        };
        if update_needed {
            self.update_presented_lines();
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focus {
    General,
    StatusLine,
    Filters,
}

pub(crate) fn render_app<B: Backend>(app: &mut App, f: &mut tui::Frame<B>) {
    let area = f.size();
    let (app_widget, app_state) = app.widget();
    let cursor = app_widget.cursor(area);
    f.render_stateful_widget(app_widget, area, app_state);

    if let Some(c) = cursor {
        f.set_cursor(c.x, c.y)
    }
}

pub(crate) fn handle_event(app: &mut App, event: Event) {
    match event {
        Event::Key(key) => app.on_key(key),
        Event::Mouse(mouse) => app.on_mouse(mouse),
        Event::Resize(x, y) => app.on_resize(x, y),
        _ => (),
    }
}
