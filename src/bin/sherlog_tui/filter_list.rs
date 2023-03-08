use std::fmt::Display;
use std::str::FromStr;

use crossterm::event::KeyCode;
use regex::Regex;
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState};

use crate::ty::{Cursor, React, RenderWithState};
use crate::widgets::{ListWithCursor, OpaqueOverlay};
use sherlog::RegexFilter;

#[derive(Default)]
pub struct FilterList {
    pub entries: Vec<FilterEntry>,
    edit_cursor: Option<u16>,
    pub state: ListState,
}

impl FilterList {
    pub fn new() -> Self {
        FilterList {
            entries: Vec::new(),
            edit_cursor: None,
            state: ListState::default(),
        }
    }
    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn select(&mut self, n: usize) {
        self.state.select(Some(n.clamp(0, self.entries.len() - 1)))
    }

    pub fn select_next(&mut self) {
        match self.selected() {
            Some(s) => self.select(s + 1),
            None if !self.entries.is_empty() => self.select(0),
            None => {}
        }
    }

    pub fn select_prev(&mut self) {
        match self.selected() {
            Some(s) => self.select(s.saturating_sub(1)),
            None if !self.entries.is_empty() => self.select(self.entries.len() - 1),
            None => {}
        }
    }

    fn build_list_item(f: &FilterEntry) -> ListItem {
        let mut s = String::new();
        if !f.active {
            s.push('#')
        }
        if f.negate {
            s.push('!');
        }
        if !s.is_empty() {
            s.push(' ')
        }
        s.push_str(f.value.as_str());
        ListItem::new(s)
    }

    fn on_edit_insert_char(&mut self, c: char) {
        if let Some(selected) = self.selected() {
            if let Some((filter, cursor)) = self
                .entries
                .get_mut(selected)
                .zip(self.edit_cursor.as_mut())
            {
                filter.insert_at(c, *cursor as usize);
                *cursor += 1;
            }
        }
    }

    fn on_edit_backspace(&mut self) {
        if let Some(selected) = self.selected() {
            if let Some((filter, cursor)) = self
                .entries
                .get_mut(selected)
                .zip(self.edit_cursor.as_mut())
            {
                filter.backspace_at(*cursor as usize);
                *cursor = cursor.saturating_sub(1);
            }
        }
    }

    fn on_edit_move_cursor_right(&mut self) {
        if let Some(c) = self.edit_cursor.as_mut() {
            *c = c.saturating_add(1);
        }
    }

    fn on_edit_move_cursor_left(&mut self) {
        if let Some(c) = self.edit_cursor.as_mut() {
            *c = c.saturating_sub(1);
        }
    }

    fn append_new(&mut self) {
        self.entries.insert(
            (self.selected().unwrap_or_default() + 1).clamp(0, self.entries.len()),
            FilterEntry::new(""),
        );
    }

    fn insert_new(&mut self) {
        self.entries.insert(
            self.selected()
                .unwrap_or_default()
                .clamp(0, self.entries.len()),
            FilterEntry::new(""),
        );
    }

    fn selected_filter_mut(&mut self) -> Option<&mut FilterEntry> {
        self.selected().and_then(|s| self.entries.get_mut(s))
    }

    fn negate_selected(&mut self) {
        if let Some(filter) = self.selected_filter_mut() {
            filter.negate = !filter.negate
        }
    }

    fn toggle_disable_selected(&mut self) {
        if let Some(filter) = self.selected_filter_mut() {
            filter.active = !filter.active
        }
    }

    fn delete_selected(&mut self) {
        if let Some(selected) = self.selected() {
            if selected < self.entries.len() {
                self.entries.remove(selected);
            }
        }
    }

    fn edit_selected(&mut self) {
        self.edit_cursor = self.selected_entry().map(|e| e.value.as_str().len() as u16);
    }

    fn selected_entry(&self) -> Option<&FilterEntry> {
        self.selected().and_then(|s| self.entries.get(s))
    }

    pub fn make_regex_filter_vec(&self) -> Vec<RegexFilter> {
        self.entries
            .iter()
            .filter_map(FilterEntry::try_to_regex_filter)
            .collect()
    }

    fn cursor(&self) -> Option<Cursor> {
        self.selected().and_then(|selected| {
            self.edit_cursor
                .zip(self.entries.get(selected))
                .map(|(cursor, filter)| {
                    // This offset should be connected with build_list_item
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
                    Cursor::new(cursor + offset, selected as u16)
                })
        })
    }
}

impl RenderWithState for FilterList {
    type Widget<'a> = OpaqueOverlay<ListWithCursor<'a>>;

    fn widget(
        &mut self,
    ) -> (
        Self::Widget<'_>,
        &mut <Self::Widget<'_> as tui::widgets::StatefulWidget>::State,
    ) {
        (
            OpaqueOverlay(ListWithCursor {
                list: List::new(
                    self.entries
                        .iter()
                        .map(Self::build_list_item)
                        .collect::<Vec<_>>(),
                )
                .block(
                    Block::default()
                        .border_type(BorderType::Rounded)
                        .borders(Borders::all())
                        .title("Filters"),
                )
                .highlight_style(Style::default().fg(Color::Yellow)),

                cursor: self.cursor(),
            }),
            &mut self.state,
        )
    }
}

impl<'a> React<'a> for FilterList {
    type Reaction = FilterListReaction;

    fn on_key(&'a mut self, key: crossterm::event::KeyEvent) -> FilterListReaction {
        let editing = self.edit_cursor.is_some();
        let mut reaction = FilterListReaction::Nothing;
        match key.code {
            KeyCode::Char(c) if editing => self.on_edit_insert_char(c),
            KeyCode::Backspace if editing => self.on_edit_backspace(),
            KeyCode::Left if editing => self.on_edit_move_cursor_left(),
            KeyCode::Right if editing => self.on_edit_move_cursor_right(),
            KeyCode::Esc | KeyCode::Enter if editing => self.edit_cursor = None,
            KeyCode::Esc | KeyCode::Enter => reaction = FilterListReaction::Defocus,
            KeyCode::Up => self.select_prev(),
            KeyCode::Down => self.select_next(),
            KeyCode::Char('a') => {
                self.append_new();
                self.select_next();
                self.edit_selected();
            }
            KeyCode::Char('i') => {
                self.insert_new();
                if self.selected().is_none() {
                    self.select(0);
                }
                self.edit_selected();
            }
            KeyCode::Char('d') => self.toggle_disable_selected(),
            KeyCode::Char('e') => self.edit_selected(),
            KeyCode::Char('n') => self.negate_selected(),
            KeyCode::Backspace | KeyCode::Delete => self.delete_selected(),
            _ => {}
        };
        reaction
    }

    fn on_mouse(&'a mut self, _mouse: crossterm::event::MouseEvent) -> Self::Reaction {
        FilterListReaction::Nothing
    }
}

pub enum FilterListReaction {
    Nothing,
    Defocus,
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
        let new = format!("{new_left}{right}");
        self.value = FilterValue::new(&new);
    }

    pub fn insert_at(&mut self, c: char, pos: usize) {
        let old = self.value.as_str();

        let (left, right) = old.split_at(pos);
        let new = format!("{left}{c}{right}");
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
            FilterValue::Invalid(s) => s,
        }
    }
}
