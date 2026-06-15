"""Python EQ Client — calls llama-server-mini HTTP API and parses structured output.

Mirrors the Rust LlamaCppAdapter logic:
- JSON-envelope prompt format wraps user message in {"instruction","schema","user_message"}
- Parses both flat (affect_primary) and nested (affect.primary) output schemas
- Model vocabulary mapping with substring fallback
"""

import json
import logging
import time
from typing import Any

import httpx

from .config import settings
from .models import (
    AffectPrimary,
    EQState,
    IntentCategory,
    PrivacySensitivity,
    ResponseTone,
    RiskLevel,
    SessionInfo,
    AffectState,
    IntentState,
    RiskState,
    PrivacyState,
    ResponsePolicy,
    ContextState,
)

logger = logging.getLogger(__name__)

# ── Prompt Constants ────────────────────────────────────────────────────────────

CLASSIFICATION_SYSTEM_PROMPT = """You are an emotional state classifier operating on-device. Analyze the user's text and return ONLY valid JSON with no additional text. Use this exact schema:

{
  "affect_primary": "string — one of: neutral, calm, curious, pleased, hopeful, confused, uncertain, frustrated, angry, sad, anxious, overwhelmed, fatigued, embarrassed, lonely, excited, urgent",
  "affect_valence": "float -1.0 to 1.0 (negative to positive)",
  "affect_arousal": "float 0.0 to 1.0 (calm to excited)",
  "intent_category": "string — one of: practical_guidance, emotional_support, decision_support, venting, planning, clarification, conflict_navigation, reflection, task_execution, creative_help, technical_help, safety_related",
  "risk_level": "string — one of: none, low, medium, high, crisis",
  "anonymized_summary": "brief privacy-safe summary without any names, locations, or identifiers"
}

Rules:
- Return ONLY valid JSON inside ```json ... ``` fences
- Never include names, emails, phone numbers, or other PII in the summary
- If unsure, use conservative values (neutral, 0.0, none)"""


def _build_json_envelope(user_message: str) -> str:
    """Build the JSON-envelope prompt that wraps user text.

    Phi-4-mini-instruct ignores system instructions telling it to "only output JSON"
    but reliably follows the JSON-within-JSON format.
    """
    envelope = json.dumps({
        "instruction": CLASSIFICATION_SYSTEM_PROMPT,
        "schema": "affect_primary (string), affect_valence (float), affect_arousal (float), intent_category (string), risk_level (string), anonymized_summary (string)",
        "user_message": user_message,
    })
    return envelope


# ── Vocabulary Mappings ────────────────────────────────────────────────────────

AFFECT_VOCABULARY: dict[str, AffectPrimary] = {
    "neutral": AffectPrimary.neutral,
    "calm": AffectPrimary.calm,
    "curious": AffectPrimary.curious,
    "pleased": AffectPrimary.pleased,
    "hopeful": AffectPrimary.hopeful,
    "confused": AffectPrimary.confused,
    "uncertain": AffectPrimary.uncertain,
    "frustrated": AffectPrimary.frustrated,
    "anger": AffectPrimary.angry,
    "angry": AffectPrimary.angry,
    "sad": AffectPrimary.sad,
    "sadness": AffectPrimary.sad,
    "anxious": AffectPrimary.anxious,
    "anxiety": AffectPrimary.anxious,
    "overwhelmed": AffectPrimary.overwhelmed,
    "overwhelm": AffectPrimary.overwhelmed,
    "fatigued": AffectPrimary.fatigued,
    "fatigue": AffectPrimary.fatigued,
    "tired": AffectPrimary.fatigued,
    "embarrassed": AffectPrimary.embarrassed,
    "embarrassment": AffectPrimary.embarrassed,
    "lonely": AffectPrimary.lonely,
    "loneliness": AffectPrimary.lonely,
    "excited": AffectPrimary.excited,
    "excitement": AffectPrimary.excited,
    "urgent": AffectPrimary.urgent,
    "urgency": AffectPrimary.urgent,
}

INTENT_VOCABULARY: dict[str, IntentCategory] = {
    "practical_guidance": IntentCategory.practical_guidance,
    "seeking_guidance": IntentCategory.practical_guidance,
    "emotional_support": IntentCategory.emotional_support,
    "support": IntentCategory.emotional_support,
    "decision_support": IntentCategory.decision_support,
    "venting": IntentCategory.venting,
    "planning": IntentCategory.planning,
    "clarification": IntentCategory.clarification,
    "conflict_navigation": IntentCategory.conflict_navigation,
    "reflection": IntentCategory.reflection,
    "task_execution": IntentCategory.task_execution,
    "creative_help": IntentCategory.creative_help,
    "technical_help": IntentCategory.technical_help,
    "tech_help": IntentCategory.technical_help,
    "safety_related": IntentCategory.safety_related,
}

RISK_VOCABULARY: dict[str, RiskLevel] = {
    "none": RiskLevel.none_,
    "low": RiskLevel.low,
    "medium": RiskLevel.medium,
    "high": RiskLevel.high,
    "crisis": RiskLevel.crisis,
}


def _vocabulary_lookup(
    raw: str, vocab: dict[str, Any], default: Any
) -> Any:
    """Look up a string in a vocabulary map with substring fallback."""
    cleaned = raw.strip().lower().strip('"').strip("'")
    # Direct match
    if cleaned in vocab:
        return vocab[cleaned]
    # Substring match
    for key, value in vocab.items():
        if key in cleaned or cleaned in key:
            return value
    return default


# ── Parser ─────────────────────────────────────────────────────────────────────

def _extract_json(text: str) -> dict[str, Any]:
    """Extract JSON object from model output, handling markdown fences.

    Robust against:
    - Markdown fences (```json ... ``` or ``` ... ```)
    - Leading/trailing non-JSON text
    - Empty or whitespace-only output
    - Edge case: fences at very start of string
    """
    raw = text.strip()

    if not raw:
        return {}

    # Step 1: Strip markdown code fences
    while "```" in raw:
        start = raw.index("```")
        # Find the end of this fence
        rest = raw[start + 3:]
        end = rest.index("```") if "```" in rest else len(rest)
        # Content is between the first ``` and the closing ```
        content_before = raw[:start]
        content_inside = rest[:end]
        content_after = rest[end + 3:] if end + 3 <= len(rest) else ""
        # Reconstruct: keep content inside (it's the JSON we want)
        raw = (content_before + content_inside + content_after).strip()
        # Loop in case of nested fences

    # Step 2: Find outermost {...} object
    brace_start = raw.find("{")
    brace_end = raw.rfind("}")
    if brace_start >= 0 and brace_end > brace_start:
        raw = raw[brace_start : brace_end + 1]

    if not raw or raw == "{}":
        return {}

    # Step 3: Parse
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        # Step 4: Last resort — try to find any valid JSON in the text
        for i in range(len(raw)):
            if raw[i] == "{":
                for j in range(len(raw) - 1, i, -1):
                    if raw[j] == "}":
                        candidate = raw[i : j + 1]
                        try:
                            return json.loads(candidate)
                        except json.JSONDecodeError:
                            continue
        raise


def _get_field(data: dict, *keys: str, default: Any = None) -> Any:
    """Get a field from parsed JSON, trying keys in order (flat vs nested)."""
    for key in keys:
        if "." in key:
            # Nested key: "affect.primary"
            parts = key.split(".")
            val = data
            for part in parts:
                if isinstance(val, dict):
                    val = val.get(part)
                else:
                    val = None
                    break
            if val is not None:
                return val
        else:
            if key in data:
                return data[key]
    return default


def parse_classification(raw_output: str) -> EQState:
    """Parse raw model output into an EQState.

    Handles both flat schema (affect_primary) and nested schema (affect.primary),
    matching the Rust parser behavior.
    """
    data = _extract_json(raw_output)

    # Affect
    affect_primary_raw = _get_field(
        data, "affect_primary", "affect.primary", "primary",
        default="unknown"
    )
    affect_valence = float(_get_field(
        data, "affect_valence", "affect.valence", "valence",
        default=0.0
    ))
    affect_arousal = float(_get_field(
        data, "affect_arousal", "affect.arousal", "arousal",
        default=0.0
    ))

    # Intent
    intent_raw = _get_field(
        data, "intent_category", "intent.category", "intent.primary", "category",
        default="unknown"
    )

    # Risk
    risk_raw = _get_field(
        data, "risk_level", "risk.level", "level",
        default="none"
    )

    # Summary
    summary = str(_get_field(
        data, "anonymized_summary", "context.anonymized_summary",
        "summary", "anonymized_summary",
        default=""
    ))

    # Build EQ State
    return EQState(
        affect=AffectState(
            primary=_vocabulary_lookup(affect_primary_raw, AFFECT_VOCABULARY, AffectPrimary.unknown),
            valence=max(-1.0, min(1.0, affect_valence)),
            arousal=max(0.0, min(1.0, affect_arousal)),
            confidence=0.8,
            evidence_type="semantic_inference",
        ),
        intent=IntentState(
            category=_vocabulary_lookup(intent_raw, INTENT_VOCABULARY, IntentCategory.unknown),
            confidence=0.7,
        ),
        risk=RiskState(
            level=_vocabulary_lookup(risk_raw, RISK_VOCABULARY, RiskLevel.unknown),
            signals=[],
            confidence=0.7,
        ),
        privacy=PrivacyState(
            sensitivity_level=PrivacySensitivity.medium,
            pii_removed=False,
        ),
        context=ContextState(
            anonymized_summary=summary,
        ),
    )


# ── EQ Client ──────────────────────────────────────────────────────────────────

class EQClientError(Exception):
    """Raised when the backend SLM returns an error or times out."""


class EQClient:
    """HTTP client for the llama-server-mini backend.

    Mirrors the Rust LlamaCppAdapter's TcpStream-based HTTP/1.1 client.
    """

    def __init__(
        self,
        host: str | None = None,
        port: int | None = None,
        timeout_sec: int | None = None,
        max_tokens: int | None = None,
        temperature: float | None = None,
    ):
        self.base_url = f"http://{host or settings.backend_host}:{port or settings.backend_port}"
        self.timeout_sec = timeout_sec or settings.backend_timeout_sec
        self.max_tokens = max_tokens or settings.backend_max_tokens
        self.temperature = temperature or settings.backend_temperature

    def health_check(self) -> bool:
        """Check if the backend is reachable."""
        try:
            resp = httpx.get(
                f"{self.base_url}/health",
                timeout=5.0,
            )
            return resp.status_code == 200
        except Exception:
            return False

    def status(self) -> str:
        """Get human-readable status string."""
        if self.health_check():
            return f"connected to {self.base_url} (phi-4)"
        return f"backend unreachable at {self.base_url}"

    def _reset_context(self) -> None:
        """Reset the backend context to prevent context overflow.

        llama-server-mini accumulates context across requests if slots fill up.
        This mirrors the Rust strategy of fresh connections per request.
        """
        try:
            httpx.post(
                f"{self.base_url}/reset",
                timeout=5.0,
                headers={"Connection": "close"},
            )
        except Exception:
            pass  # Reset is best-effort

    def classify(self, text: str, session_id: str | None = None) -> EQState:
        """Send text to the backend SLM for classification.

        This mirrors the Rust implementation:
        1. Build JSON-envelope prompt
        2. Reset backend context (prevents overflow)
        3. POST to /v1/chat/completions via fresh connection
        4. Parse the response with dual-schema support
        """
        # Reset context before each request (like Rust's fresh TCP connection)
        self._reset_context()

        prompt = _build_json_envelope(text)
        url = f"{self.base_url}/v1/chat/completions"

        payload = {
            "messages": [
                {"role": "system", "content": CLASSIFICATION_SYSTEM_PROMPT},
                {"role": "user", "content": prompt},
            ],
            "max_tokens": self.max_tokens,
            "temperature": self.temperature,
            "stream": False,
        }

        logger.debug("POST %s (session=%s, text_len=%d)", url, session_id, len(text))

        # Use a fresh client with Connection: close to match Rust TcpStream behavior
        try:
            with httpx.Client() as client:
                resp = client.post(
                    url,
                    json=payload,
                    timeout=self.timeout_sec,
                    headers={"Connection": "close"},
                )
                resp.raise_for_status()
        except httpx.TimeoutException:
            raise EQClientError(
                f"Backend timeout after {self.timeout_sec}s"
            )
        except httpx.HTTPStatusError as e:
            raise EQClientError(
                f"Backend HTTP {e.response.status_code}: {e.response.text[:200]}"
            )
        except httpx.RequestError as e:
            raise EQClientError(
                f"Backend connection error: {e}"
            )

        body = resp.json()
        try:
            raw_output = body["choices"][0]["message"]["content"]
        except (KeyError, IndexError) as e:
            raise EQClientError(
                f"Unexpected response format: {e} — body: {json.dumps(body)[:200]}"
            )

        # Handle context-full error from server
        if raw_output.strip() == "[context full]":
            raise EQClientError("Backend context is full — try resetting the server")

        return parse_classification(raw_output)

    def classify_with_latency(self, text: str, session_id: str | None = None) -> tuple[EQState, int]:
        """Classify and return (EQState, latency_ms)."""
        start = time.perf_counter()
        result = self.classify(text, session_id)
        elapsed = int((time.perf_counter() - start) * 1000)
        return result, elapsed
