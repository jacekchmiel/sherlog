use std::fmt::Display;

use super::span::{Span, SpanRef};

#[derive(Debug, Clone)]
pub struct TextLine {
    pub line_num: usize,
    pub spans: Vec<Span>,
}

impl From<(usize, String)> for TextLine {
    fn from(value: (usize, String)) -> Self {
        TextLine {
            line_num: value.0,
            spans: vec![value.1.into()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextLineRef<'a> {
    pub line_num: usize,
    pub spans: Vec<SpanRef<'a>>,
}

impl<'a> TextLineRef<'a> {
    pub fn raw(line_num: usize, text: &'a str) -> Self {
        TextLineRef {
            line_num,
            spans: vec![SpanRef::raw(text)],
        }
    }

    pub fn to_text_line(&self) -> TextLine {
        TextLine {
            line_num: self.line_num,
            spans: self.spans.iter().map(SpanRef::to_span).collect(),
        }
    }
}

impl Display for TextLineRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for span in self.spans.iter() {
            f.write_str(span.content)?;
        }
        Ok(())
    }
}
