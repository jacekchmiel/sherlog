use sherlog::{SpanKind, SpanRef, TextLine};
use tui::style::{Color, Style};
use tui::text::Spans;
use tui::widgets::{Paragraph, Wrap};

use crate::ty::Render;

pub(crate) struct TextArea {
    pub x: usize,
    pub wrap: bool,
    pub lines: Vec<TextLine>,
}

impl TextArea {
    pub fn new() -> Self {
        TextArea {
            x: 0,
            wrap: false,
            lines: vec![],
        }
    }

    pub fn scroll_left(&mut self) {
        self.x = self.x.saturating_sub(1);
    }

    pub fn scroll_right(&mut self) {
        self.x = self.x.saturating_add(1);
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

    pub fn first_line(&self) -> Option<&TextLine> {
        self.lines.first()
    }

    pub fn last_line(&self) -> Option<&TextLine> {
        self.lines.last()
    }
}

impl Render for TextArea {
    type Widget<'a> = Paragraph<'a>;

    fn widget(&self) -> Paragraph<'_> {
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
