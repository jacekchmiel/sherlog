use regex::Regex;

#[derive(Clone, Debug)]
pub struct RegexFilter {
    pub pattern: Regex,
    pub negate: bool,
}

impl RegexFilter {
    pub fn is_match(&self, line: &str) -> bool {
        self.pattern.is_match(line) ^ self.negate
    }
}

pub struct Sherlog {
    lines: Vec<String>,
    pub filters: Vec<RegexFilter>,
    pub highlight: Option<Regex>,
}

impl Sherlog {
    pub fn new(text: &str) -> Self {
        Sherlog {
            lines: text.lines().map(String::from).collect(),
            filters: Vec::new(),
            highlight: None,
        }
    }

    pub fn get_lines(&self, first: usize, cnt: Option<usize>) -> Vec<TextLineRef> {
        let filtered_lines: Vec<_> = self
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| self.filters.iter().all(|pat| pat.is_match(line)))
            .skip(first)
            .take(cnt.unwrap_or(usize::MAX))
            .collect();

        filtered_lines
            .into_iter()
            .map(|(n, line)| self.make_text_line(n + 1, line).to_owned())
            .collect()
    }

    fn make_text_line<'a>(&'a self, n: usize, line: &'a str) -> TextLineRef<'a> {
        if let Some(pattern) = &self.highlight {
            let mut pos = 0;
            let mut spans = Vec::new();
            for m in pattern.find_iter(line) {
                spans.push(SpanRef::raw(&line[pos..m.start()]));
                spans.push(SpanRef::highlight(&line[m.start()..m.end()]));
                pos = m.end();
            }
            if pos != line.len() {
                spans.push(SpanRef::raw(&line[pos..]));
            }
            TextLineRef { line_num: n, spans }
        } else {
            TextLineRef::raw(n, line)
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

#[derive(Debug, Clone)]
pub struct TextLine {
    pub line_num: usize,
    pub spans: Vec<Span>,
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

#[derive(Debug, Clone)]
pub struct Span {
    pub content: String,
    pub kind: SpanKind,
}

impl Span {
    pub fn remove_left<'a>(&'a self, n: usize) -> SpanRef<'a> {
        SpanRef {
            content: &self.content[n..],
            kind: self.kind,
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

impl<'a> Into<Span> for SpanRef<'a> {
    fn into(self) -> Span {
        self.to_span()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpanKind {
    Raw,
    Highlight,
}
