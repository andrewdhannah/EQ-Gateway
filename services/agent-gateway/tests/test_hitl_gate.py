"""Test the HITL gate — pause/resume for human review."""

from agent_gateway.models import ChatRequest, EQState, HumanDecision, RiskLevel, RoutingDecision, RouteResult
from agent_gateway import hitl_gate


def _make_route_result(risk: str = "medium") -> RouteResult:
    return RouteResult(
        decision=RoutingDecision.requires_human_approval,
        reason="Test: medium risk",
        risk_level=RiskLevel.medium,
        sensitivity_level="medium",
    )


def test_pause_for_review():
    """Pausing a request stores it in the pending store."""
    request = ChatRequest(message="I'm really struggling today")
    state = EQState()
    route_result = _make_route_result()
    request_id = "test-pause-001"

    hitl_gate.pause_for_review(request_id, request, state, route_result)
    pending = hitl_gate.get_pending_requests()

    ids = [p["request_id"] for p in pending]
    assert request_id in ids


def test_submit_human_decision_approve():
    """Submitting an approval updates the pending request."""
    request = ChatRequest(message="Test message")
    state = EQState()
    route_result = _make_route_result()
    request_id = "test-approve-001"

    hitl_gate.pause_for_review(request_id, request, state, route_result)
    success = hitl_gate.submit_human_decision(
        request_id=request_id,
        decision=HumanDecision.approve,
    )
    assert success is True

    status = hitl_gate.get_request_status(request_id)
    assert status is not None
    assert status["status"] == "completed"
    assert status["approved"] is True


def test_submit_human_decision_reject():
    """Submitting a rejection updates the pending request."""
    request = ChatRequest(message="Test message")
    state = EQState()
    route_result = _make_route_result()
    request_id = "test-reject-001"

    hitl_gate.pause_for_review(request_id, request, state, route_result)
    success = hitl_gate.submit_human_decision(
        request_id=request_id,
        decision=HumanDecision.reject,
        reviewer_notes="Not appropriate",
    )
    assert success is True

    status = hitl_gate.get_request_status(request_id)
    assert status is not None
    assert status["status"] == "completed"
    assert status["approved"] is False


def test_submit_decision_nonexistent_request():
    """Submitting a decision for a nonexistent request returns False."""
    success = hitl_gate.submit_human_decision(
        request_id="nonexistent",
        decision=HumanDecision.approve,
    )
    assert success is False


def test_get_pending_requests_returns_list():
    """get_pending_requests returns a list of pending request summaries."""
    request = ChatRequest(message="Help me please")
    state = EQState()
    route_result = _make_route_result()

    hitl_gate.pause_for_review("test-list-001", request, state, route_result)
    pending = hitl_gate.get_pending_requests()

    assert isinstance(pending, list)
    if pending:
        p = pending[0]
        assert "request_id" in p
        assert "message" in p
        assert "risk_level" in p
        assert "affect_primary" in p


def test_get_request_status_nonexistent():
    """get_request_status returns None for nonexistent request."""
    status = hitl_gate.get_request_status("does-not-exist")
    assert status is None
