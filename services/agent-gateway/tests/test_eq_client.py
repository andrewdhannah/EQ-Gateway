"""Test the EQ Client parser — no live backend needed."""

import json

from agent_gateway.eq_client import parse_classification
from agent_gateway.models import AffectPrimary, IntentCategory, RiskLevel


def test_parse_flat_schema():
    """Parse flat schema output (affect_primary, intent_category, etc.)."""
    raw = """```json
{
    "affect_primary": "frustrated",
    "affect_valence": -0.6,
    "affect_arousal": 0.7,
    "intent_category": "technical_help",
    "risk_level": "low",
    "anonymized_summary": "User is stuck on a build issue"
}
```"""
    result = parse_classification(raw)
    assert result.affect.primary == AffectPrimary.frustrated
    assert result.affect.valence == -0.6
    assert result.affect.arousal == 0.7
    assert result.intent.category == IntentCategory.technical_help
    assert result.risk.level == RiskLevel.low
    assert "build issue" in result.context.anonymized_summary


def test_parse_nested_schema():
    """Parse nested schema output (affect.primary, intent.category)."""
    raw = """{
    "affect": {
        "primary": "anxious",
        "valence": -0.4,
        "arousal": 0.8
    },
    "intent": {
        "category": "emotional_support"
    },
    "risk": {
        "level": "medium"
    },
    "context": {
        "anonymized_summary": "User is worried about a presentation"
    }
}"""
    result = parse_classification(raw)
    assert result.affect.primary == AffectPrimary.anxious
    assert result.affect.valence == -0.4
    assert result.intent.category == IntentCategory.emotional_support
    assert result.risk.level == RiskLevel.medium
    assert "presentation" in result.context.anonymized_summary


def test_parse_flat_takes_priority():
    """Flat keys should take priority over nested keys."""
    raw = """{
    "affect_primary": "frustrated",
    "affect": {
        "primary": "calm"
    },
    "intent_category": "technical_help",
    "intent": {
        "category": "emotional_support"
    }
}"""
    result = parse_classification(raw)
    # Flat keys win
    assert result.affect.primary == AffectPrimary.frustrated
    assert result.intent.category == IntentCategory.technical_help


def test_parse_vocabulary_mapping():
    """Model vocabulary mapping with synonyms."""
    test_cases = [
        ("anger", "affect_primary", AffectPrimary.angry),
        ("sadness", "affect_primary", AffectPrimary.sad),
        ("anxiety", "affect_primary", AffectPrimary.anxious),
        ("overwhelm", "affect_primary", AffectPrimary.overwhelmed),
        ("fatigue", "affect_primary", AffectPrimary.fatigued),
        ("seeking_guidance", "intent_category", IntentCategory.practical_guidance),
        ("support", "intent_category", IntentCategory.emotional_support),
    ]
    for raw, field, expected in test_cases:
        text = json.dumps({field: raw})
        result = parse_classification(text)
        found = False
        if isinstance(expected, AffectPrimary):
            if result.affect.primary == expected:
                found = True
        elif isinstance(expected, IntentCategory):
            if result.intent.category == expected:
                found = True
        assert found, f"Failed to map '{raw}' (field={field}) to {expected}"


def test_parse_substring_matching():
    """Substring fallback matching."""
    raw = """{"affect_primary": " I'm feeling frustrated with this ", "intent_category": "tech_help"}"""
    result = parse_classification(raw)
    assert result.affect.primary == AffectPrimary.frustrated
    assert result.intent.category == IntentCategory.technical_help


def test_parse_missing_fields_use_defaults():
    """Missing fields should use defaults without crashing."""
    raw = """{"affect_primary": "unknown"}"""
    result = parse_classification(raw)
    assert result.affect.primary == AffectPrimary.unknown
    assert result.affect.valence == 0.0
    assert result.intent.category == IntentCategory.unknown
    assert result.risk.level == RiskLevel.none_


def test_parse_all_emotional_variants():
    """All 18 emotional variants should parse correctly."""
    emotions = [
        "neutral", "calm", "curious", "pleased", "hopeful",
        "confused", "uncertain", "frustrated", "angry", "sad",
        "anxious", "overwhelmed", "fatigued", "embarrassed",
        "lonely", "excited", "urgent",
    ]
    for emotion in emotions:
        raw = json.dumps({"affect_primary": emotion})
        result = parse_classification(raw)
        assert result.affect.primary.value == emotion, f"Failed for {emotion}"


def test_parse_all_intent_variants():
    """All 13 intent variants should parse correctly."""
    intents = [
        "practical_guidance", "emotional_support", "decision_support",
        "venting", "planning", "clarification", "conflict_navigation",
        "reflection", "task_execution", "creative_help",
        "technical_help", "safety_related",
    ]
    for intent in intents:
        raw = json.dumps({"intent_category": intent})
        result = parse_classification(raw)
        assert result.intent.category.value == intent, f"Failed for {intent}"


def test_parse_invalid_json():
    """Invalid JSON should raise an error."""
    import pytest

    with pytest.raises(Exception):
        parse_classification("not json at all")
