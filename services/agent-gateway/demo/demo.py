#!/usr/bin/env python3
"""EQ Gateway — Agent Service Demo.

Exercises the full pipeline against a live llama-server-mini backend.

Prerequisites:
- llama-server-mini running on 127.0.0.1:9120 (or EQ_BACKEND_URL)
- Phi-4-mini-instruct Q4_K_M model loaded

Usage:
    python demo/demo.py
    python demo/demo.py --message "I'm really frustrated with this build error"
"""

import argparse
import json
import sys
import time
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "src"))

from agent_gateway.eq_client import EQClient, EQClientError
from agent_gateway.router import route
from agent_gateway.models import EQState, RoutingDecision


# ── Test Messages ─────────────────────────────────────────────────────────────

TEST_MESSAGES = [
    # Technical frustration
    "I'm really frustrated with this bug in the build system — it keeps crashing "
    "and I've tried everything. Can you help me figure out what's wrong?",
    # Pre-performance anxiety  
    "I have a big presentation tomorrow and I'm really nervous. I've prepared "
    "but I'm afraid I'll freeze up in front of everyone.",
    # General curiosity
    "Can you explain how the EQ Gateway pipeline works? I'm curious about the "
    "privacy guarantees.",
    # Extended frustration
    "I've been working on this for weeks and nothing is working. The documentation "
    "is wrong, the examples don't compile, and I'm running out of ideas.",
    # Concern for another
    "I'm really worried about my friend — they haven't been themselves lately. "
    "They've been withdrawing from everyone and I don't know how to help.",
    # Sensitive / workplace conflict (triggers HITL gate)
    "I'm having a serious conflict with my manager at work. They keep taking credit "
    "for my work and I'm considering quitting. What should I do?",
]


def print_header(text: str):
    print(f"\n{'='*60}")
    print(f"  {text}")
    print(f"{'='*60}")


def print_eq_state(state: EQState, label: str = "EQ State"):
    print(f"\n  📊 {label}:")
    print(f"     Affect: {state.affect.primary.value:15s}  "
          f"valence={state.affect.valence:+.1f}  "
          f"arousal={state.affect.arousal:.1f}")
    print(f"     Intent: {state.intent.category.value:15s}  "
          f"confidence={state.intent.confidence:.2f}")
    print(f"     Risk:   {state.risk.level.value:15s}  "
          f"signals={state.risk.signals}")
    if state.context.anonymized_summary:
        summary = state.context.anonymized_summary
        if len(summary) > 80:
            summary = summary[:77] + "..."
        print(f"     Summary: {summary}")


def print_route(decision: str, reason: str):
    icon = {"local_only": "🟢", "cloud_with_metadata": "🟡",
            "requires_human_approval": "🟠", "block": "🔴"}.get(decision, "⚪")
    print(f"\n  {icon} Route: {decision}")
    print(f"     Reason: {reason}")


def print_receipt(receipt_path: str):
    if receipt_path and Path(receipt_path).exists():
        with open(receipt_path) as f:
            data = json.load(f)
        print(f"\n  📋 Receipt: {receipt_path}")
        print(f"     Status: {data['final_status']}")
        print(f"     Route:  {data['model_route']}")
        print(f"     PII:    {'⚠️  Detected' if data['pii_detected'] else '✅ None'}")
        print(f"     Human:  {'🫡 Required' if data['human_approval_required'] else '🤖 Auto'}")
        print(f"     Latency: {data['latency_ms']}ms")


def main():
    parser = argparse.ArgumentParser(description="EQ Gateway Agent Service Demo")
    parser.add_argument("--message", "-m", type=str, help="Single message to analyze")
    parser.add_argument("--all", "-a", action="store_true", help="Run all test messages")
    parser.add_argument("--host", type=str, default=None, help="Backend host")
    parser.add_argument("--port", type=int, default=None, help="Backend port")
    args = parser.parse_args()

    # Initialize client
    client = EQClient(host=args.host, port=args.port)

    print_header("EQ Gateway — Agent Service Demo")
    print(f"  Backend: {client.base_url}")
    print(f"  Status:  {client.status()}")

    if not client.health_check():
        print("\n  ❌ Backend not reachable!")
        print("  Make sure llama-server-mini is running on 127.0.0.1:9120")
        print("  Or set EQ_BACKEND_URL=http://host:port")
        sys.exit(1)

    print("  ✅ Backend connected!\n")

    # Determine which messages to run
    if args.message:
        messages = [args.message]
    elif args.all:
        messages = TEST_MESSAGES
    else:
        # Run a curated subset by default
        messages = TEST_MESSAGES[:3]

    print(f"  Running {len(messages)} message(s)...\n")

    for i, msg in enumerate(messages, 1):
        print_header(f"Message {i}/{len(messages)}")
        print(f"\n  💬 \"{msg[:100]}{'...' if len(msg) > 100 else ''}\"")

        # Step 1: Classify
        start = time.perf_counter()
        try:
            state, latency = client.classify_with_latency(msg)
            elapsed = int((time.perf_counter() - start) * 1000)
        except EQClientError as e:
            print(f"\n  ❌ Classification failed: {e}")
            continue

        print_eq_state(state)
        print(f"\n     ⏱  {latency}ms (total: {elapsed}ms)")

        # Step 2: Route
        route_result = route(state)
        print_route(route_result.decision.value, route_result.reason)

        # Step 3: Check if HITL would be triggered
        if route_result.decision == RoutingDecision.requires_human_approval:
            print("\n  🟠 → HITL gate: Would pause for human approval")
        elif route_result.decision == RoutingDecision.block:
            print("\n  🔴 → Blocked by policy")
        else:
            print(f"\n  ✅ → Proceeding with {route_result.decision.value}")

        print(f"\n  {'─'*55}")

    print_header("Demo Complete")
    print(f"  {len(messages)} messages processed.")
    print("\n  Next steps:")
    print("    python -m uvicorn agent_gateway.main:app --port 8090")
    print("    → POST /chat with your messages")

    # Save last result as sample
    if 'state' in locals():
        sample_path = Path(__file__).resolve().parent.parent / "demo" / "last_result.json"
        with open(sample_path, "w") as f:
            json.dump(state.model_dump(), f, indent=2)
        print(f"\n  Last result saved to: {sample_path}")


if __name__ == "__main__":
    main()
