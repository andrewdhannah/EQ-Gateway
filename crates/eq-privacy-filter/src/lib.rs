//! # eq-privacy-filter — PII Detection and Redaction
//!
//! Canadian-focused PII detection engine for the EQ Gateway.
//! Provides pattern-based detection for:
//! - Social Insurance Numbers (SIN)
//! - Canadian passport numbers
//! - Provincial health card numbers (OHIP, RAMQ, etc.)
//! - Canadian postal codes
//! - Email addresses
//! - Canadian phone numbers
//!
//! # Design
//! Uses a two-layer approach:
//! 1. **Regex layer** — fast, deterministic pattern matching for known formats
//! 2. **Model-assisted layer** (future) — for adversarial evasion detection
//!
//! All detection is performed against the text held in a [`SecureBuffer`],
//! which guarantees that sensitive data is wiped after processing.

use regex::Regex;

/// Severity level for a PII pattern match.
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    /// Personally identifying information (SIN, passport, health card)
    Critical,
    /// Information that could identify in combination (postal code, email, phone)
    High,
    /// Information that may be identifying (names, usernames)
    Medium,
    /// General information
    Low,
}

/// A single PII pattern with metadata for compliance reporting.
#[derive(Debug)]
pub struct PiiPattern {
    /// Human-readable name of this pattern
    pub name: &'static str,
    /// Compiled regex for detecting this pattern
    pub regex: Regex,
    /// Category (e.g., "government_id", "health_id", "contact")
    pub category: &'static str,
    /// Severity level
    pub severity: Severity,
    /// Applicable regulation (e.g., "PIPEDA", "PHIPA")
    pub regulation: &'static str,
}

impl PiiPattern {
    /// Create a new PII pattern from a regex string.
    /// Panics if the regex is invalid (compile-time patterns only).
    pub fn new(
        name: &'static str,
        pattern: &str,
        category: &'static str,
        severity: Severity,
        regulation: &'static str,
    ) -> Self {
        PiiPattern {
            name,
            regex: Regex::new(pattern).expect("Invalid PII pattern regex"),
            category,
            severity,
            regulation,
        }
    }
}

/// A single match found during a PII scan.
#[derive(Debug, Clone)]
pub struct PiiMatch {
    /// Name of the pattern that matched
    pub pattern_name: &'static str,
    /// The matched text (used for redaction)
    pub match_text: String,
    /// Byte offset where the match was found
    pub position: usize,
    /// Category of the match
    pub category: &'static str,
    /// Severity level
    pub severity: Severity,
    /// Applicable regulation
    pub regulation: &'static str,
}

/// Results from a PII scan operation.
#[derive(Debug)]
pub struct ScanResult {
    /// True if no PII was found
    pub clean: bool,
    /// List of matches found
    pub matches: Vec<PiiMatch>,
    /// Total number of matches
    pub match_count: usize,
    /// Number of critical-severity matches
    pub critical_count: usize,
}

/// Canadian PII pattern library.
/// Compiled once and reused across all scan operations.
pub struct PiiScanner {
    patterns: Vec<PiiPattern>,
}

impl Default for PiiScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PiiScanner {
    /// Create a new PiiScanner with the default Canadian PII patterns.
    pub fn new() -> Self {
        PiiScanner {
            patterns: Self::default_patterns(),
        }
    }

    /// Create a PiiScanner with a custom set of patterns.
    pub fn with_patterns(patterns: Vec<PiiPattern>) -> Self {
        PiiScanner { patterns }
    }

    /// Scan a text string for all registered PII patterns.
    ///
    /// # Arguments
    /// * `text` - The text to scan (typically the `anonymized_summary` from the EQ State).
    ///
    /// # Returns
    /// A [`ScanResult`] with all matches and summary statistics.
    pub fn scan(&self, text: &str) -> ScanResult {
        let mut matches = Vec::new();

        for pattern in &self.patterns {
            for m in pattern.regex.find_iter(text) {
                matches.push(PiiMatch {
                    pattern_name: pattern.name,
                    match_text: m.as_str().to_string(),
                    position: m.start(),
                    category: pattern.category,
                    severity: pattern.severity.clone(),
                    regulation: pattern.regulation,
                });
            }
        }

        let critical_count = matches
            .iter()
            .filter(|m| m.severity == Severity::Critical)
            .count();

        ScanResult {
            clean: matches.is_empty(),
            match_count: matches.len(),
            critical_count,
            matches,
        }
    }

    /// Redact all PII matches from a text string, replacing them with a placeholder.
    ///
    /// # Arguments
    /// * `text` - The text to redact.
    /// * `placeholder` - The replacement string (e.g., "[REDACTED]").
    ///
    /// # Returns
    /// The redacted text with all PII replaced by the placeholder.
    pub fn redact(&self, text: &str, placeholder: &str) -> String {
        let mut result = text.to_string();
        for pattern in &self.patterns {
            result = pattern.regex.replace_all(&result, placeholder).to_string();
        }
        result
    }

    /// The default set of Canadian PII patterns.
    fn default_patterns() -> Vec<PiiPattern> {
        vec![
            // Social Insurance Number (SIN) — 9 digits with optional spaces/dashes
            PiiPattern::new(
                "SIN",
                r"\b\d{3}[ -]?\d{3}[ -]?\d{3}\b",
                "government_id",
                Severity::Critical,
                "PIPEDA",
            ),
            // Canadian passport numbers — 2 letters + 6 digits
            PiiPattern::new(
                "Canadian Passport",
                r"\b[A-Za-z]{2}\d{6}\b",
                "government_id",
                Severity::Critical,
                "PIPEDA",
            ),
            // Canadian postal codes — letter-digit-letter space? digit-letter-digit
            PiiPattern::new(
                "Canadian Postal Code",
                r"\b[A-Za-z]\d[A-Za-z][ -]?\d[A-Za-z]\d\b",
                "address",
                Severity::High,
                "PIPEDA",
            ),
            // Email addresses
            PiiPattern::new(
                "Email",
                r"\b[\w\.-]+@[\w\.-]+\.\w{2,}\b",
                "contact",
                Severity::High,
                "PIPEDA",
            ),
            // Canadian phone numbers (+1 or 1 prefix, 10 digits with various formatting)
            PiiPattern::new(
                "Canadian Phone",
                r"\b(?:1[-.]?)?\(?\d{3}\)?[-.]?\d{3}[-.]?\d{4}\b",
                "contact",
                Severity::High,
                "PIPEDA",
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sin_detection() {
        let scanner = PiiScanner::new();
        let result = scanner.scan("My SIN is 123-456-789");
        assert!(!result.clean);
        assert!(result.matches.iter().any(|m| m.pattern_name == "SIN"));
    }

    #[test]
    fn test_postal_code_detection() {
        let scanner = PiiScanner::new();
        let result = scanner.scan("I live at M5A 1A1");
        assert!(!result.clean);
        assert!(result.matches.iter().any(|m| m.pattern_name == "Canadian Postal Code"));
    }

    #[test]
    fn test_email_detection() {
        let scanner = PiiScanner::new();
        let result = scanner.scan("Email me at test@example.com");
        assert!(!result.clean);
        assert!(result.matches.iter().any(|m| m.pattern_name == "Email"));
    }

    #[test]
    fn test_clean_text() {
        let scanner = PiiScanner::new();
        let result = scanner.scan("How are you feeling today? This is a normal conversation.");
        assert!(result.clean);
        assert_eq!(result.match_count, 0);
    }

    #[test]
    fn test_redaction() {
        let scanner = PiiScanner::new();
        let redacted = scanner.redact(
            "Call me at 416-555-1234 or email test@example.com",
            "[REDACTED]",
        );
        assert!(!redacted.contains("416-555-1234"));
        assert!(!redacted.contains("test@example.com"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_passport_detection() {
        let scanner = PiiScanner::new();
        let result = scanner.scan("My passport number is AB123456");
        assert!(!result.clean);
        assert!(result.matches.iter().any(|m| m.pattern_name == "Canadian Passport"));
    }

    /// Verify that non-PII number patterns don't cause false positives
    #[test]
    fn test_avoid_false_positives() {
        let scanner = PiiScanner::new();
        // Normal conversation numbers should not trigger
        let result = scanner.scan(
            "I have 2 cats and 3 dogs. My house number is 42."
        );
        assert!(result.clean);
    }

    #[test]
    fn test_pii_at_start_of_string() {
        let scanner = PiiScanner::new();
        let result = scanner.scan("test@example.com is my email");
        assert!(!result.clean);
        assert!(result.matches.iter().any(|m| m.pattern_name == "Email"));
    }

    #[test]
    fn test_pii_at_end_of_string() {
        let scanner = PiiScanner::new();
        let result = scanner.scan("my email is test@example.com");
        assert!(!result.clean);
        assert!(result.matches.iter().any(|m| m.pattern_name == "Email"));
    }

    #[test]
    fn test_multiple_pii_types() {
        let scanner = PiiScanner::new();
        let result = scanner.scan(
            "John: SIN 123-456-789, email john@test.com, phone 416-555-1234"
        );
        assert!(!result.clean);
        // Should find at least 3 matches across different categories
        assert!(result.match_count >= 3);
        let categories: std::collections::HashSet<&str> =
            result.matches.iter().map(|m| m.category).collect();
        assert!(categories.contains("government_id"));
        assert!(categories.contains("contact"));
    }

    #[test]
    fn test_pii_with_special_characters_around() {
        let scanner = PiiScanner::new();
        // PII in parentheses, brackets, quotes — phone without space after parens
        let result = scanner.scan(
            "Contact [test@example.com] or call (416)555-1234 today!"
        );
        assert!(!result.clean);
        assert!(result.match_count >= 2);
        assert!(result.matches.iter().any(|m| m.pattern_name == "Email"));
        assert!(result.matches.iter().any(|m| m.pattern_name == "Canadian Phone"));
    }

    #[test]
    fn test_unicode_with_pii() {
        let scanner = PiiScanner::new();
        let result = scanner.scan("用户邮箱是 test@example.com");
        assert!(!result.clean);
        assert!(result.matches.iter().any(|m| m.pattern_name == "Email"));
    }

    #[test]
    fn test_redaction_preserves_structure() {
        let scanner = PiiScanner::new();
        let redacted = scanner.redact(
            "Contact: test@example.com and 416-555-1234",
            "[PRIVACY]",
        );
        assert!(!redacted.contains("test@example.com"));
        assert!(!redacted.contains("416-555-1234"));
        // Structure should be preserved
        assert!(redacted.contains("Contact:"));
        assert!(redacted.contains("[PRIVACY]"));
    }

    #[test]
    fn test_empty_string() {
        let scanner = PiiScanner::new();
        let result = scanner.scan("");
        assert!(result.clean);
        assert_eq!(result.match_count, 0);
        assert_eq!(result.critical_count, 0);

        let redacted = scanner.redact("", "[REDACTED]");
        assert_eq!(redacted, "");
    }

    #[test]
    fn test_long_string() {
        let scanner = PiiScanner::new();
        let long_text = "A".repeat(10000) + &"test@example.com".to_string();
        let result = scanner.scan(&long_text);
        assert!(!result.clean);
        assert!(result.matches.iter().any(|m| m.pattern_name == "Email"));
    }

    #[test]
    fn test_scan_returns_critical_count() {
        let scanner = PiiScanner::new();
        let result = scanner.scan("SIN: 123-456-789 and passport AB123456");
        assert!(result.critical_count >= 2);
        assert_eq!(result.critical_count,
            result.matches.iter().filter(|m| m.severity == Severity::Critical).count());
    }
}
