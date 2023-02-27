use tui::style::{Color, Style};
use tui::text::Spans;
use tui::widgets::{Paragraph, Wrap};

use crate::sherlog_core::{self, TextLine};

pub struct TextArea {
    pub x: usize,
    pub height: u16,
    pub wrap: bool,
    pub lines: Vec<TextLine>,
}

impl TextArea {
    pub fn widget(&self) -> Paragraph {
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

    fn make_spans<'a>(line: &'a TextLine, offset: usize) -> tui::text::Spans<'a> {
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
            .map(|s| Self::make_span(s))
            .collect::<Vec<_>>()
            .into()
    }

    fn make_span<'a>(span: sherlog_core::SpanRef<'a>) -> tui::text::Span<'a> {
        match span.kind {
            sherlog_core::SpanKind::Raw => tui::text::Span::raw(span.content),
            sherlog_core::SpanKind::Highlight => {
                tui::text::Span::styled(span.content, Style::default().fg(Color::Red))
            }
        }
    }
}
