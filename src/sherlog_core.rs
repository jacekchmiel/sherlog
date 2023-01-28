use std::collections::HashMap;

use regex::Regex;

pub struct Sherlog {
    text_lines: Vec<String>,
    filter: Option<Regex>,
}

pub struct FindRange {
    pub start: usize,
    pub end: usize,
}

impl Sherlog {
    pub fn new(text: &str) -> Self {
        Sherlog {
            text_lines: text.lines().map(String::from).collect(),
            filter: None,
        }
    }

    pub fn get_lines(&self, first: usize, cnt: Option<usize>) -> Vec<String> {
        self.text_lines
            .iter()
            .skip(first)
            .filter(|line| {
                self.filter
                    .as_ref()
                    .map(|pat| pat.is_match(line))
                    .unwrap_or(true)
            })
            .take(cnt.unwrap_or(usize::MAX))
            .cloned()
            .collect()
    }

    pub fn line_count(&self) -> usize {
        self.text_lines.len()
    }

    pub fn find<'r>(
        &self,
        pattern: &'r Regex,
        first: usize,
        cnt: Option<usize>,
    ) -> HashMap<usize, Vec<FindRange>> {
        self.text_lines
            .iter()
            .enumerate()
            .skip(first)
            .take(cnt.unwrap_or(usize::MAX))
            .map(|(n, line)| {
                (
                    n,
                    pattern
                        .find_iter(line)
                        .map(move |m| FindRange {
                            start: m.start(),
                            end: m.end(),
                        })
                        .collect(),
                )
            })
            .collect()
    }

    pub fn set_filter(&mut self, filter: Regex) {
        self.filter = Some(filter)
    }

    pub fn remove_filter(&mut self) {
        self.filter = None
    }
}
