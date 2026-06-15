"""Test the model router — routing decisions based on EQ State risk/sensitivity."""

from agent_gateway.models import (
    EQState,
    RiskLevel,
    PrivacySensitivity,
    RoutingDecision,
)
from agent_gateway.router import route, needs_human_approval


def _make_eq_state(
    risk: RiskLevel = RiskLevel.none_,
    sensitivity: PrivacySensitivity = PrivacySensitivity.public,
) -> EQState:
    """Helper to create an EQState with specific risk/sensitivity."""
    state = EQState()
    state.risk.level = risk
    state.privacy.sensitivity_level = sensitivity
    return state


def test_route_low_risk_low_sensitivity():
    """Low risk + low sensitivity → local_only."""
    state = _make_eq_state(RiskLevel.low, PrivacySensitivity.low)
    result = route(state)
    assert result.decision == RoutingDecision.local_only


def test_route_none_risk_public():
    """None risk + public → local_only."""
    state = _make_eq_state(RiskLevel.none_, PrivacySensitivity.public)
    result = route(state)
    assert result.decision == RoutingDecision.local_only


def test_route_low_risk_high_sensitivity():
    """Low risk + high sensitivity → cloud_with_metadata."""
    state = _make_eq_state(RiskLevel.low, PrivacySensitivity.high)
    result = route(state)
    assert result.decision == RoutingDecision.cloud_with_metadata


def test_route_medium_risk_requires_approval():
    """Medium risk → always requires_human_approval."""
    state = _make_eq_state(RiskLevel.medium)
    result = route(state)
    assert result.decision == RoutingDecision.requires_human_approval
    assert needs_human_approval(result) is True


def test_route_high_risk_blocked():
    """High risk → always block."""
    state = _make_eq_state(RiskLevel.high)
    result = route(state)
    assert result.decision == RoutingDecision.block


def test_route_crisis_blocked():
    """Crisis risk → always block."""
    state = _make_eq_state(RiskLevel.crisis)
    result = route(state)
    assert result.decision == RoutingDecision.block


def test_route_unknown_risk_safe_default():
    """Unknown risk defaults to local_only."""
    state = _make_eq_state(RiskLevel.unknown, PrivacySensitivity.public)
    result = route(state)
    assert result.decision == RoutingDecision.local_only


def test_route_medium_risk_any_sensitivity_requires_approval():
    """Medium risk overrides any sensitivity level."""
    for sens in PrivacySensitivity:
        state = _make_eq_state(RiskLevel.medium, sens)
        result = route(state)
        assert result.decision == RoutingDecision.requires_human_approval, (
            f"Failed for sensitivity={sens}"
        )


def test_needs_human_approval():
    """needs_human_approval returns True only for requires_human_approval."""
    state = _make_eq_state(RiskLevel.medium)
    result = route(state)
    assert needs_human_approval(result) is True

    state2 = _make_eq_state(RiskLevel.low)
    result2 = route(state2)
    assert needs_human_approval(result2) is False


def test_route_has_reason():
    """Route result includes a human-readable reason."""
    state = _make_eq_state(RiskLevel.medium, PrivacySensitivity.high)
    result = route(state)
    assert len(result.reason) > 10


def test_route_preserves_risk_and_sensitivity():
    """Route result preserves the original risk and sensitivity values."""
    state = _make_eq_state(RiskLevel.low, PrivacySensitivity.medium)
    result = route(state)
    assert result.risk_level == RiskLevel.low
    assert result.sensitivity_level == PrivacySensitivity.medium
