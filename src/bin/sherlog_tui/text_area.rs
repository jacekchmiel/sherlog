use tui::style::{Color, Style};
use tui::text::Spans;
use tui::widgets::{Paragraph, Wrap};

use crate::ty::Render;
use sherlog::{self, SpanKind, SpanRef, TextLine};

pub(crate) struct TextArea {
    pub x: usize,
    pub y: usize,
    pub max_y: usize,
    pub height: u16, // Shouldn't we get this as render feedback?
    pub wrap: bool,
    pub lines: Vec<TextLine>,
}

impl TextArea {
    pub fn new(height: u16, max_y: usize) -> Self {
        TextArea {
            x: 0,
            y: 0,
            max_y,
            height,
            wrap: false,
            lines: vec![],
        }
    }

    pub fn scroll_up(&mut self, line_cnt: usize) -> bool {
        let old_y = self.y;
        self.y = self.y.saturating_sub(line_cnt);
        old_y != self.y
    }

    pub fn scroll_down(&mut self, line_cnt: usize) -> bool {
        let old_y = self.y;
        self.y = self.y.saturating_add(line_cnt);
        if self.y > self.max_y {
            self.y = self.max_y;
        }
        old_y != self.y
    }

    pub fn scroll_left(&mut self) -> bool {
        let old_x = self.x;
        self.x = self.x.saturating_sub(1);
        old_x != self.x
    }

    pub fn scroll_right(&mut self) -> bool {
        let old_x = self.x;
        self.x = self.x.saturating_add(1);
        old_x != self.x
    }

    pub fn go_top(&mut self) -> bool {
        let old_y = self.y;
        self.y = 0;
        old_y != self.y
    }

    pub fn go_bottom(&mut self) -> bool {
        let old_y = self.y;
        self.y = self.max_y;
        old_y != self.y
    }

    fn make_spans(line: &TextLine, offset: usize) -> tui::text::Spans<'_> {
        let mut chars_to_remove = offset;
        let spans = line.spans.iter();
        spans
            .filter_map(|s| {
                if chars_to_remove >= s.content.len() {
                    chars_to_remove -= s.content.len();
                    None
                } else {
                    let remaining = s.remove_left(chars_to_remove);
                    chars_to_remove = 0;
                    Some(remaining)
                }
            })
            .map(Self::make_span)
            .collect::<Vec<_>>()
            .into()
    }

    fn make_span(span: SpanRef<'_>) -> tui::text::Span {
        match span.kind {
            SpanKind::Raw => tui::text::Span::raw(span.content),
            SpanKind::Highlight => {
                tui::text::Span::styled(span.content, Style::default().fg(Color::Red))
            }
        }
    }

    pub fn toggle_wrap(&mut self) -> bool {
        self.wrap = !self.wrap;
        self.wrap
    }
}

impl<'a> Render<'a> for TextArea {
    type Widget = Paragraph<'a>;

    fn widget(&'a self) -> Paragraph<'a> {
        let spans: Vec<Spans> = self
            .lines
            .iter()
            .map(|line| Self::make_spans(line, self.x))
            .collect();
        let mut paragraph = Paragraph::new(spans);
        if self.wrap {
            paragraph = paragraph.wrap(Wrap { trim: false })
        }
        paragraph
    }
}
