use std::fmt::Display;
use std::str::FromStr;

use crossterm::event::KeyCode;
use regex::Regex;
use tui::style::{Color, Style};
use tui::widgets::{BorderType, List, ListItem, ListState};

use crate::sherlog_core::RegexFilter;
use crate::tui_widgets::OverlayBlock;

#[derive(Default)]
pub struct FilterList {
    pub entries: Vec<FilterEntry>,
    pub selected: usize,
    edit_cursor: Option<u16>,
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
            List::new(
                self.entries
                    .iter()
                    .map(Self::build_list_item)
                    .collect::<Vec<_>>(),
            )
            .highlight_style(Style::default().fg(Color::Yellow)),
        )
        .border(BorderType::Double, Style::default().fg(Color::Green));
        (overlayed_list, list_state)
    }

    pub fn cursor(&self) -> Option<(u16, u16)> {
        self.edit_cursor
            .zip(self.entries.get(self.selected))
            .map(|(cursor, filter)| {
                // This offset should be connefcted with build_list_item
                let mut offset = 0;
                if filter.negate {
                    offset += 1;
                }
                if !filter.active {
                    offset += 1;
                }
                if offset > 0 {
                    offset += 1;
                }
                ((cursor + offset) as u16, self.selected as u16)
            })
    }

    pub fn is_editing(&self) -> bool {
        self.edit_cursor.is_some()
    }

    fn build_list_item(f: &FilterEntry) -> ListItem {
        let mut s = String::new();
        if !f.active {
            s.push_str("#")
        }
        if f.negate {
            s.push_str("!");
        }
        if !s.is_empty() {
            s.push(' ')
        }
        s.push_str(f.value.as_str());
        ListItem::new(s)
    }

    pub fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        let editing = self.edit_cursor.is_some();
        match key.code {
            KeyCode::Char(c) if editing => self.on_edit_insert_char(c),
            KeyCode::Backspace if editing => self.on_edit_backspace(),
            KeyCode::Left if editing => self.on_edit_move_cursor_left(),
            KeyCode::Right if editing => self.on_edit_move_cursor_right(),
            //TODO: Esc cancel, enter apply
            KeyCode::Esc | KeyCode::Enter if editing => self.edit_cursor = None,
            KeyCode::Up => self.select_prev(),
            KeyCode::Down => self.select_next(),
            KeyCode::Char('a') => {
                self.append_new();
                self.select_next();
                self.edit_selected();
            }
            KeyCode::Char('i') => {
                self.insert_new();
                self.edit_selected();
            }
            KeyCode::Char('d') => self.toggle_disable_selected(),
            KeyCode::Char('e') => self.edit_selected(),
            KeyCode::Char('n') => self.negate_selected(),
            KeyCode::Backspace | KeyCode::Delete => self.delete_selected(),
            _ => {}
        }
    }

    fn on_edit_insert_char(&mut self, c: char) {
        self.entries
            .get_mut(self.selected)
            .zip(self.edit_cursor.as_mut())
            .map(|(filter, cur)| {
                filter.insert_at(c, *cur as usize);
                *cur += 1;
            });
    }

    fn on_edit_backspace(&mut self) {
        self.entries
            .get_mut(self.selected)
            .zip(self.edit_cursor.as_mut())
            .map(|(filter, c)| {
                filter.backspace_at(*c as usize);
                *c = c.saturating_sub(1);
            });
    }

    fn on_edit_move_cursor_right(&mut self) {
        self.edit_cursor.as_mut().map(|c| *c = c.saturating_add(1));
    }

    fn on_edit_move_cursor_left(&mut self) {
        self.edit_cursor.as_mut().map(|c| *c = c.saturating_sub(1));
    }

    fn append_new(&mut self) {
        self.entries.insert(
            (self.selected + 1).clamp(0, self.entries.len()),
            FilterEntry::new(""),
        );
    }

    fn insert_new(&mut self) {
        self.entries.insert(self.selected, FilterEntry::new(""));
    }

    fn negate_selected(&mut self) {
        self.entries
            .get_mut(self.selected)
            .map(|f| f.negate = !f.negate);
    }

    fn toggle_disable_selected(&mut self) {
        self.entries
            .get_mut(self.selected)
            .map(|f| f.active = !f.active);
    }

    fn delete_selected(&mut self) {
        if self.selected < self.entries.len() {
            self.entries.remove(self.selected);
        }
    }

    fn edit_selected(&mut self) {
        self.edit_cursor = self.selected_entry().map(|e| e.value.as_str().len() as u16);
    }

    fn selected_entry(&self) -> Option<&FilterEntry> {
        self.entries.get(self.selected)
    }

    pub fn make_regex_filter_vec(&self) -> Vec<RegexFilter> {
        self.entries
            .iter()
            .filter_map(FilterEntry::try_to_regex_filter)
            .collect()
    }
}

pub struct FilterEntry {
    pub value: FilterValue,
    pub negate: bool,
    pub active: bool,
}

impl FilterEntry {
    pub fn new(s: &str) -> Self {
        FilterEntry {
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

    pub fn try_to_regex_filter(&self) -> Option<RegexFilter> {
        match &self.value {
            FilterValue::Valid(pattern) if self.active => Some(RegexFilter {
                pattern: pattern.clone(),
                negate: self.negate,
            }),
            _ => None,
        }
    }
}

impl Display for FilterEntry {
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
