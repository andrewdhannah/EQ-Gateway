"""Model Router — decides how to handle a request based on EQ State risk/sensitivity.

Routing decision matrix:

| Risk Level | Sensitivity   | Decision                | Reason                               |
|:-----------|:--------------|:------------------------|:-------------------------------------|
| none/low   | public/low    | local_only              | Safe, no routing needed              |
| none/low   | medium/high   | cloud_with_metadata     | Safe enough, can use cloud           |
| medium     | any           | requires_human_approval | Moderate risk — human must approve   |
| high/crisis| any           | block                   | Escalate to crisis resources         |
"""

from .config import settings
from .models import (
    EQState,
    RiskLevel,
    PrivacySensitivity,
    RoutingDecision,
    RouteResult,
)


def _risk_value(level: RiskLevel) -> int:
    return {
        RiskLevel.none_: 0,
        RiskLevel.low: 1,
        RiskLevel.medium: 2,
        RiskLevel.high: 3,
        RiskLevel.crisis: 4,
        RiskLevel.unknown: 0,
    }.get(level, 0)


def _sensitivity_value(sensitivity: PrivacySensitivity) -> int:
    return {
        PrivacySensitivity.public: 0,
        PrivacySensitivity.low: 1,
        PrivacySensitivity.medium: 2,
        PrivacySensitivity.high: 3,
        PrivacySensitivity.restricted: 4,
        PrivacySensitivity.unknown: 1,
    }.get(sensitivity, 1)


def _auto_approve_threshold() -> int:
    """Get the numeric threshold for auto-approval from config."""
    return {
        "none": 0,
        "low": 1,
        "medium": 2,
    }.get(settings.hitl_auto_approve_risk_below, 1)


def route(eq_state: EQState) -> RouteResult:
    """Determine the routing decision based on EQ State."""
    risk = eq_state.risk.level
    sensitivity = eq_state.privacy.sensitivity_level
    risk_val = _risk_value(risk)
    sens_val = _sensitivity_value(sensitivity)

    # Crisis — always block
    if risk == RiskLevel.crisis or risk == RiskLevel.high:
        return RouteResult(
            decision=RoutingDecision.block,
            reason=(
                f"Risk level is {risk.value}. "
                "Request blocked — escalate to crisis resources."
            ),
            risk_level=risk,
            sensitivity_level=sensitivity,
        )

    # Medium risk — require human approval
    if risk == RiskLevel.medium:
        return RouteResult(
            decision=RoutingDecision.requires_human_approval,
            reason=(
                f"Risk level is {risk.value}. "
                "Human approval required before proceeding."
            ),
            risk_level=risk,
            sensitivity_level=sensitivity,
        )

    # Low/none risk — depends on sensitivity
    if sens_val >= 3:  # high or restricted sensitivity
        return RouteResult(
            decision=RoutingDecision.cloud_with_metadata,
            reason=(
                f"Risk is {risk.value} but sensitivity is {sensitivity.value}. "
                "Cloud routing with metadata-only is acceptable."
            ),
            risk_level=risk,
            sensitivity_level=sensitivity,
        )

    # Default — local only, safe
    return RouteResult(
        decision=RoutingDecision.local_only,
        reason=(
            f"Risk is {risk.value}, sensitivity is {sensitivity.value}. "
            "Proceeding with local-only processing."
        ),
        risk_level=risk,
        sensitivity_level=sensitivity,
    )


def needs_human_approval(route_result: RouteResult) -> bool:
    """Check if this route result requires the HITL gate."""
    return route_result.decision == RoutingDecision.requires_human_approval
