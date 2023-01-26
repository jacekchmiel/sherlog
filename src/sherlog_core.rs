pub struct Sherlog {
    text_lines: Vec<String>,
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
}
