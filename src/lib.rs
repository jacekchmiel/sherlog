mod ty;

pub use regex::Regex;
pub use ty::filter::RegexFilter;
pub use ty::span::{SpanKind, SpanRef};
pub use ty::text::{TextLine, TextLineRef};

pub struct Sherlog {
    lines: Vec<String>,
    filters: Vec<RegexFilter>,
    highlight: Option<Regex>,
    index_filtered: LineIndex,
}

impl Sherlog {
    pub fn new(text: &str) -> Self {
        let lines: Vec<_> = text.lines().map(String::from).collect();
        let index_filtered = LineIndex((0..lines.len()).collect());
        Sherlog {
            lines,
            filters: Vec::new(),
            highlight: None,
            index_filtered,
        }
    }

    pub fn filter(&mut self, filters: Vec<RegexFilter>) {
        self.filters = filters;
        let filtered_lines: Vec<_> = self
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| self.filters.iter().all(|pat| pat.is_match(line)))
            .map(|(n, _)| n)
            .collect();
        self.index_filtered = LineIndex(filtered_lines);
    }

    pub fn highlight(&mut self, highlight: Option<Regex>) {
        self.highlight = highlight
    }

    pub fn get_lines(&self, first: usize, cnt: Option<usize>) -> Vec<TextLineRef> {
        self.index_filtered
            .0
            .iter()
            .skip(first)
            .take(cnt.unwrap_or(usize::MAX))
            .filter_map(|n| {
                self.lines
                    .get(*n)
                    .map(|line| self.make_text_line(n + 1, line))
            })
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

struct LineIndex(Vec<usize>);

#[cfg(test)]
mod test {
    use super::*;

    fn as_strings(lines: Vec<TextLineRef>) -> Vec<String> {
        lines.into_iter().map(|l| l.to_string()).collect()
    }

    #[test]
    fn provides_lines() {
        let data = "line1\nline2\n";
        let sherlog = Sherlog::new(data);
        assert_eq!(
            as_strings(sherlog.get_lines(0, None)),
            vec![String::from("line1"), String::from("line2")]
        );
    }
}
