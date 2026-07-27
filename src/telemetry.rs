use regex::Regex;

// patterns that match known telemetry / analytics lines emitted by agents
#[allow(dead_code)]
const PATTERNS: &[&str] = &[
    r"(?i)telemetry",
    r"(?i)sentry\.io",
    r"(?i)amplitude\.com",
    r"(?i)segment\.io",
    r"(?i)statsig",
    r"(?i)posthog",
    r"(?i)mixpanel",
    r"(?i)datadog",
    r"(?i)anonymous[_\s]id",
    r"(?i)tracking[_\s]id",
    r"(?i)opted[_\s]in",
    r"(?i)usage[_\s]stats",
];

#[allow(dead_code)]
pub struct Filter {
    regexes: Vec<Regex>,
}

#[allow(dead_code)]
impl Filter {
    pub fn new() -> Self {
        let regexes = PATTERNS
            .iter()
            .map(|p| Regex::new(p).unwrap())
            .collect();
        Filter { regexes }
    }

    // returns None if the line should be suppressed
    pub fn check<'a>(&self, line: &'a str) -> Option<&'a str> {
        for re in &self.regexes {
            if re.is_match(line) {
                return None;
            }
        }
        Some(line)
    }
}
