"""Tests for the EQ State JSON Schema validator."""

import json
from pathlib import Path

import pytest

from eq_gateway_eval import EQStateValidator, ValidationResult

# ---------------------------------------------------------------------------
# Fixture paths
# ---------------------------------------------------------------------------
FIXTURES = Path(__file__).resolve().parent.parent / "fixtures"


@pytest.fixture(scope="session")
def validator() -> EQStateValidator:
    """Shared validator instance (loads schema once per test session)."""
    return EQStateValidator()


@pytest.fixture
def sample_state() -> dict:
    """Load the sample valid EQ State fixture."""
    path = FIXTURES / "sample_eq_state.json"
    with open(path, "r") as f:
        return json.load(f)


@pytest.fixture
def invalid_missing() -> dict:
    """Load the fixture missing a required field."""
    path = FIXTURES / "invalid_missing_field.json"
    with open(path, "r") as f:
        return json.load(f)


@pytest.fixture
def invalid_bad_values() -> dict:
    """Load the fixture with bad/invalid values."""
    path = FIXTURES / "invalid_bad_values.json"
    with open(path, "r") as f:
        return json.load(f)


# ---------------------------------------------------------------------------
# Valid EQ State
# ---------------------------------------------------------------------------


class TestValidEQState:
    def test_valid_state_passes(self, validator: EQStateValidator, sample_state: dict):
        result = validator.validate_state(sample_state)
        assert result.valid, f"Valid state should pass: {result.errors}"

    def test_valid_json_string(self, validator: EQStateValidator, sample_state: dict):
        json_str = json.dumps(sample_state)
        result = validator.validate_json(json_str)
        assert result.valid, f"Valid JSON string should pass: {result.errors}"

    def test_valid_state_has_data(self, validator: EQStateValidator, sample_state: dict):
        result = validator.validate_state(sample_state)
        assert result.data is not None
        assert result.data["schema_version"] == "0.1"

    def test_validation_result_bool(self, validator: EQStateValidator, sample_state: dict):
        result = validator.validate_state(sample_state)
        assert bool(result) is True
        assert str(result).startswith("✓")


# ---------------------------------------------------------------------------
# Invalid — Missing Required Field
# ---------------------------------------------------------------------------


class TestMissingField:
    def test_missing_evidence_type_fails(self, validator: EQStateValidator, invalid_missing: dict):
        result = validator.validate_state(invalid_missing)
        assert not result.valid
        # Should complain about 'evidence_type' being missing in 'affect'
        assert any("evidence_type" in err for err in result.errors), (
            f"Expected error about 'evidence_type', got: {result.errors}"
        )

    def test_missing_field_has_no_data(self, validator: EQStateValidator, invalid_missing: dict):
        result = validator.validate_state(invalid_missing)
        assert result.data is None

    def test_validation_result_bool_false(self, validator: EQStateValidator, invalid_missing: dict):
        result = validator.validate_state(invalid_missing)
        assert bool(result) is False
        assert str(result).startswith("✗")


# ---------------------------------------------------------------------------
# Invalid — Bad Values (type, enum, range violations)
# ---------------------------------------------------------------------------


class TestBadValues:
    def test_multiple_errors(self, validator: EQStateValidator, invalid_bad_values: dict):
        result = validator.validate_state(invalid_bad_values)
        assert not result.valid
        # Should catch many errors
        assert len(result.errors) > 5, f"Expected many errors, got {len(result.errors)}: {result.errors}"

    def test_schema_version_type_error(self, validator: EQStateValidator, invalid_bad_values: dict):
        result = validator.validate_state(invalid_bad_values)
        schema_errors = [e for e in result.errors if "schema_version" in e]
        assert len(schema_errors) > 0, f"Expected schema_version type error: {result.errors}"

    def test_affect_primary_enum_error(self, validator: EQStateValidator, invalid_bad_values: dict):
        result = validator.validate_state(invalid_bad_values)
        affect_errors = [e for e in result.errors if "affect" in e and "primary" in e]
        assert len(affect_errors) > 0, f"Expected affect.primary enum error: {result.errors}"

    def test_valence_out_of_range(self, validator: EQStateValidator, invalid_bad_values: dict):
        result = validator.validate_state(invalid_bad_values)
        range_errors = [e for e in result.errors if "valence" in e and "999" in e]
        assert len(range_errors) > 0, f"Expected valence range error: {result.errors}"


# ---------------------------------------------------------------------------
# Edge Cases
# ---------------------------------------------------------------------------


class TestEdgeCases:
    def test_empty_dict_fails(self, validator: EQStateValidator):
        result = validator.validate_state({})
        assert not result.valid
        assert len(result.errors) > 0

    def test_none_fails(self, validator: EQStateValidator):
        result = validator.validate_state(None)  # type: ignore[arg-type]
        assert not result.valid
        assert len(result.errors) > 0

    def test_invalid_json_string(self, validator: EQStateValidator):
        result = validator.validate_json("{{broken json")
        assert not result.valid
        assert any("Invalid JSON" in err for err in result.errors)

    def test_schema_property(self, validator: EQStateValidator):
        assert "$id" in validator.schema
        assert "EQState" in validator.schema.get("title", "")

    def test_extra_property_fails(self, validator: EQStateValidator):
        """Any additional property at the root level should fail."""
        state = {
            "schema_version": "0.1",
            "session": {
                "ephemeral_session_id": "550e8400-e29b-41d4-a716-446655440000",
                "timestamp_local": "2026-06-08T12:30:00Z",
                "device_processing_only": False,
            },
            "affect": {
                "primary": "neutral",
                "secondary": [],
                "valence": 0.0,
                "arousal": 0.3,
                "confidence": 0.5,
                "evidence_type": "semantic_inference",
            },
            "intent": {
                "category": "unknown",
                "subtype": "test",
                "confidence": 0.5,
            },
            "risk": {
                "level": "none",
                "signals": [],
                "confidence": 0.5,
                "requires_local_escalation": False,
            },
            "privacy": {
                "sensitivity_level": "public",
                "raw_text_shared": False,
                "pii_removed": False,
                "sensitive_domains_detected": [],
                "redaction_confidence": 0.9,
            },
            "response_policy": {
                "tone": "neutral_professional",
                "warmth": 0.5,
                "directness": 0.5,
                "length": "medium",
                "pace": "steady",
                "max_followup_questions": 2,
                "format": "prose",
            },
            "context": {
                "anonymized_summary": "Test.",
                "included_raw_excerpt": False,
                "retrieval_notes_included": False,
            },
            "extra_field_should_not_exist": True,
        }
        result = validator.validate_state(state)
        assert not result.valid
        assert any("extra_field" in err for err in result.errors)


# ---------------------------------------------------------------------------
# Round-trip: Rust engine output validation
# ---------------------------------------------------------------------------


class TestRustEngineOutput:
    """Tests that validate output from the Rust engine pipeline."""

    def test_rust_sample_matches_fixture(self, validator: EQStateValidator, sample_state: dict):
        """
        Simulates validating output from the Rust engine.
        In CI, this would call the compiled Rust binary and validate real output.
        """
        result = validator.validate_state(sample_state)
        assert result.valid

        # Verify critical Rust-crate-level invariants
        assert result.data is not None

        affect = result.data["affect"]
        assert -1.0 <= affect["valence"] <= 1.0
        assert 0.0 <= affect["arousal"] <= 1.0
        assert 0.0 <= affect["confidence"] <= 1.0

        privacy = result.data["privacy"]
        assert isinstance(privacy["pii_removed"], bool)
        assert isinstance(privacy["raw_text_shared"], bool)

        risk = result.data["risk"]
        assert isinstance(risk["requires_local_escalation"], bool)


# ---------------------------------------------------------------------------
# Schema loading errors
# ---------------------------------------------------------------------------


class TestSchemaLoading:
    def test_missing_schema_file(self):
        with pytest.raises(FileNotFoundError):
            EQStateValidator(schema_path="/nonexistent/path/schema.json")

    def test_multiple_validators_share_schema(self):
        v1 = EQStateValidator()
        v2 = EQStateValidator()
        # Both should reference (different copies of) the same schema
        assert v1.schema["$id"] == v2.schema["$id"]
