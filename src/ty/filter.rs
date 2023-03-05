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

impl From<Regex> for RegexFilter {
    fn from(pattern: Regex) -> Self {
        RegexFilter {
            pattern,
            negate: false,
        }
    }
}
