use std::collections::HashMap;

use regex::Regex;

pub struct Sherlog {
    text_lines: Vec<String>,
}

pub struct FindRange {
    pub start: usize,
    pub end: usize,
}

impl Sherlog {
    pub fn new(text: &str) -> Self {
        Sherlog {
            text_lines: text.lines().map(String::from).collect(),
        }
    }

    pub fn get_lines(&self, first: usize, cnt: Option<usize>) -> Vec<String> {
        self.text_lines
            .iter()
            .skip(first)
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
}
