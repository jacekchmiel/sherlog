mod ty;

use std::collections::{BTreeMap, BTreeSet};

use log::debug;
pub use regex::Regex;
pub use ty::filter::RegexFilter;
pub use ty::span::{SpanKind, SpanRef};
pub use ty::text::{TextLine, TextLineRef};

pub struct Sherlog {
    lines: Vec<String>,
    filters: Vec<RegexFilter>,
    highlight: Option<Regex>,
    index_filtered: BTreeSet<usize>,
    index_search: BTreeMap<usize, Vec<(u32, u32)>>,
}

impl Sherlog {
    pub fn new(text: &str) -> Self {
        let lines: Vec<_> = text.lines().map(String::from).collect();
        let index_filtered = (0..lines.len()).collect();
        Sherlog {
            lines,
            filters: Vec::new(),
            highlight: None,
            index_filtered,
            index_search: BTreeMap::new(),
        }
    }

    pub fn filter(&mut self, filters: Vec<RegexFilter>) {
        self.filters = filters;
        let filtered_lines: BTreeSet<_> = self
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| self.filters.iter().all(|pat| pat.is_match(line)))
            .map(|(n, _)| n)
            .collect();
        self.index_filtered = filtered_lines;
    }

    pub fn search(&mut self, pattern: Option<Regex>) {
        match pattern {
            Some(pattern) => self.do_search(pattern),
            None => {
                self.index_search = BTreeMap::new();
                self.highlight = None;
            }
        }
    }

    fn do_search(&mut self, pattern: Regex) {
        self.index_search = BTreeMap::new();
        // Builds search results index
        for (n, line) in self.lines.iter().enumerate() {
            for found in pattern.find_iter(line) {
                self.index_search
                    .entry(n)
                    .or_default()
                    .push((found.start() as u32, found.end() as u32))
            }
        }
        // For now we force highlight to search pattern as current API cannot handle overlapping search result
        // and highlight. And we do want to mark search spans.
        self.highlight = Some(pattern);
    }

    //TODO: consider changing api to iterator
    pub fn next_search_result(&self, start: usize) -> Option<usize> {
        self.index_search
            .range(start..)
            // TODO: this find will be inefficient when there is a lot of search matching lines but not a lot filtered
            .find(|i| self.index_filtered.contains(i.0))
            .map(|i| *i.0)
    }

    pub fn prev_search_result(&self, start: usize) -> Option<usize> {
        self.index_search
            .range(..start + 1)
            .rev()
            // TODO: this find will be inefficient when there is a lot of search matching lines but not a lot filtered
            .find(|i| self.index_filtered.contains(i.0))
            .map(|i| *i.0)
    }

    pub fn highlight(&mut self, highlight: Option<Regex>) {
        self.highlight = highlight
    }

    pub fn get_lines(&self, first: usize, cnt: Option<usize>) -> Vec<TextLineRef> {
        debug!("get_lines - first: {first} cnt: {cnt:?}");
        let lines: Vec<_> = self
            .index_filtered
            .range(first..)
            .take(cnt.unwrap_or(usize::MAX))
            .filter_map(|n| self.lines.get(*n).map(|line| self.make_text_line(*n, line)))
            .collect();

        log_returned_lines("get_lines", lines.as_slice());

        lines
    }

    pub fn get_lines_rev(&self, last: usize, cnt: Option<usize>) -> Vec<TextLineRef> {
        debug!("get_lines_rev - last: {last} cnt: {cnt:?}");
        // Seems that we cannot get last cnt elements from BTreeSet::Range.
        // We need to double reverse and to do so we need to store intermediate processed data.
        let reversed: Vec<_> = self
            .index_filtered
            .range(..last.saturating_add(1))
            .rev()
            .take(cnt.unwrap_or(usize::MAX))
            .collect();

        let lines: Vec<_> = reversed
            .into_iter()
            .rev()
            .filter_map(|n| self.lines.get(*n).map(|line| self.make_text_line(*n, line)))
            .collect();

        log_returned_lines("get_lines_rev", lines.as_slice());
        lines
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

fn log_returned_lines(func: &str, lines: &[TextLineRef<'_>]) {
    match lines {
        [single] => debug!("{func} - return single line {}", single.line_num),
        [first, .., last] => {
            debug!(
                "{func} - return lines {} - {}",
                first.line_num, last.line_num
            )
        }
        [] => debug!("{func} - return empty"),
    }
}
#[cfg(test)]
mod test {
    use super::*;

    fn as_strings(lines: Vec<TextLineRef>) -> Vec<String> {
        lines.into_iter().map(|l| l.to_string()).collect()
    }

    #[test]
    fn can_provide_all_lines() {
        let data = "line1\nline2\nline3\n";
        let sherlog = Sherlog::new(data);
        assert_eq!(
            as_strings(sherlog.get_lines(0, None)),
            vec![
                String::from("line1"),
                String::from("line2"),
                String::from("line3")
            ]
        );
    }

    #[test]
    fn can_scroll_filtered_lines() {
        let data = "line1\nline2a\nline2b\nline3\nline2c\nline3\n";
        let mut sherlog = Sherlog::new(data);
        sherlog.filter(vec!["line2".try_into().unwrap()]);

        assert_eq!(
            as_strings(sherlog.get_lines(0, None)),
            vec![
                String::from("line2a"),
                String::from("line2b"),
                String::from("line2c"),
            ]
        );

        assert_eq!(
            as_strings(sherlog.get_lines(0, Some(1))),
            vec![String::from("line2a")]
        );

        assert_eq!(
            as_strings(sherlog.get_lines(0, Some(2))),
            vec![String::from("line2a"), String::from("line2b"),]
        );

        assert_eq!(
            as_strings(sherlog.get_lines(0, Some(3))),
            vec![
                String::from("line2a"),
                String::from("line2b"),
                String::from("line2c")
            ]
        );

        assert_eq!(
            as_strings(sherlog.get_lines(2, Some(1))),
            vec![String::from("line2b")]
        );

        assert_eq!(
            as_strings(sherlog.get_lines(2, Some(2))),
            vec![String::from("line2b"), String::from("line2c")]
        );

        assert_eq!(
            as_strings(sherlog.get_lines(2, None)),
            vec![String::from("line2b"), String::from("line2c")]
        );

        assert_eq!(
            as_strings(sherlog.get_lines(4, None)),
            vec![String::from("line2c")]
        );

        assert_eq!(as_strings(sherlog.get_lines(5, None)), Vec::<String>::new());
    }

    #[test]
    fn can_filter() {
        let data = "line1\nline2\nline3\n";
        let mut sherlog = Sherlog::new(data);
        sherlog.filter(vec!["line2".try_into().unwrap()]);
        assert_eq!(
            as_strings(sherlog.get_lines(0, None)),
            vec![String::from("line2")]
        );
    }

    #[test]
    fn can_search() {
        let data = "line1\nline2\nline3\n";
        let mut sherlog = Sherlog::new(data);
        sherlog.search(Some(Regex::new("line2").unwrap()));
        assert_eq!(sherlog.next_search_result(0), Some(1));
        assert_eq!(sherlog.next_search_result(1), Some(1));
        assert_eq!(sherlog.next_search_result(2), None);
        assert_eq!(sherlog.next_search_result(3), None);

        assert_eq!(sherlog.prev_search_result(3), Some(1));
        assert_eq!(sherlog.prev_search_result(2), Some(1));
        assert_eq!(sherlog.prev_search_result(1), Some(1));
        assert_eq!(sherlog.prev_search_result(0), None);
    }

    #[test]
    fn can_search_filtered() {
        let data = "line1\nline2\nline3\n";
        let mut sherlog = Sherlog::new(data);
        sherlog.filter(vec!["line2|3".try_into().unwrap()]);
        sherlog.search(Some(Regex::new("line").unwrap()));

        assert_eq!(sherlog.next_search_result(0), Some(1));
        assert_eq!(sherlog.next_search_result(1), Some(1));
        assert_eq!(sherlog.next_search_result(2), Some(2));
        assert_eq!(sherlog.next_search_result(3), None);

        assert_eq!(sherlog.prev_search_result(3), Some(2));
        assert_eq!(sherlog.prev_search_result(2), Some(2));
        assert_eq!(sherlog.prev_search_result(1), Some(1));
        assert_eq!(sherlog.prev_search_result(0), None);
    }
}
