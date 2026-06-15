"""HITL (Human-In-The-Loop) Gate — LangGraph state machine.

Pauses processing when the model router flags a request as sensitive.
Presents the request to a human reviewer for approve/reject/edit.
Resumes or aborts based on the human decision.
"""

import logging
from datetime import datetime, timezone
from typing import Any
from uuid import uuid4

from langgraph.checkpoint.memory import MemorySaver
from langgraph.graph import StateGraph, START, END
from typing_extensions import TypedDict

from .config import settings
from .models import (
    ChatRequest,
    EQState,
    HumanDecision,
    RouteResult,
    RoutingDecision,
)

logger = logging.getLogger(__name__)


# ── State ──────────────────────────────────────────────────────────────────────

class HITLState(TypedDict):
    """State for the HITL LangGraph."""
    request_id: str
    request: ChatRequest
    eq_state: EQState
    route_result: RouteResult
    human_decision: HumanDecision | None
    edited_message: str | None
    reviewer_notes: str | None
    approved: bool
    completed: bool
    error: str | None


# ── Pending requests store ─────────────────────────────────────────────────────

# In-memory store of requests awaiting human review.
# In production this would be a database; for POC it's a dict.
_pending_requests: dict[str, HITLState] = {}

# Completed decisions store
_completed_decisions: dict[str, HITLState] = {}


# ── LangGraph Nodes ────────────────────────────────────────────────────────────

def enter_gate(state: HITLState) -> HITLState:
    """Entry node — log that we're entering the HITL gate."""
    logger.info(
        "HITL gate: request=%s risk=%s entering pause",
        state["request_id"],
        state["route_result"].risk_level.value,
    )
    # Store in pending
    _pending_requests[state["request_id"]] = state
    return state


def check_decision(state: HITLState) -> HITLState:
    """Check if a human decision has been made.

    This node is polled or triggered by the human decision endpoint.
    If no decision yet, the graph remains paused (LangGraph interrupt).
    """
    pending = _pending_requests.get(state["request_id"])
    if pending is None:
        # Already processed
        return state

    decision = pending.get("human_decision")
    if decision is None:
        # Still waiting — return as-is (graph will be interrupted)
        return state

    # Decision received
    state["human_decision"] = decision
    state["edited_message"] = pending.get("edited_message")
    state["reviewer_notes"] = pending.get("reviewer_notes")

    if decision == HumanDecision.approve:
        state["approved"] = True
        state["completed"] = True
        logger.info("HITL gate: request=%s APPROVED", state["request_id"])
    elif decision == HumanDecision.reject:
        state["approved"] = False
        state["completed"] = True
        logger.info("HITL gate: request=%s REJECTED", state["request_id"])
    elif decision == HumanDecision.edit:
        state["approved"] = True
        state["completed"] = True
        logger.info("HITL gate: request=%s APPROVED WITH EDITS", state["request_id"])
    elif decision == HumanDecision.timed_out:
        state["approved"] = False
        state["completed"] = True
        state["error"] = "Human review timed out"
        logger.info("HITL gate: request=%s TIMED OUT", state["request_id"])

    # Move from pending to completed
    _completed_decisions[state["request_id"]] = state
    _pending_requests.pop(state["request_id"], None)

    return state


def route_by_decision(state: HITLState) -> str:
    """Route based on the human decision."""
    if state["approved"]:
        return "approved"
    return "rejected"


# ── Build Graph ────────────────────────────────────────────────────────────────

def build_hitl_graph() -> StateGraph:
    """Build the HITL gate state graph."""
    builder = StateGraph(HITLState)

    builder.add_node("enter_gate", enter_gate)
    builder.add_node("check_decision", check_decision)

    builder.add_edge(START, "enter_gate")
    builder.add_edge("enter_gate", "check_decision")
    builder.add_conditional_edges(
        "check_decision",
        route_by_decision,
        {
            "approved": END,
            "rejected": END,
        },
    )

    return builder.compile(checkpointer=MemorySaver())


# Global graph instance
hitl_graph = build_hitl_graph()


# ── Public API ─────────────────────────────────────────────────────────────────

def pause_for_review(
    request_id: str,
    request: ChatRequest,
    eq_state: EQState,
    route_result: RouteResult,
) -> None:
    """Submit a request to the HITL gate for human review.

    This creates the initial state and stores it for later pickup.
    """
    initial_state: HITLState = {
        "request_id": request_id,
        "request": request,
        "eq_state": eq_state,
        "route_result": route_result,
        "human_decision": None,
        "edited_message": None,
        "reviewer_notes": None,
        "approved": False,
        "completed": False,
        "error": None,
    }
    _pending_requests[request_id] = initial_state
    logger.info(
        "HITL gate: request=%s paused for review (risk=%s)",
        request_id,
        route_result.risk_level.value,
    )


def submit_human_decision(
    request_id: str,
    decision: HumanDecision,
    edited_message: str | None = None,
    reviewer_notes: str | None = None,
) -> bool:
    """Submit a human decision for a paused request.

    Returns True if the request was found and updated.
    """
    pending = _pending_requests.get(request_id)
    if pending is None:
        logger.warning(
            "HITL gate: request=%s not found in pending store",
            request_id,
        )
        return False

    now = datetime.now(timezone.utc).isoformat()

    pending["human_decision"] = decision
    pending["edited_message"] = edited_message
    pending["reviewer_notes"] = reviewer_notes

    # Set approval/completion state
    if decision in (HumanDecision.approve, HumanDecision.edit):
        pending["approved"] = True
    elif decision in (HumanDecision.reject, HumanDecision.timed_out):
        pending["approved"] = False

    pending["completed"] = True

    # Move from pending to completed
    _completed_decisions[request_id] = pending
    _pending_requests.pop(request_id, None)

    logger.info(
        "HITL gate: request=%s decision=%s submitted (approved=%s)",
        request_id,
        decision.value,
        pending["approved"],
    )
    return True


def get_pending_requests() -> list[dict[str, Any]]:
    """List all requests currently awaiting human review."""
    return [
        {
            "request_id": rid,
            "message": st["request"].message[:200],
            "risk_level": st["route_result"].risk_level.value,
            "sensitivity": st["route_result"].sensitivity_level.value,
            "affect_primary": st["eq_state"].affect.primary.value,
            "reason": st["route_result"].reason,
        }
        for rid, st in _pending_requests.items()
    ]


def get_request_status(request_id: str) -> dict[str, Any] | None:
    """Get the status of a request (pending or completed)."""
    if request_id in _pending_requests:
        st = _pending_requests[request_id]
        return {
            "status": "pending",
            "request_id": request_id,
            "message": st["request"].message[:200],
            "risk_level": st["route_result"].risk_level.value,
        }
    if request_id in _completed_decisions:
        st = _completed_decisions[request_id]
        return {
            "status": "completed",
            "request_id": request_id,
            "decision": st["human_decision"].value if st["human_decision"] else None,
            "approved": st["approved"],
        }
    return None
