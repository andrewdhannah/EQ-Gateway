//! # eq-state-compiler — EQ State JSON Construction
//!
//! Assembles the validated EQ State payload from the outputs of:
//! - Affect classification (primary emotion, valence, arousal)
//! - Intent classification (category, subtype)
//! - Risk assessment (level, signals)
//! - Privacy filter (sensitivity, PII status)
//! - Response policy (tone, warmth, directness, length)
//! - Context summary (anonymized text)
//!
//! All outputs are structured to match the TypeScript Zod schema
//! defined in `schemas/eq-state.schema.ts`.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Controlled Vocabularies (mirrors TypeScript enums)
// ---------------------------------------------------------------------------

/// Valid primary emotional descriptors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AffectPrimary {
    Neutral,
    Calm,
    Curious,
    Pleased,
    Hopeful,
    Confused,
    Uncertain,
    Frustrated,
    Angry,
    Sad,
    Anxious,
    Overwhelmed,
    Fatigued,
    Embarrassed,
    Lonely,
    Excited,
    Urgent,
    Unknown,
}

impl std::fmt::Display for AffectPrimary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{:?}", self));
        write!(f, "{}", s)
    }
}

/// High-level classification of user intent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IntentCategory {
    PracticalGuidance,
    EmotionalSupport,
    DecisionSupport,
    Venting,
    Planning,
    Clarification,
    ConflictNavigation,
    Reflection,
    TaskExecution,
    CreativeHelp,
    TechnicalHelp,
    SafetyRelated,
    Unknown,
}

/// Safety urgency levels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Crisis,
    Unknown,
}

/// Privacy sensitivity threshold.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PrivacySensitivity {
    Public,
    Low,
    Medium,
    High,
    Restricted,
    Unknown,
}

/// Stylistic instructions for the Large AI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseTone {
    CalmDirect,
    GentleDirect,
    WarmBrief,
    NeutralProfessional,
    HighlyPractical,
    Reflective,
    Encouraging,
    Minimal,
    Unknown,
}

impl std::fmt::Display for ResponseTone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{:?}", self));
        write!(f, "{}", s)
    }
}

// ---------------------------------------------------------------------------
// Sub-State Structs (mirrors TypeScript sub-schemas)
// ---------------------------------------------------------------------------

/// Quantitative and qualitative measures of current emotion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectState {
    /// The dominant emotion label
    pub primary: AffectPrimary,
    /// Nuanced secondary emotions detected in the text
    #[serde(default)]
    pub secondary: Vec<AffectPrimary>,
    /// Valence score: -1.0 (extreme negative) to 1.0 (extreme positive)
    pub valence: f64,
    /// Arousal score: 0.0 (low energy/calm) to 1.0 (high energy/excited)
    pub arousal: f64,
    /// Model's confidence in affect classification (0.0 to 1.0)
    pub confidence: f64,
    /// How the signal was derived (e.g., "semantic_inference")
    pub evidence_type: String,
}

/// Classification of user intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentState {
    /// The broad category of intent
    pub category: IntentCategory,
    /// A more granular description of the user's specific goal
    pub subtype: String,
    /// Model's confidence in intent classification (0.0 to 1.0)
    pub confidence: f64,
}

/// Safety assessment for escalation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskState {
    /// Current urgency/safety level
    pub level: RiskLevel,
    /// Specific signals triggering this level
    #[serde(default)]
    pub signals: Vec<String>,
    /// Model's confidence in risk assessment (0.0 to 1.0)
    pub confidence: f64,
    /// Whether system must bypass cloud calls for safety
    pub requires_local_escalation: bool,
}

/// Metadata regarding data sanitization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyState {
    /// Determined sensitivity level
    pub sensitivity_level: PrivacySensitivity,
    /// Whether raw unredacted text is being sent to cloud
    pub raw_text_shared: bool,
    /// Confirmation that PII has been removed
    pub pii_removed: bool,
    /// List of sensitive domains detected
    #[serde(default)]
    pub sensitive_domains_detected: Vec<String>,
    /// Model's confidence in redaction accuracy (0.0 to 1.0)
    pub redaction_confidence: f64,
}

/// Behavioral constraints for the Large AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePolicy {
    /// Desired tone of the response
    pub tone: ResponseTone,
    /// Degree of warmth/empathy (0.0 to 1.0)
    pub warmth: f64,
    /// Degree of directness/brevity (0.0 to 1.0)
    pub directness: f64,
    /// Target response length
    pub length: String,
    /// Expected conversational pace
    #[serde(default = "default_pace")]
    pub pace: String,
    /// Hard limit on follow-up questions per turn
    #[serde(default = "default_max_followup")]
    pub max_followup_questions: u32,
    /// Preferred response format
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_pace() -> String { "steady".to_string() }
fn default_max_followup() -> u32 { 2 }
fn default_format() -> String { "prose".to_string() }

/// Sanitized, anonymized summary of the dialogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextState {
    /// Privacy-safe text summary of the dialogue
    pub anonymized_summary: String,
    /// Whether a specific text excerpt was approved for sharing
    pub included_raw_excerpt: bool,
    /// Whether retrieval notes are included
    pub retrieval_notes_included: bool,
}

/// Session metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// UUID v4 for this ephemeral session
    pub ephemeral_session_id: String,
    /// ISO-8601 timestamp
    pub timestamp_local: String,
    /// True if Tier 0 (fully local)
    pub device_processing_only: bool,
}

// ---------------------------------------------------------------------------
// Master EQ State (mirrors EQStateSchema from TypeScript)
// ---------------------------------------------------------------------------

/// The complete EQ State payload.
/// This is the validated JSON object that crosses the MCP bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EQState {
    /// Schema version for forward compatibility
    pub schema_version: String,
    /// Session metadata
    pub session: SessionInfo,
    /// Emotional dimension
    pub affect: AffectState,
    /// Goal-oriented dimension
    pub intent: IntentState,
    /// Safety dimension
    pub risk: RiskState,
    /// Privacy and sanitization dimension
    pub privacy: PrivacyState,
    /// Behavioral dimension
    pub response_policy: ResponsePolicy,
    /// Contextual dimension
    pub context: ContextState,
}

impl EQState {
    /// Serialize the EQ State to a JSON string.
    /// Returns `None` if serialization fails.
    pub fn to_json(&self) -> Option<String> {
        serde_json::to_string_pretty(self).ok()
    }

    /// Serialize the EQ State to a compact JSON string (no whitespace).
    /// Returns `None` if serialization fails.
    pub fn to_json_compact(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }

    /// Deserialize an EQ State from a JSON string.
    /// Returns `None` if deserialization fails.
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_eq_state() -> EQState {
        EQState {
            schema_version: "0.1".to_string(),
            session: SessionInfo {
                ephemeral_session_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                timestamp_local: "2026-06-08T12:00:00Z".to_string(),
                device_processing_only: false,
            },
            affect: AffectState {
                primary: AffectPrimary::Frustrated,
                secondary: vec![AffectPrimary::Confused],
                valence: -0.4,
                arousal: 0.7,
                confidence: 0.85,
                evidence_type: "semantic_inference".to_string(),
            },
            intent: IntentState {
                category: IntentCategory::EmotionalSupport,
                subtype: "Seeking validation".to_string(),
                confidence: 0.76,
            },
            risk: RiskState {
                level: RiskLevel::Low,
                signals: vec!["frustration".to_string()],
                confidence: 0.62,
                requires_local_escalation: false,
            },
            privacy: PrivacyState {
                sensitivity_level: PrivacySensitivity::Medium,
                raw_text_shared: false,
                pii_removed: true,
                sensitive_domains_detected: vec!["workplace".to_string()],
                redaction_confidence: 0.95,
            },
            response_policy: ResponsePolicy {
                tone: ResponseTone::GentleDirect,
                warmth: 0.7,
                directness: 0.4,
                length: "medium".to_string(),
                pace: "steady".to_string(),
                max_followup_questions: 2,
                format: "prose".to_string(),
            },
            context: ContextState {
                anonymized_summary: "User is expressing frustration about a workplace situation.".to_string(),
                included_raw_excerpt: false,
                retrieval_notes_included: true,
            },
        }
    }

    #[test]
    fn test_serialize_deserialize() {
        let state = sample_eq_state();
        let json = state.to_json().expect("Serialization failed");
        let deserialized = EQState::from_json(&json).expect("Deserialization failed");

        assert_eq!(deserialized.schema_version, "0.1");
        assert_eq!(deserialized.affect.primary, AffectPrimary::Frustrated);
        assert_eq!(deserialized.intent.category, IntentCategory::EmotionalSupport);
        assert_eq!(deserialized.risk.level, RiskLevel::Low);
    }

    #[test]
    fn test_json_output_format() {
        let state = sample_eq_state();
        let json = state.to_json_compact().expect("Serialization failed");

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        // Verify required fields exist
        assert!(parsed.get("schema_version").is_some());
        assert!(parsed.get("session").is_some());
        assert!(parsed.get("affect").is_some());
        assert!(parsed.get("intent").is_some());
        assert!(parsed.get("risk").is_some());
        assert!(parsed.get("privacy").is_some());
        assert!(parsed.get("response_policy").is_some());
        assert!(parsed.get("context").is_some());
    }

    #[test]
    fn test_affect_display() {
        assert_eq!(AffectPrimary::Frustrated.to_string(), "frustrated");
        assert_eq!(AffectPrimary::Unknown.to_string(), "unknown");
    }
}
