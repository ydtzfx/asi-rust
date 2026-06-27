use regex::Regex;
use std::sync::LazyLock;

/// Pattern definitions for 7 categories of prompt injection attacks.
static PATTERNS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    vec![
        (
            "ignore_previous",
            Regex::new(r"(?i)(ignore|disregard|forget|bypass)\s+(all\s+)?(previous|above|prior)\s+(instructions|commands|directives|rules)").unwrap(),
        ),
        (
            "system_prompt_leakage",
            Regex::new(r"(?i)(reveal|show|print|output|leak|display)\s+(your\s+)?(system\s+)?prompt|what\s+(are|were|is)\s+your\s+(instructions|system\s+prompt|rules)").unwrap(),
        ),
        (
            "role_playing",
            Regex::new(r"(?i)(now\s+)?(act\s+as|pretend|roleplay|you.+(are|now)|from\s+now\s+on)\s+.{0,40}(unrestricted|unfiltered|free|jailbreak|DAN|hypothetical|character|mode)").unwrap(),
        ),
        (
            "dan_mode",
            Regex::new(r"(?i)\bDAN\b|do\s+anything\s+now|no\s+(restrictions|limits|boundaries|filter)").unwrap(),
        ),
        (
            "template_injection",
            Regex::new(r"(?i)\{\{.*\}\}|<%.*%>|#\{.*\}|\$\[\s*exec|\$\{.*\}").unwrap(),
        ),
        (
            "indirect_injection",
            Regex::new(r"(?i)(user\s+said|message\s+contains|context\s+includes|from\s+the\s+user).{0,20}(ignore|forget|override|disregard)").unwrap(),
        ),
        (
            "token_smuggling",
            Regex::new(r"(?i)(base64|rot13|caesar|cipher|encoded|obfuscated).{0,30}(instruction|prompt|command|tell|say)").unwrap(),
        ),
    ]
});

/// Detect prompt injection patterns in the given text.
/// Returns a list of matched attack category names.
pub fn detect_prompt_injection(text: &str) -> Vec<String> {
    PATTERNS
        .iter()
        .filter(|(_, re)| re.is_match(text))
        .map(|(name, _)| name.to_string())
        .collect()
}
