#[derive(Debug, Clone)]
pub struct Span {
    pub content: String,
    pub kind: SpanKind,
}

impl Span {
    pub fn remove_left(&self, n: usize) -> SpanRef<'_> {
        SpanRef {
            content: &self.content[n..],
            kind: self.kind,
        }
    }
}

impl From<String> for Span {
    fn from(value: String) -> Self {
        Span {
            content: value,
            kind: SpanKind::Raw,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpanRef<'a> {
    pub content: &'a str,
    pub kind: SpanKind,
}

impl<'a> SpanRef<'a> {
    pub fn raw(content: &'a str) -> Self {
        SpanRef {
            content,
            kind: SpanKind::Raw,
        }
    }

    pub fn highlight(content: &'a str) -> Self {
        SpanRef {
            content,
            kind: SpanKind::Highlight,
        }
    }

    pub fn to_span(&self) -> Span {
        Span {
            content: self.content.into(),
            kind: self.kind,
        }
    }
}

impl<'a> From<SpanRef<'a>> for Span {
    fn from(val: SpanRef<'a>) -> Self {
        val.to_span()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpanKind {
    Raw,
    Highlight,
}
