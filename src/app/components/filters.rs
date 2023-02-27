use std::fmt::Display;
use std::str::FromStr;

use crossterm::event::KeyCode;
use regex::Regex;
use tui::style::{Color, Style};
use tui::widgets::{BorderType, List, ListItem, ListState};

use crate::tui_widgets::OverlayBlock;

pub struct FilterList {
    pub entries: Vec<Filter>,
    pub visible: bool,
    pub selected: usize,
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

    pub fn widget(&self) -> Option<(OverlayBlock<List>, ListState)> {
        if self.visible {
            let mut list_state = ListState::default();
            list_state.select(Some(self.selected));
            let overlayed_list = OverlayBlock::new(
                //TODO: populate with real filters
                //TODO: add/edit/delete/move support
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
            Some((overlayed_list, list_state))
        } else {
            None
        }
    }

    pub fn on_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Up => self.select_prev(),
            KeyCode::Down => self.select_next(),
            KeyCode::Esc => self.visible = false,
            _ => {}
        }
    }
}

pub struct Filter {
    pub value: Regex,
    pub negate: bool,
    pub active: bool,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.value.as_str())
    }
}

impl FromStr for Filter {
    type Err = regex::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Regex::from_str(s).map(|value| Filter {
            value,
            negate: false,
            active: true,
        })
    }
}
