use regex::Regex;

pub struct Sherlog {
    lines: Vec<String>,
    pub filter: Option<Regex>,
    pub highlight: Option<Regex>,
}

#[derive(Debug, Clone)]
pub struct TextLine<'a> {
    pub spans: Vec<Span<'a>>,
}

impl<'a> TextLine<'a> {
    pub fn new(spans: Vec<Span<'a>>) -> Self {
        TextLine { spans }
    }

    pub fn raw(text: &'a str) -> Self {
        TextLine {
            spans: vec![Span::raw(text)],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Span<'a> {
    pub content: &'a str,
    pub kind: SpanKind,
}

impl<'a> Span<'a> {
    pub fn raw(content: &'a str) -> Self {
        Span {
            content,
            kind: SpanKind::Raw,
        }
    }

    pub fn highlight(content: &'a str) -> Self {
        Span {
            content,
            kind: SpanKind::Highlight,
        }
    }

    pub fn remove_left(&self, n: usize) -> Self {
        Span {
            content: &self.content[n..],
            kind: self.kind,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpanKind {
    Raw,
    Highlight,
}

impl Sherlog {
    pub fn new(text: &str) -> Self {
        Sherlog {
            lines: text.lines().map(String::from).collect(),
            filter: None,
            highlight: None,
        }
    }

    pub fn get_lines(&self, first: usize, cnt: Option<usize>) -> Vec<TextLine> {
        let filtered_lines: Vec<_> = self
            .lines
            .iter()
            .skip(first)
            .filter(|line| {
                self.filter
                    .as_ref()
                    .map(|pat| pat.is_match(line))
                    .unwrap_or(true)
            })
            .take(cnt.unwrap_or(usize::MAX))
            .collect();

        filtered_lines
            .into_iter()
            .map(|line| self.make_text_line(line))
            .collect()
    }

    fn make_text_line<'a>(&'a self, line: &'a str) -> TextLine<'a> {
        if let Some(pattern) = &self.highlight {
            let mut pos = 0;
            let mut spans = Vec::new();
            for m in pattern.find_iter(line) {
                spans.push(Span::raw(&line[pos..m.start()]));
                spans.push(Span::highlight(&line[m.start()..m.end()]));
                pos = m.end();
            }
            if pos != line.len() {
                spans.push(Span::raw(&line[pos..]));
            }
            TextLine::new(spans)
        } else {
            TextLine::raw(line)
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}
