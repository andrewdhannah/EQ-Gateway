"""Observability Receipts — structured JSON log for every request.

Every request through the agent gateway emits a receipt with:
- request_id
- Whether raw text left the device
- PII detection status
- Redacted categories
- Model route taken
- Human approval requirement and decision
- Final status
- Latency

Receipts are written to a configurable log directory as JSON files.
"""

import json
import logging
import os
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from uuid import uuid4

from .config import settings
from .models import (
    EQState,
    FinalStatus,
    HumanDecision,
    ObservabilityReceipt,
    RouteResult,
)

logger = logging.getLogger(__name__)


def _ensure_receipt_dir() -> Path:
    """Create the receipt directory if it doesn't exist."""
    receipt_dir = Path(settings.receipt_log_dir)
    receipt_dir.mkdir(parents=True, exist_ok=True)
    return receipt_dir


def _pii_detected(eq_state: EQState) -> tuple[bool, list[str]]:
    """Check PII status from EQ State."""
    domains = eq_state.privacy.sensitive_domains_detected
    return len(domains) > 0, domains


def _raw_text_left_device(eq_state: EQState, route: RouteResult) -> bool:
    """Determine if raw text left the device based on route + privacy state."""
    # If route says local_only, text never left
    if route.decision.value == "local_only":
        return False
    # If privacy says raw text was shared
    return eq_state.privacy.raw_text_shared


def emit_receipt(
    eq_state: EQState,
    route: RouteResult,
    final_status: FinalStatus,
    latency_ms: int,
    human_decision: HumanDecision | None = None,
) -> str:
    """Emit a structured observability receipt for a completed request.

    Returns the file path to the written receipt.
    """
    request_id = str(uuid4())
    has_pii, pii_categories = _pii_detected(eq_state)

    receipt = ObservabilityReceipt(
        request_id=request_id,
        raw_text_left_device=_raw_text_left_device(eq_state, route),
        pii_detected=has_pii,
        redacted_categories=pii_categories,
        affect_primary=eq_state.affect.primary.value,
        risk_level=eq_state.risk.level.value,
        model_route=route.decision.value,
        human_approval_required=route.decision.value
        == "requires_human_approval",
        human_decision=human_decision.value if human_decision else None,
        final_status=final_status.value,
        latency_ms=latency_ms,
    )

    receipt_dir = _ensure_receipt_dir()
    file_path = receipt_dir / f"receipt_{request_id}.json"

    with open(file_path, "w") as f:
        f.write(receipt.model_dump_json(indent=2))

    logger.info(
        "Receipt written: %s (status=%s, route=%s, latency=%dms)",
        file_path,
        final_status.value,
        route.decision.value,
        latency_ms,
    )

    return str(file_path)


def list_receipts(limit: int = 10) -> list[dict[str, Any]]:
    """List the most recent receipts."""
    receipt_dir = _ensure_receipt_dir()
    files = sorted(
        receipt_dir.glob("receipt_*.json"),
        key=os.path.getmtime,
        reverse=True,
    )[:limit]

    results = []
    for f in files:
        try:
            with open(f) as fh:
                data = json.load(fh)
                results.append({
                    "request_id": data.get("request_id"),
                    "timestamp": data.get("timestamp"),
                    "final_status": data.get("final_status"),
                    "model_route": data.get("model_route"),
                    "risk_level": data.get("risk_level"),
                    "latency_ms": data.get("latency_ms"),
                })
        except Exception as e:
            logger.warning("Failed to read receipt %s: %s", f, e)

    return results
