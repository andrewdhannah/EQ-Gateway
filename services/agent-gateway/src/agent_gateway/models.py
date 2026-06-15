"""Pydantic models for request/response, EQ State, and routing."""

from datetime import datetime, timezone
from enum import Enum
from typing import Any
from uuid import uuid4

from pydantic import BaseModel, Field


# ── Enums ──────────────────────────────────────────────────────────────────────

class AffectPrimary(str, Enum):
    neutral = "neutral"
    calm = "calm"
    curious = "curious"
    pleased = "pleased"
    hopeful = "hopeful"
    confused = "confused"
    uncertain = "uncertain"
    frustrated = "frustrated"
    angry = "angry"
    sad = "sad"
    anxious = "anxious"
    overwhelmed = "overwhelmed"
    fatigued = "fatigued"
    embarrassed = "embarrassed"
    lonely = "lonely"
    excited = "excited"
    urgent = "urgent"
    unknown = "unknown"


class IntentCategory(str, Enum):
    practical_guidance = "practical_guidance"
    emotional_support = "emotional_support"
    decision_support = "decision_support"
    venting = "venting"
    planning = "planning"
    clarification = "clarification"
    conflict_navigation = "conflict_navigation"
    reflection = "reflection"
    task_execution = "task_execution"
    creative_help = "creative_help"
    technical_help = "technical_help"
    safety_related = "safety_related"
    unknown = "unknown"


class RiskLevel(str, Enum):
    none_ = "none"
    low = "low"
    medium = "medium"
    high = "high"
    crisis = "crisis"
    unknown = "unknown"


class PrivacySensitivity(str, Enum):
    public = "public"
    low = "low"
    medium = "medium"
    high = "high"
    restricted = "restricted"
    unknown = "unknown"


class ResponseTone(str, Enum):
    calm_direct = "calm_direct"
    gentle_direct = "gentle_direct"
    warm_brief = "warm_brief"
    neutral_professional = "neutral_professional"
    highly_practical = "highly_practical"
    reflective = "reflective"
    encouraging = "encouraging"
    minimal = "minimal"
    unknown = "unknown"


class RoutingDecision(str, Enum):
    """What the model router decides to do with the request."""
    local_only = "local_only"
    cloud_with_metadata = "cloud_with_metadata"
    requires_human_approval = "requires_human_approval"
    block = "block"


class HumanDecision(str, Enum):
    approve = "approve"
    reject = "reject"
    edit = "edit"
    timed_out = "timed_out"


class FinalStatus(str, Enum):
    completed = "completed"
    rejected_by_human = "rejected_by_human"
    rejected_by_policy = "rejected_by_policy"
    timed_out = "timed_out"
    error = "error"


# ── Core Models ────────────────────────────────────────────────────────────────

class SessionInfo(BaseModel):
    ephemeral_session_id: str = Field(default_factory=lambda: str(uuid4()))
    timestamp_local: str = Field(
        default_factory=lambda: datetime.now(timezone.utc).isoformat()
    )
    device_processing_only: bool = True


class AffectState(BaseModel):
    primary: AffectPrimary = AffectPrimary.unknown
    secondary: list[AffectPrimary] = []
    valence: float = 0.0
    arousal: float = 0.0
    confidence: float = 0.0
    evidence_type: str = "semantic_inference"


class IntentState(BaseModel):
    category: IntentCategory = IntentCategory.unknown
    subtype: str = ""
    confidence: float = 0.0


class RiskState(BaseModel):
    level: RiskLevel = RiskLevel.none_
    signals: list[str] = []
    confidence: float = 0.0
    requires_local_escalation: bool = False


class PrivacyState(BaseModel):
    sensitivity_level: PrivacySensitivity = PrivacySensitivity.unknown
    raw_text_shared: bool = False
    pii_removed: bool = False
    sensitive_domains_detected: list[str] = []
    redaction_confidence: float = 0.0


class ResponsePolicy(BaseModel):
    tone: ResponseTone = ResponseTone.neutral_professional
    warmth: float = 0.5
    directness: float = 0.5
    length: str = "medium"
    pace: str = "steady"
    max_followup_questions: int = 2
    format: str = "prose"


class ContextState(BaseModel):
    anonymized_summary: str = ""
    included_raw_excerpt: bool = False
    retrieval_notes_included: bool = False


class EQState(BaseModel):
    """Full EQ State payload — mirrors eq_state_schema.json v0.1."""
    schema_version: str = "0.1"
    session: SessionInfo = Field(default_factory=SessionInfo)
    affect: AffectState = Field(default_factory=AffectState)
    intent: IntentState = Field(default_factory=IntentState)
    risk: RiskState = Field(default_factory=RiskState)
    privacy: PrivacyState = Field(default_factory=PrivacyState)
    response_policy: ResponsePolicy = Field(default_factory=ResponsePolicy)
    context: ContextState = Field(default_factory=ContextState)


# ── Request / Response ─────────────────────────────────────────────────────────

class ChatRequest(BaseModel):
    """Incoming chat message to analyze."""
    message: str
    session_id: str | None = None
    user_id: str | None = None
    prefer_cloud: bool = False


class HITLRequest(BaseModel):
    """Human decision on a paused request."""
    request_id: str
    decision: HumanDecision
    edited_message: str | None = None
    reviewer_notes: str | None = None


class RouteResult(BaseModel):
    """Result of the model routing step."""
    decision: RoutingDecision
    reason: str
    risk_level: RiskLevel
    sensitivity_level: PrivacySensitivity


class ChatResponse(BaseModel):
    """Full response from the agent gateway."""
    request_id: str
    eq_state: EQState
    route: RouteResult
    human_decision: HumanDecision | None = None
    final_status: FinalStatus
    response_text: str | None = None
    receipt_path: str | None = None


# ── Receipt ────────────────────────────────────────────────────────────────────

class ObservabilityReceipt(BaseModel):
    """Structured receipt emitted for every request."""
    request_id: str
    timestamp: str = Field(
        default_factory=lambda: datetime.now(timezone.utc).isoformat()
    )
    raw_text_left_device: bool
    pii_detected: bool
    redacted_categories: list[str]
    affect_primary: str
    risk_level: str
    model_route: str
    human_approval_required: bool
    human_decision: str | None = None
    final_status: str
    latency_ms: int
    schema_version: str = "receipt-v0.1"
