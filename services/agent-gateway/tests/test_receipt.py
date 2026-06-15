"""Test observability receipts."""

import json
import os
import tempfile
from pathlib import Path

from agent_gateway.models import EQState, FinalStatus, HumanDecision
from agent_gateway.receipt import emit_receipt
from agent_gateway.router import route


def test_emit_receipt_creates_file():
    """emit_receipt creates a JSON file in the receipt directory."""
    state = EQState()
    route_result = route(state)

    with tempfile.TemporaryDirectory() as tmp:
        # Override receipt dir
        import agent_gateway.receipt as receipt_mod
        original_dir = receipt_mod.settings.receipt_log_dir
        receipt_mod.settings.receipt_log_dir = tmp

        try:
            file_path = emit_receipt(
                eq_state=state,
                route=route_result,
                final_status=FinalStatus.completed,
                latency_ms=1234,
            )
            assert os.path.exists(file_path)
            with open(file_path) as f:
                data = json.load(f)
            assert data["request_id"] is not None
            assert data["final_status"] == "completed"
            assert data["model_route"] == "local_only"
            assert data["latency_ms"] == 1234
            assert data["schema_version"] == "receipt-v0.1"
        finally:
            receipt_mod.settings.receipt_log_dir = original_dir


def test_emit_receipt_with_human_decision():
    """Receipt captures human decision when provided."""
    state = EQState()
    route_result = route(state)

    with tempfile.TemporaryDirectory() as tmp:
        import agent_gateway.receipt as receipt_mod
        original_dir = receipt_mod.settings.receipt_log_dir
        receipt_mod.settings.receipt_log_dir = tmp

        try:
            file_path = emit_receipt(
                eq_state=state,
                route=route_result,
                final_status=FinalStatus.rejected_by_human,
                latency_ms=500,
                human_decision=HumanDecision.reject,
            )
            with open(file_path) as f:
                data = json.load(f)
            assert data["final_status"] == "rejected_by_human"
            assert data["human_decision"] == "reject"
            assert data["human_approval_required"] is False  # risk was none
        finally:
            receipt_mod.settings.receipt_log_dir = original_dir


def test_emit_receipt_with_pii():
    """Receipt reflects PII detection."""
    state = EQState()
    state.privacy.sensitive_domains_detected = ["email", "phone"]
    state.privacy.pii_removed = True
    route_result = route(state)

    with tempfile.TemporaryDirectory() as tmp:
        import agent_gateway.receipt as receipt_mod
        original_dir = receipt_mod.settings.receipt_log_dir
        receipt_mod.settings.receipt_log_dir = tmp

        try:
            file_path = emit_receipt(
                eq_state=state,
                route=route_result,
                final_status=FinalStatus.completed,
                latency_ms=200,
            )
            with open(file_path) as f:
                data = json.load(f)
            assert data["pii_detected"] is True
            assert "email" in data["redacted_categories"]
            assert "phone" in data["redacted_categories"]
        finally:
            receipt_mod.settings.receipt_log_dir = original_dir
