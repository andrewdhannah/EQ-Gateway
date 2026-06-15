"""EQ Gateway Agent Service — FastAPI Application.

Endpoints:
- GET  /health          — Service health check
- POST /analyze         — Analyze a message, return EQ State (no routing)
- POST /chat            — Full pipeline: analyze → route → HITL → receipt
- GET  /pending         — List requests awaiting human review
- POST /human-decision  — Submit human decision on a paused request
- GET  /receipts        — List recent observability receipts
- GET  /receipts/{id}   — Get a specific receipt
"""

import logging

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware

from .config import settings
from .eq_client import EQClient
from .models import (
    ChatRequest,
    ChatResponse,
    EQState,
    FinalStatus,
    HITLRequest,
    HumanDecision,
    RouteResult,
    RoutingDecision,
)
from .receipt import emit_receipt, list_receipts
from .router import needs_human_approval, route
from . import hitl_gate

logger = logging.getLogger(__name__)

# ── FastAPI App ────────────────────────────────────────────────────────────────

app = FastAPI(
    title="EQ Gateway — Agent Service",
    version="0.1.0",
    description="Local context firewall for agentic AI with HITL gates",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# ── Dependencies ───────────────────────────────────────────────────────────────

eq_client = EQClient()


# ── Endpoints ──────────────────────────────────────────────────────────────────

@app.get("/health")
async def health():
    """Health check endpoint."""
    backend_ok = eq_client.health_check()
    return {
        "status": "ok" if backend_ok else "degraded",
        "backend": eq_client.status(),
        "backend_connected": backend_ok,
    }


@app.post("/analyze")
async def analyze(request: ChatRequest):
    """Analyze a message and return EQ State.

    This is the raw analysis endpoint — no routing, no HITL, no receipt.
    Useful for debugging and integration testing.
    """
    try:
        eq_state, latency_ms = eq_client.classify_with_latency(
            request.message,
            request.session_id,
        )
        return {
            "request_id": request.session_id or "debug",
            "eq_state": eq_state.model_dump(),
            "latency_ms": latency_ms,
        }
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e))


@app.post("/chat", response_model=ChatResponse)
async def chat(request: ChatRequest):
    """Full agent pipeline: analyze → route → HITL → receipt.

    Flow:
    1. Classify the message via local SLM
    2. Route based on risk/sensitivity
    3. If high risk → pause for human approval (HITL)
    4. Emit observability receipt
    5. Return structured response
    """
    # Step 1: Classify
    try:
        eq_state, latency_ms = eq_client.classify_with_latency(
            request.message,
            request.session_id,
        )
    except Exception as e:
        logger.error("Classification failed: %s", e)
        raise HTTPException(status_code=502, detail=f"Classification failed: {e}")

    # Step 2: Route
    route_result = route(eq_state)
    logger.info(
        "Request routed: risk=%s sensitivity=%s decision=%s",
        route_result.risk_level.value,
        route_result.sensitivity_level.value,
        route_result.decision.value,
    )

    human_decision = None
    final_status = FinalStatus.completed
    response_text = None

    # Step 3: HITL Gate (if required)
    if needs_human_approval(route_result):
        request_id = request.session_id or f"req_{latency_ms}"
        hitl_gate.pause_for_review(
            request_id=request_id,
            request=request,
            eq_state=eq_state,
            route_result=route_result,
        )

        # For non-interactive calls, check if a decision is already available
        decision_state = hitl_gate.get_request_status(request_id)
        if decision_state and decision_state["status"] == "completed":
            # Decision already submitted by external endpoint
            completed = hitl_gate._completed_decisions.get(request_id)
            if completed:
                human_decision = completed["human_decision"]
                if completed["approved"]:
                    final_status = FinalStatus.completed
                    response_text = _generate_response(eq_state, route_result)
                else:
                    final_status = FinalStatus.rejected_by_human
                    response_text = None
        else:
            # Request is paused — return pending status
            return ChatResponse(
                request_id=request_id,
                eq_state=eq_state,
                route=route_result,
                final_status=FinalStatus.completed,
                response_text=_generate_response(eq_state, route_result),
            )

    elif route_result.decision == RoutingDecision.block:
        final_status = FinalStatus.rejected_by_policy
        response_text = None
    else:
        # Safe — proceed directly
        response_text = _generate_response(eq_state, route_result)

    # Step 4: Emit receipt
    receipt_path = emit_receipt(
        eq_state=eq_state,
        route=route_result,
        final_status=final_status,
        latency_ms=latency_ms,
        human_decision=human_decision,
    )

    return ChatResponse(
        request_id=request.session_id or f"auto_{latency_ms}",
        eq_state=eq_state,
        route=route_result,
        human_decision=human_decision,
        final_status=final_status,
        response_text=response_text,
        receipt_path=receipt_path,
    )


@app.get("/pending")
async def list_pending():
    """List all requests currently awaiting human review."""
    return {"pending": hitl_gate.get_pending_requests()}


@app.post("/human-decision")
async def human_decision(decision: HITLRequest):
    """Submit a human decision on a paused request."""
    success = hitl_gate.submit_human_decision(
        request_id=decision.request_id,
        decision=decision.decision,
        edited_message=decision.edited_message,
        reviewer_notes=decision.reviewer_notes,
    )
    if not success:
        raise HTTPException(
            status_code=404,
            detail=f"Request {decision.request_id} not found in pending store",
        )
    return {
        "status": "ok",
        "request_id": decision.request_id,
        "decision": decision.decision.value,
    }


@app.get("/receipts")
async def list_receipts_endpoint(limit: int = 10):
    """List recent observability receipts."""
    return {"receipts": list_receipts(limit=limit)}


# ── Helpers ────────────────────────────────────────────────────────────────────

def _generate_response(eq_state: EQState, route_result: RouteResult) -> str:
    """Generate a placeholder response based on EQ State.

    In production this would call the cloud/large model.
    For POC, we return a simulated adaptive response.
    """
    affect = eq_state.affect.primary.value
    intent = eq_state.intent.category.value
    risk = eq_state.risk.level.value

    # Simulated adaptive response
    responses = {
        ("frustrated", "technical_help"): (
            "I can see you're frustrated with a technical issue. "
            "Let me help you work through this step by step."
        ),
        ("anxious", "emotional_support"): (
            "I hear how anxious this situation is making you feel. "
            "Take a breath — we'll figure this out together."
        ),
        ("curious", "clarification"): (
            "Great question! Let me clarify that for you."
        ),
        ("sad", "emotional_support"): (
            "I'm sorry you're feeling this way. I'm here to listen "
            "and help however I can."
        ),
    }

    key = (affect, intent)
    if key in responses:
        return responses[key]

    # Default adaptive response
    tone = "warm and supportive" if risk in ("low", "medium") else "neutral"
    return (
        f"I've noted you're feeling {affect} and looking for {intent.replace('_', ' ')}. "
        f"I'll respond in a {tone} tone. "
        f"How can I best support you right now?"
    )


# ── Entry Point ────────────────────────────────────────────────────────────────

def run():
    """Run the FastAPI app with uvicorn."""
    import uvicorn

    uvicorn.run(
        "agent_gateway.main:app",
        host=settings.host,
        port=settings.port,
        log_level=settings.log_level,
        reload=True,
    )


if __name__ == "__main__":
    run()
