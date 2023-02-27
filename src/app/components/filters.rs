use std::fmt::Display;
use std::str::FromStr;

use crossterm::event::KeyCode;
use regex::Regex;
use tui::style::{Color, Style};
use tui::widgets::{BorderType, List, ListItem, ListState};

use crate::tui_widgets::OverlayBlock;

pub struct FilterList {
    pub entries: Vec<Filter>,
    pub selected: usize,
    pub edit_cursor: Option<(u16, u16)>,
}

impl FilterList {
    pub fn select_next(&mut self) {
        self.selected += 1;
        if self.selected > self.entries.len() - 1 {
            self.selected = self.entries.len() - 1;
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn widget(&self) -> (OverlayBlock<List>, ListState) {
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected));
        let overlayed_list = OverlayBlock::new(
            //TODO: move support
            //TODO: apply filters on close
            List::new(
                self.entries
                    .iter()
                    .map(|s| ListItem::new(s.to_string()))
                    .collect::<Vec<_>>(),
            )
            .highlight_style(Style::default().fg(Color::Yellow)),
        )
        .border(BorderType::Double, Style::default().fg(Color::Green));
        (overlayed_list, list_state)
    }

    pub fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        let editing = self.edit_cursor.is_some();
        match key.code {
            KeyCode::Char(c) if editing => self.on_edit_insert_char(c),
            KeyCode::Backspace if editing => self.on_edit_backspace(),
            KeyCode::Left if editing => self.on_edit_move_cursor_left(),
            KeyCode::Right if editing => self.on_edit_move_cursor_right(),
            KeyCode::Esc if editing => self.edit_cursor = None,
            KeyCode::Up => self.select_prev(),
            KeyCode::Down => self.select_next(),
            KeyCode::Char('a') => self.add_new(),
            KeyCode::Char('d') => self.delete_selected(),
            KeyCode::Char('e') => self.edit_selected(),
            _ => {}
        }
    }

    fn on_edit_insert_char(&mut self, c: char) {
        self.entries
            .get_mut(self.selected)
            .zip(self.edit_cursor.as_mut())
            .map(|(filter, cur)| {
                filter.insert_at(c, cur.0 as usize);
                (*cur).0 += 1;
            });
    }

    fn on_edit_backspace(&mut self) {
        self.entries
            .get_mut(self.selected)
            .zip(self.edit_cursor.as_mut())
            .map(|(filter, c)| {
                filter.backspace_at(c.0 as usize);
                (*c).0 = c.0.saturating_sub(1);
            });
    }

    fn on_edit_move_cursor_right(&mut self) {
        self.edit_cursor
            .as_mut()
            .map(|c| (*c).0 = c.0.saturating_add(1));
    }

    fn on_edit_move_cursor_left(&mut self) {
        self.edit_cursor
            .as_mut()
            .map(|c| (*c).0 = c.0.saturating_sub(1));
    }

    fn add_new(&mut self) {
        self.entries.push(Filter::new("New filter"));
    }

    fn delete_selected(&mut self) {
        if self.selected < self.entries.len() {
            self.entries.remove(self.selected);
        }
    }

    fn edit_selected(&mut self) {
        self.edit_cursor = self
            .selected_entry()
            .map(|e| (e.value.as_str().len() as u16, self.selected as u16));
    }

    fn selected_entry(&self) -> Option<&Filter> {
        self.entries.get(self.selected)
    }
}

pub struct Filter {
    pub value: FilterValue,
    pub negate: bool,
    pub active: bool,
}

impl Filter {
    pub fn new(s: &str) -> Self {
        Filter {
            value: FilterValue::new(s),
            negate: false,
            active: true,
        }
    }

    pub fn backspace_at(&mut self, pos: usize) {
        let old = self.value.as_str();
        let (left, right) = old.split_at(pos);
        if left.is_empty() {
            return;
        }

        let new_left = &left[0..left.len() - 1];
        let new = format!("{}{}", new_left, right);
        self.value = FilterValue::new(&new);
    }

    pub fn insert_at(&mut self, c: char, pos: usize) {
        let old = self.value.as_str();

        let (left, right) = old.split_at(pos);
        let new = format!("{}{}{}", left, c, right);
        self.value = FilterValue::new(&new);
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.value.as_str())
    }
}

pub enum FilterValue {
    Valid(Regex),
    Invalid(String),
}

impl FilterValue {
    fn new(s: &str) -> Self {
        match Regex::from_str(s) {
            Ok(r) => FilterValue::Valid(r),
            Err(_) => FilterValue::Invalid(String::from(s)),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            FilterValue::Valid(r) => r.as_str(),
            FilterValue::Invalid(s) => &s,
        }
    }
}
