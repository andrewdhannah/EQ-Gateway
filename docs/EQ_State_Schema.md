# 04 — EQ State Schema and MCP Tool Contract

**Project:** EQ Gateway  
**Status:** Draft v0.1  
**Date:** May 21, 2026

---

## 1. Purpose

This document defines the structured output contract for the local EQ Engine and the MCP tools that expose privacy-safe emotional state to a larger AI model.

The schema must be:

```text
strict
compact
versioned
auditable
privacy-aware
model-agnostic
MCP-friendly
safe under partial failure
```

The local SLM should output state, not prose.

---

## 2. Design Principles

```text
Prefer "unknown" over guessing.
Return confidence scores.
Never include hidden reasoning.
Never include raw PII by default.
Separate affect, intent, risk, privacy, and response policy.
Make every cloud-bound field auditable.
Keep output small enough for mobile and MCP use.
```

---

## 3. EQ State Object

## 3.1 Full Draft Schema Example

```json
{
  "schema_version": "0.1",
  "session": {
    "ephemeral_session_id": "uuid",
    "timestamp_local": "2026-05-21T10:00:00-04:00",
    "device_processing_only": true
  },
  "affect": {
    "primary": "frustrated",
    "secondary": ["fatigued", "uncertain"],
    "valence": -0.42,
    "arousal": 0.61,
    "confidence": 0.78,
    "evidence_type": "semantic_inference_only"
  },
  "intent": {
    "category": "decision_support",
    "subtype": "workplace_response",
    "confidence": 0.84
  },
  "support_need": {
    "primary": "problem_solving",
    "secondary": "emotional_grounding",
    "avoid": ["over_reassurance", "moralizing", "long_lecture"]
  },
  "risk": {
    "level": "none",
    "signals": [],
    "confidence": 0.91,
    "requires_local_escalation": false
  },
  "privacy": {
    "sensitivity_level": "medium",
    "raw_text_shared": false,
    "pii_removed": true,
    "sensitive_domains_detected": ["workplace"],
    "redaction_confidence": 0.86
  },
  "response_policy": {
    "tone": "calm_direct",
    "warmth": "moderate",
    "directness": "high",
    "length": "medium_short",
    "pace": "steady",
    "max_followup_questions": 1,
    "format": "clear_steps"
  },
  "context": {
    "anonymized_summary": "User is frustrated about a workplace situation and wants practical help deciding how to respond. Specific identifiers removed.",
    "included_raw_excerpt": false,
    "retrieval_notes_included": false
  }
}
```

---

## 4. Required Top-Level Fields

```text
schema_version
session
affect
intent
risk
privacy
response_policy
context
```

Optional:

```text
support_need
debug
evaluation_tags
```

Debug fields must never leave the device in production mode.

---

## 5. Controlled Vocabularies

## 5.1 Affect Primary

```text
neutral
calm
curious
pleased
hopeful
confused
uncertain
frustrated
angry
sad
anxious
overwhelmed
fatigued
embarrassed
lonely
excited
urgent
unknown
```

## 5.2 Intent Category

```text
practical_guidance
emotional_support
decision_support
venting
planning
clarification
conflict_navigation
reflection
task_execution
creative_help
technical_help
safety_related
unknown
```

## 5.3 Risk Level

```text
none
low
medium
high
crisis
unknown
```

## 5.4 Privacy Sensitivity

```text
public
low
medium
high
restricted
unknown
```

## 5.5 Response Tone

```text
calm_direct
gentle_direct
warm_brief
neutral_professional
highly_practical
reflective
encouraging
minimal
unknown
```

## 5.6 Response Length

```text
very_short
short
medium_short
medium
long
unknown
```

---

## 6. Field Semantics

## 6.1 Affect

Affect describes the likely emotional state inferred from user language and local context.

It must not be treated as truth.

Use:

```text
likely
appears
inferred
confidence
```

Avoid:

```text
the user is definitely
the user feels
diagnosis
clinical labels
```

## 6.2 Valence and Arousal

Valence:

```text
-1.0 = very negative
 0.0 = neutral
 1.0 = very positive
```

Arousal:

```text
0.0 = low energy
1.0 = high activation/urgency
```

## 6.3 Intent

Intent classifies what the user is trying to accomplish.

Intent should be more important than mood when generating the cloud prompt.

Example:

```text
A frustrated user asking for a draft still needs drafting help, not just emotional reassurance.
```

## 6.4 Risk

Risk should bias toward caution.

If uncertain:

```text
risk.level = "unknown"
```

For medium/high/crisis risk, the phone should retain local control and may block cloud escalation.

## 6.5 Privacy

Privacy fields describe what was removed and what can be shared.

They must not merely describe the user’s topic. They must describe disclosure constraints.

## 6.6 Response Policy

Response policy tells the large model how to answer.

This is the bridge between EQ inference and practical AI behavior.

---

## 7. JSON Schema Draft

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "EQState",
  "type": "object",
  "required": [
    "schema_version",
    "session",
    "affect",
    "intent",
    "risk",
    "privacy",
    "response_policy",
    "context"
  ],
  "properties": {
    "schema_version": {
      "type": "string",
      "const": "0.1"
    },
    "session": {
      "type": "object",
      "required": ["ephemeral_session_id", "timestamp_local", "device_processing_only"],
      "properties": {
        "ephemeral_session_id": {"type": "string"},
        "timestamp_local": {"type": "string"},
        "device_processing_only": {"type": "boolean"}
      }
    },
    "affect": {
      "type": "object",
      "required": ["primary", "secondary", "valence", "arousal", "confidence"],
      "properties": {
        "primary": {"type": "string"},
        "secondary": {"type": "array", "items": {"type": "string"}},
        "valence": {"type": "number", "minimum": -1, "maximum": 1},
        "arousal": {"type": "number", "minimum": 0, "maximum": 1},
        "confidence": {"type": "number", "minimum": 0, "maximum": 1},
        "evidence_type": {"type": "string"}
      }
    },
    "intent": {
      "type": "object",
      "required": ["category", "confidence"],
      "properties": {
        "category": {"type": "string"},
        "subtype": {"type": "string"},
        "confidence": {"type": "number", "minimum": 0, "maximum": 1}
      }
    },
    "risk": {
      "type": "object",
      "required": ["level", "signals", "confidence", "requires_local_escalation"],
      "properties": {
        "level": {"type": "string"},
        "signals": {"type": "array", "items": {"type": "string"}},
        "confidence": {"type": "number", "minimum": 0, "maximum": 1},
        "requires_local_escalation": {"type": "boolean"}
      }
    },
    "privacy": {
      "type": "object",
      "required": ["sensitivity_level", "raw_text_shared", "pii_removed"],
      "properties": {
        "sensitivity_level": {"type": "string"},
        "raw_text_shared": {"type": "boolean"},
        "pii_removed": {"type": "boolean"},
        "sensitive_domains_detected": {"type": "array", "items": {"type": "string"}},
        "redaction_confidence": {"type": "number", "minimum": 0, "maximum": 1}
      }
    },
    "response_policy": {
      "type": "object",
      "required": ["tone", "warmth", "directness", "length", "max_followup_questions"],
      "properties": {
        "tone": {"type": "string"},
        "warmth": {"type": "string"},
        "directness": {"type": "string"},
        "length": {"type": "string"},
        "pace": {"type": "string"},
        "max_followup_questions": {"type": "integer", "minimum": 0, "maximum": 3},
        "format": {"type": "string"}
      }
    },
    "context": {
      "type": "object",
      "required": ["anonymized_summary", "included_raw_excerpt"],
      "properties": {
        "anonymized_summary": {"type": "string"},
        "included_raw_excerpt": {"type": "boolean"},
        "retrieval_notes_included": {"type": "boolean"}
      }
    }
  }
}
```

---

## 8. MCP Tool Contract

## 8.1 Tool: `get_eq_state`

Description:

```text
Returns current privacy-preserving emotional, intent, and risk state inferred on-device.
```

Input:

```json
{
  "detail_level": "minimal"
}
```

Allowed detail levels:

```text
minimal
standard
extended
```

Output:

```json
{
  "schema_version": "0.1",
  "affect": {},
  "intent": {},
  "risk": {},
  "privacy": {}
}
```

## 8.2 Tool: `get_response_policy`

Description:

```text
Returns the recommended response tone, pacing, length, and behavioral constraints.
```

Input:

```json
{}
```

Output:

```json
{
  "tone": "calm_direct",
  "warmth": "moderate",
  "directness": "high",
  "length": "medium_short",
  "max_followup_questions": 1,
  "avoid": ["diagnosis", "over_reassurance"]
}
```

## 8.3 Tool: `get_anonymized_context`

Description:

```text
Returns a locally generated privacy-safe summary of context relevant to the current task.
```

Input:

```json
{
  "task_scope": "draft_response",
  "max_tokens": 800
}
```

Output:

```json
{
  "privacy_tier": "tier_2_anonymized_summary",
  "summary": "User wants help drafting a careful response to a sensitive workplace issue. Identifiers removed.",
  "raw_text_shared": false,
  "redactions": {
    "names_removed": true,
    "organizations_removed": true,
    "locations_removed": true,
    "rare_details_generalized": true
  }
}
```

## 8.4 Tool: `request_more_context`

Description:

```text
Requests explicit user approval before escalating to a more revealing privacy tier.
```

Input:

```json
{
  "reason": "Exact wording may improve drafting quality.",
  "requested_tier": "tier_3_user_approved_excerpt"
}
```

Output:

```json
{
  "user_action_required": true,
  "approval_status": "pending",
  "allowed_alternatives": [
    "metadata_only",
    "anonymized_summary",
    "cancel"
  ]
}
```

---

## 9. Validation Rules

Before any EQ State leaves the device:

```text
JSON schema must validate.
PII scan must pass.
Privacy tier must be attached.
TTL must be attached.
Payload must be scoped to current session.
No hidden reasoning field.
No raw text unless privacy tier permits it.
No debug fields in production.
```

---

## 10. Failure Behavior

| Failure | Required Behavior |
|---|---|
| Invalid JSON | Retry once with strict repair prompt; fallback to neutral/unknown state. |
| Low confidence affect | Return `unknown` or low-confidence label. |
| PII found in summary | Block and regenerate. |
| Risk high/crisis | Keep local; trigger safety flow. |
| Tool asks for forbidden data | Return refusal object. |
| Payload expired | Require fresh device-side generation. |

Example refusal object:

```json
{
  "error": {
    "code": "PRIVACY_SCOPE_DENIED",
    "message": "Requested data is outside the current privacy tier.",
    "allowed_alternatives": ["get_eq_state", "get_anonymized_context"]
  }
}
```

---

## 11. Minimal v0.1 Schema

For MVP, use a smaller schema:

```json
{
  "schema_version": "0.1",
  "affect": {
    "primary": "frustrated",
    "confidence": 0.78
  },
  "intent": {
    "category": "decision_support",
    "confidence": 0.84
  },
  "risk": {
    "level": "none",
    "requires_local_escalation": false
  },
  "privacy": {
    "sensitivity_level": "medium",
    "raw_text_shared": false
  },
  "response_policy": {
    "tone": "calm_direct",
    "length": "medium_short",
    "max_followup_questions": 1
  },
  "context": {
    "anonymized_summary": "User wants practical help with a sensitive situation. Identifiers removed."
  }
}
```

This is enough to prove the product.

---

## 12. Final Principle

The schema is the product boundary.

Models can change. Runtimes can change. MCP hosts can change.

The EQ State contract should remain stable.
