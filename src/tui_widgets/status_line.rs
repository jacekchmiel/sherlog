use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::text::Text;
use tui::widgets::Widget;

use crate::sherlog_tui_app::{Cursor, RenderCursor};

pub struct StatusLine<'a> {
    left_aligned: Vec<Text<'a>>,
    right_aligned: Vec<Text<'a>>,
    cursor: Option<usize>,
}

impl<'a> StatusLine<'a> {
    pub fn new() -> Self {
        StatusLine {
            left_aligned: Default::default(),
            right_aligned: Default::default(),
            cursor: None,
        }
    }

    pub fn left<T: Into<Text<'a>>>(mut self, text: T) -> Self {
        self.left_aligned.push(text.into());
        self
    }

    #[allow(dead_code)] // not used anywhere yet but as useful as right_maybe
    pub fn left_maybe<T: Into<Text<'a>>>(self, text: Option<T>) -> Self {
        match text {
            Some(text) => self.left(text),
            None => self,
        }
    }

    pub fn right<T: Into<Text<'a>>>(mut self, text: T) -> Self {
        self.right_aligned.push(text.into());
        self
    }

    pub fn right_maybe<T: Into<Text<'a>>>(self, text: Option<T>) -> Self {
        match text {
            Some(text) => self.right(text),
            None => self,
        }
    }

    /// Inserts cursor after last left field if show_cursor is true
    pub fn cursor_maybe(mut self, show_cursor: bool) -> Self {
        if show_cursor {
            self.cursor = Some(self.left_aligned.len());
        } else {
            self.cursor = None;
        }
        self
    }
}

impl<'a> Widget for StatusLine<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut cur = area.left();
        for item in self.left_aligned.into_iter() {
            let a = Rect {
                x: area.x + cur,
                width: item.width() as u16,
                ..area
            };

            render_field(item, a, buf);
            cur += a.width + 1;
        }

        let mut cur = 0;
        for item in self.right_aligned {
            let a = Rect {
                x: area.x + area.width - cur - item.width() as u16,
                width: item.width() as u16,
                ..area
            };
            render_field(item, a, buf);
            cur += a.width + 1;
        }
    }
}

impl RenderCursor for StatusLine<'_> {
    fn cursor(&self, area: Rect) -> Option<Cursor> {
        self.cursor.map(|left_cnt| {
            let mut x = self
                .left_aligned
                .iter()
                .take(left_cnt)
                .map(Text::width)
                .sum::<usize>() as u16;
            x += left_cnt.saturating_sub(1) as u16;
            Cursor::new(x, 0).inside(area)
        })
    }
}

fn render_field<'a>(text: Text<'a>, area: Rect, buf: &mut Buffer) {
    if text.lines.get(0).is_none() {
        return;
    }
    let first_line_spans = text.lines.into_iter().next().unwrap().0;
    let chars = first_line_spans
        .iter()
        .flat_map(|s| s.content.as_ref().chars());
    for (i, ch) in chars.take(area.width as usize).enumerate() {
        buf.get_mut(area.x + i as u16, area.y).set_char(ch);
    }
}
