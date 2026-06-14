# Python Evaluation Harness — The Scientist

**Status:** Design Draft  
**Version:** 0.1  
**Date:** June 08, 2026  
**Layer:** Research Layer (The Scientist)

---

## 1. Purpose

The Python Evaluation Harness is the **bridge between research and production**. Its jobs:

1. **Validate** that a given SLM (quantized or raw) produces output matching the EQ State schema.
2. **Quantize** model weights and KV cache, then measure the accuracy loss.
3. **Benchmark** inference speed across mobile-relevant hardware targets.
4. **Generate** compliance evidence (PIPEDA/GDPR) by proving PII redaction accuracy.
5. **Export** a "model card" that the Rust engine's CI consumes to accept/reject a model.

---

## 2. Project Structure

```
evaluation/
├── pyproject.toml               # Python project config (poetry or pip)
├── requirements.txt             # Pinned dependencies
├── config/
│   ├── eval_config.yaml         # Master evaluation configuration
│   ├── schema_overrides.yaml    # Tolerance thresholds for validation
│   └── pii_patterns.yaml        # Canadian PII patterns for redaction tests
│
├── harness/
│   ├── __init__.py
│   ├── schema_validator.py      # Zod-equivalent schema validation in Python
│   ├── model_runner.py          # Loads model, runs inference, collects output
│   ├── pii_scanner.py           # Scans model output for residual PII
│   ├── quantize_benchmark.py    # Quantization accuracy/speed measurement
│   ├── compliance_reporter.py   # Generates PIPEDA/GDPR evidence reports
│   └── model_card_exporter.py   # Exports the model acceptance card (JSON)
│
├── schemas/
│   ├── __init__.py
│   ├── eq_state_schema.py       # Python port of the TypeScript Zod schema
│   └── mcp_tools_schema.py      # Python port of the MCP tool definitions
│
├── test_corpus/
│   ├── canadian_pii/            # 100+ synthetic Canadian PII examples
│   │   ├── sin_numbers.txt       # Social Insurance Numbers
│   │   ├── health_cards.txt      # Provincial health card formats
│   │   ├── addresses.txt         # Canadian address formats
│   │   ├── banking.txt           # Transit/account numbers
│   │   └── passports.txt         # Passport number formats
│   ├── emotional_states/        # Prompt → expected affect/intent pairs
│   │   ├── frustration.jsonl
│   │   ├── anxiety.jsonl
│   │   ├── grief.jsonl
│   │   └── neutral.jsonl
│   └── edge_cases/              # Boundary and adversarial inputs
│       ├── empty_string.txt
│       ├── extremely_long.txt
│       ├── mixed_language.txt
│       └── adversarial_pii.txt   # PII hidden in creative text
│
├── tests/
│   ├── test_schema_validator.py
│   ├── test_pii_scanner.py
│   ├── test_quantize_benchmark.py
│   └── test_compliance_reporter.py
│
├── scripts/
│   ├── run_full_eval.py         # End-to-end evaluation pipeline
│   ├── quick_validation.py      # Fast smoke test (for dev iteration)
│   └── export_model_card.py     # Standalone model card exporter
│
└── output/
    ├── eval_report_latest.json  # Most recent evaluation results
    ├── compliance_report/       # Generated compliance PDFs
    └── model_cards/             # Exported model card JSON files
```

---

## 3. Schema Validation (Python Port)

Since we defined the Source of Truth in TypeScript/Zod, the Python harness needs an equivalent. We use `pydantic` for strict validation with field-level constraints:

```python
# harness/schema_validator.py

"""
Schema validation engine for EQ State payloads.
Mirrors the TypeScript Zod schemas from schemas/eq-state.schema.ts.
Uses pydantic v2 for strict typing and validation.
"""

from pydantic import BaseModel, Field, field_validator
from typing import Literal
from enum import Enum
from datetime import datetime


# ---------------------------------------------------------------------------
# 1. CONTROLLED VOCABULARIES (Mirrors TypeScript enums)
# ---------------------------------------------------------------------------

class AffectPrimary(str, Enum):
    NEUTRAL = "neutral"
    CALM = "calm"
    CURIOUS = "curious"
    PLEASED = "pleased"
    HOPEFUL = "hopeful"
    CONFUSED = "confused"
    UNCERTAIN = "uncertain"
    FRUSTRATED = "frustrated"
    ANGRY = "angry"
    SAD = "sad"
    ANXIOUS = "anxious"
    OVERWHELMED = "overwhelmed"
    FATIGUED = "fatigued"
    EMBARRASSED = "embarrassed"
    LONELY = "lonely"
    EXCITED = "excited"
    URGENT = "urgent"
    UNKNOWN = "unknown"


class IntentCategory(str, Enum):
    PRACTICAL_GUIDANCE = "practical_guidance"
    EMOTIONAL_SUPPORT = "emotional_support"
    DECISION_SUPPORT = "decision_support"
    VENTING = "venting"
    PLANNING = "planning"
    CLARIFICATION = "clarification"
    CONFLICT_NAVIGATION = "conflict_navigation"
    REFLECTION = "reflection"
    TASK_EXECUTION = "task_execution"
    CREATIVE_HELP = "creative_help"
    TECHNICAL_HELP = "technical_help"
    SAFETY_RELATED = "safety_related"
    UNKNOWN = "unknown"


class RiskLevel(str, Enum):
    NONE = "none"
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRISIS = "crisis"
    UNKNOWN = "unknown"


class PrivacySensitivity(str, Enum):
    PUBLIC = "public"
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    RESTRICTED = "restricted"
    UNKNOWN = "unknown"


class ResponseTone(str, Enum):
    CALM_DIRECT = "calm_direct"
    GENTLE_DIRECT = "gentle_direct"
    WARM_BRIEF = "warm_brief"
    NEUTRAL_PROFESSIONAL = "neutral_professional"
    HIGHLY_PRACTICAL = "highly_practical"
    REFLECTIVE = "reflective"
    ENCOURAGING = "encouraging"
    MINIMAL = "minimal"
    UNKNOWN = "unknown"


# ---------------------------------------------------------------------------
# 2. SUB-MODELS (Mirrors TypeScript sub-schemas)
# ---------------------------------------------------------------------------

class AffectState(BaseModel):
    """Quantitative and qualitative measures of current emotion."""
    primary: AffectPrimary
    secondary: list[AffectPrimary] = Field(default_factory=list)
    valence: float = Field(ge=-1.0, le=1.0)
    arousal: float = Field(ge=0.0, le=1.0)
    confidence: float = Field(ge=0.0, le=1.0)
    evidence_type: str


class IntentState(BaseModel):
    """Classification of what the user is attempting to achieve."""
    category: IntentCategory
    subtype: str
    confidence: float = Field(ge=0.0, le=1.0)


class RiskState(BaseModel):
    """Safety assessment for escalation."""
    level: RiskLevel
    signals: list[str] = Field(default_factory=list)
    confidence: float = Field(ge=0.0, le=1.0)
    requires_local_escalation: bool = False


class PrivacyState(BaseModel):
    """Metadata regarding data sanitization and disclosure limits."""
    sensitivity_level: PrivacySensitivity
    raw_text_shared: bool = False
    pii_removed: bool = False
    sensitive_domains_detected: list[str] = Field(default_factory=list)
    redaction_confidence: float = Field(ge=0.0, le=1.0)


class ResponsePolicy(BaseModel):
    """Behavioral constraints for the Large AI."""
    tone: ResponseTone
    warmth: float = Field(ge=0.0, le=1.0)
    directness: float = Field(ge=0.0, le=1.0)
    length: Literal["short", "medium", "long"]
    pace: str = "steady"
    max_followup_questions: int = Field(ge=0, default=2)
    format: str = "prose"


class ContextState(BaseModel):
    """Sanitized, anonymized summary of the dialogue."""
    anonymized_summary: str
    included_raw_excerpt: bool = False
    retrieval_notes_included: bool = False


# ---------------------------------------------------------------------------
# 3. MASTER MODEL (Mirrors EQStateSchema)
# ---------------------------------------------------------------------------

class EQState(BaseModel):
    """The complete EQ State payload. Mirrors EQStateSchema from TypeScript."""

    schema_version: str = "0.1"
    session: dict = Field(default_factory=lambda: {
        "ephemeral_session_id": "00000000-0000-0000-0000-000000000000",
        "timestamp_local": datetime.utcnow().isoformat() + "Z",
        "device_processing_only": True
    })
    affect: AffectState
    intent: IntentState
    risk: RiskState
    privacy: PrivacyState
    response_policy: ResponsePolicy
    context: ContextState

    @field_validator("schema_version")
    @classmethod
    def version_must_match(cls, v: str) -> str:
        """Ensure the schema version is recognized by the Rust engine."""
        allowed_versions = {"0.1", "0.2"}
        if v not in allowed_versions:
            raise ValueError(f"Unknown schema version: {v}. Must be one of {allowed_versions}")
        return v
```

---

## 4. PII Redaction Accuracy Testing

This is the most important test for regulatory compliance. The SLM must **never** leak PII into the `anonymized_summary` field.

```python
# harness/pii_scanner.py

"""
PII scanning engine for validating EQ Gateway redaction.
Tests the SLM's anonymization output against known Canadian PII patterns.
"""

import re
from pathlib import Path
from typing import List, Dict, Tuple


class PiiPattern:
    """A single PII pattern with metadata for compliance reporting."""

    def __init__(
        self,
        name: str,
        pattern: str,
        category: str,
        severity: str,          # 'critical' | 'high' | 'medium' | 'low'
        regulation: str,         # e.g., 'PIPEDA', 'PHIPA', 'GDPR'
    ):
        self.name = name
        self.regex = re.compile(pattern)
        self.category = category
        self.severity = severity
        self.regulation = regulation


# ---------------------------------------------------------------------------
# CANADIAN PII PATTERN LIBRARY
# ---------------------------------------------------------------------------

CANADIAN_PII_PATTERNS: List[PiiPattern] = [
    # Social Insurance Number (SIN)
    PiiPattern(
        name="SIN",
        pattern=r"\b\d{3}[ -]?\d{3}[ -]?\d{3}\b",
        category="government_id",
        severity="critical",
        regulation="PIPEDA",
    ),
    # Canadian passport numbers
    PiiPattern(
        name="Canadian Passport",
        pattern=r"\b[A-Z]{2}\d{6}\b",
        category="government_id",
        severity="critical",
        regulation="PIPEDA",
    ),
    # Provincial health card numbers (Ontario: 10 digits, Quebec: 12 digits + letter, etc.)
    PiiPattern(
        name="Ontario Health Card",
        pattern=r"\b\d{4}[ -]?\d{3}[ -]?\d{3}[ -]?\d{4}\b",
        category="health_id",
        severity="critical",
        regulation="PHIPA",
    ),
    # Canadian postal codes
    PiiPattern(
        name="Canadian Postal Code",
        pattern=r"\b[A-Za-z]\d[A-Za-z][ -]?\d[A-Za-z]\d\b",
        category="address",
        severity="high",
        regulation="PIPEDA",
    ),
    # Email addresses
    PiiPattern(
        name="Email",
        pattern=r"\b[\w\.-]+@[\w\.-]+\.\w{2,}\b",
        category="contact",
        severity="high",
        regulation="PIPEDA",
    ),
    # Phone numbers (Canadian format: +1 or 1 followed by 10 digits)
    PiiPattern(
        name="Canadian Phone",
        pattern=r"\b(?:1[-.]?)?\(?\d{3}\)?[-.]?\d{3}[-.]?\d{4}\b",
        category="contact",
        severity="high",
        regulation="PIPEDA",
    ),
]


class PiiScanner:
    """
    Scans text for PII patterns and returns a detailed report.
    Used to validate that the SLM's anonymization is effective.
    """

    def __init__(self, patterns: List[PiiPattern] = None):
        self.patterns = patterns or CANADIAN_PII_PATTERNS

    def scan(self, text: str) -> Dict:
        """
        Scan a string for all registered PII patterns.

        Returns:
            A dict with:
            - 'clean': bool (True if no PII found)
            - 'matches': list of {pattern_name, match_text, position, category, severity}
            - 'match_count': total number of PII matches
            - 'critical_count': number of critical-severity matches
        """
        matches = []
        for pattern in self.patterns:
            for m in pattern.regex.finditer(text):
                matches.append({
                    "pattern_name": pattern.name,
                    "match_text": m.group(),
                    "position": m.start(),
                    "category": pattern.category,
                    "severity": pattern.severity,
                    "regulation": pattern.regulation,
                })

        return {
            "clean": len(matches) == 0,
            "matches": matches,
            "match_count": len(matches),
            "critical_count": sum(1 for m in matches if m["severity"] == "critical"),
        }

    def scan_corpus(self, corpus_dir: Path) -> Dict:
        """
        Scan an entire test corpus directory.
        Returns aggregate statistics for compliance reporting.
        """
        results = {
            "files_scanned": 0,
            "total_matches": 0,
            "critical_leaks": 0,
            "files_with_leaks": [],
            "pattern_breakdown": {},
        }

        for file_path in corpus_dir.rglob("*"):
            if not file_path.is_file():
                continue
            if file_path.suffix in {".txt", ".jsonl", ".md"}:
                text = file_path.read_text(encoding="utf-8")
                scan_result = self.scan(text)
                results["files_scanned"] += 1
                results["total_matches"] += scan_result["match_count"]
                results["critical_leaks"] += scan_result["critical_count"]

                if not scan_result["clean"]:
                    results["files_with_leaks"].append(str(file_path))

                for m in scan_result["matches"]:
                    pname = m["pattern_name"]
                    if pname not in results["pattern_breakdown"]:
                        results["pattern_breakdown"][pname] = 0
                    results["pattern_breakdown"][pname] += 1

        return results
```

---

## 5. Full Evaluation Pipeline

```python
# scripts/run_full_eval.py

"""
End-to-end evaluation pipeline.

Usage:
    python scripts/run_full_eval.py --model path/to/model.gguf \\
                                    --corpus test_corpus/ \\
                                    --output output/eval_report_latest.json

This script:
1. Loads the model and runs inference on the test corpus.
2. Validates every output against the EQ State schema.
3. Scans every anonymized_summary for PII leaks.
4. Benchmarks inference speed.
5. Generates a compliance evidence report.
6. Exports a model card JSON for the Rust engine CI.
"""

import argparse
import json
import time
from pathlib import Path
from datetime import datetime

from harness.schema_validator import EQState
from harness.model_runner import ModelRunner
from harness.pii_scanner import PiiScanner
from harness.quantize_benchmark import QuantizeBenchmark
from harness.compliance_reporter import ComplianceReporter
from harness.model_card_exporter import ModelCardExporter


def parse_args():
    parser = argparse.ArgumentParser(description="EQ Gateway Full Evaluation Pipeline")
    parser.add_argument("--model", required=True, help="Path to quantized model file")
    parser.add_argument("--corpus", required=True, help="Path to test corpus directory")
    parser.add_argument("--output", default="output/eval_report_latest.json")
    parser.add_argument("--quick", action="store_true", help="Run fast smoke test only")
    return parser.parse_args()


def run_evaluation(args) -> dict:
    """Run the full evaluation pipeline and return results."""

    results = {
        "eval_timestamp": datetime.utcnow().isoformat() + "Z",
        "model_path": args.model,
        "schema_version": "0.2",
        "status": "in_progress",
    }

    # ------------------------------------------------------------------
    # Phase 1: Model Loading & Inference
    # ------------------------------------------------------------------
    print(f"[1/5] Loading model: {args.model}")
    runner = ModelRunner(args.model)
    runner.load()

    print(f"[2/5] Running inference on corpus: {args.corpus}")
    inference_results = runner.run_corpus(
        corpus_dir=Path(args.corpus),
        quick=args.quick,
    )
    results["inference"] = {
        "total_prompts": inference_results["total"],
        "successful": inference_results["successful"],
        "failed": inference_results["failed"],
        "avg_latency_ms": inference_results["avg_latency_ms"],
        "outputs": inference_results["outputs"],  # List of raw model outputs
    }

    # ------------------------------------------------------------------
    # Phase 2: Schema Validation
    # ------------------------------------------------------------------
    print("[3/5] Validating outputs against EQ State schema")
    validation_results = {"passed": 0, "failed": 0, "errors": []}

    for i, output in enumerate(inference_results["outputs"]):
        try:
            parsed = EQState.model_validate_json(output["eq_state_json"])
            validation_results["passed"] += 1
        except Exception as e:
            validation_results["failed"] += 1
            validation_results["errors"].append({
                "prompt_index": i,
                "prompt_preview": output["prompt"][:100],
                "error": str(e),
            })

    results["validation"] = validation_results

    # ------------------------------------------------------------------
    # Phase 3: PII Leak Scan
    # ------------------------------------------------------------------
    print("[4/5] Scanning anonymized summaries for PII leaks")
    scanner = PiiScanner()
    pii_results = {"total_clean": 0, "total_leaks": 0, "leak_details": []}

    for i, output in enumerate(inference_results["outputs"]):
        summary = output.get("anonymized_summary", "")
        scan = scanner.scan(summary)
        if scan["clean"]:
            pii_results["total_clean"] += 1
        else:
            pii_results["total_leaks"] += 1
            pii_results["leak_details"].append({
                "prompt_index": i,
                "prompt_preview": output["prompt"][:100],
                "matches": scan["matches"],
            })

    results["pii_scan"] = pii_results

    # ------------------------------------------------------------------
    # Phase 4: Benchmark (skip if --quick)
    # ------------------------------------------------------------------
    if not args.quick:
        print("[5/5] Running quantization benchmark")
        benchmark = QuantizeBenchmark()
        benchmark_results = benchmark.run(model_path=args.model)
        results["benchmark"] = benchmark_results
    else:
        results["benchmark"] = {"skipped": True, "reason": "Quick mode"}

    # ------------------------------------------------------------------
    # Final Status
    # ------------------------------------------------------------------
    results["status"] = "completed"
    results["summary"] = {
        "schema_pass_rate": f"{validation_results['passed'] / (validation_results['passed'] + validation_results['failed']) * 100:.1f}%" if (validation_results['passed'] + validation_results['failed']) > 0 else "N/A",
        "pii_leak_rate": f"{pii_results['total_leaks'] / (pii_results['total_clean'] + pii_results['total_leaks']) * 100:.1f}%" if (pii_results['total_clean'] + pii_results['total_leaks']) > 0 else "N/A",
        "recommendation": "PASS" if (validation_results['failed'] == 0 and pii_results['total_leaks'] == 0) else "FAIL",
    }

    return results


def main():
    args = parse_args()
    results = run_evaluation(args)

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(results, indent=2))

    print(f"\nEvaluation complete. Report written to: {output_path}")
    print(f"  Schema pass rate: {results['summary']['schema_pass_rate']}")
    print(f"  PII leak rate:    {results['summary']['pii_leak_rate']}")
    print(f"  Recommendation:    {results['summary']['recommendation']}")


if __name__ == "__main__":
    main()
```

---

## 6. Model Card Exporter

The model card is the **contract between research and production**. The Rust engine's CI reads this to decide whether to accept a new model.

```python
# harness/model_card_exporter.py

"""
Exports a model card JSON that the Rust engine CI consumes.
This is the artifact that gates model deployment.
"""

from dataclasses import dataclass, asdict
from datetime import datetime
import json


@dataclass
class ModelCard:
    """Immutable model acceptance card for CI/CD."""

    # Identity
    model_name: str
    model_version: str
    architecture: str          # e.g., "Llama-3B", "Phi-2.7B"
    quantization: str          # e.g., "Q4_K_M", "FP8_KV"

    # Performance metrics (PASS/FAIL thresholds for CI)
    inference_latency_ms: float
    memory_footprint_mb: float
    schema_compliance_pct: float   # Must be 100%
    pii_leak_rate_pct: float       # Must be 0%
    perplexity_score: float

    # Validation
    test_corpus_size: int
    validation_timestamp: str

    # Decision
    approved: bool                  # Set by evaluation pipeline
    approver: str = "eval_harness"  # Automated

    def to_json(self, path: str) -> None:
        """Write the model card to a JSON file for CI consumption."""
        data = asdict(self)
        with open(path, "w") as f:
            json.dump(data, f, indent=2)

    @staticmethod
    def ci_thresholds() -> dict:
        """CI gate thresholds. If any metric falls outside these, the build fails."""
        return {
            "schema_compliance_pct": 100.0,   # No exceptions
            "pii_leak_rate_pct": 0.0,          # Zero tolerance
            "inference_latency_ms_max": 500.0, # Snapdragon 8 Elite target
            "memory_footprint_mb_max": 2048.0, # 2GB max for model + cache
        }
```

---

## 7. Test Corpus (Canadian PII Focus)

The test corpus is designed to match the specific privacy risks of Canadian users:

| File | Content | Purpose |
|:---|:---|:---|
| `sin_numbers.txt` | 20 valid-format SIN numbers in context sentences | Test SIN redaction |
| `health_cards.txt` | Ontario (OHIP), Quebec (RAMQ), BC (MSP) formats | Test health data redaction |
| `addresses.txt` | Canadian addresses with postal codes | Test location redaction |
| `banking.txt` | Transit numbers, institution numbers, account numbers | Test financial PII |
| `passports.txt` | Passport numbers embedded in conversation | Test travel document redaction |
| `adversarial_pii.txt` | PII hidden inside prose, split across lines, leetspeak | Test evasion resistance |

---

## 8. CI/CD Integration

```yaml
# .github/workflows/model-eval.yml

name: Model Evaluation

on:
  push:
    paths:
      - 'models/**'
      - 'evaluation/**'

jobs:
  evaluate:
    runs-on: ubuntu-latest  # GPU-enabled runner
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      
      - name: Install dependencies
        run: |
          cd evaluation
          pip install -r requirements.txt
      
      - name: Run full evaluation
        run: |
          cd evaluation
          python scripts/run_full_eval.py \
            --model ../models/eq-gateway-1b-q4.gguf \
            --corpus test_corpus/ \
            --output output/eval_report.json
      
      - name: Check model card approval
        run: |
          cd evaluation
          python -c "
            import json
            with open('output/eval_report.json') as f:
              report = json.load(f)
            if report['summary']['recommendation'] != 'PASS':
              print('Model evaluation FAILED. See report for details.')
              exit(1)
            print('Model evaluation PASSED.')
          "
      
      - name: Upload evaluation artifacts
        uses: actions/upload-artifact@v4
        with:
          name: eval-report
          path: evaluation/output/
```

---

## 9. Dependencies

```txt
# requirements.txt

# Core validation (mirrors Zod)
pydantic>=2.0,<3.0

# Model inference
torch>=2.1.0
transformers>=4.36.0
accelerate>=0.25.0
sentencepiece>=0.1.99

# Quantization
bitsandbytes>=0.41.0
auto-gptq>=0.5.0

# PII scanning
regex>=2023.12.25

# Benchmarking & reporting
psutil>=5.9.0
pandas>=2.1.0

# Testing
pytest>=7.4.0
pytest-cov>=4.1.0

# CLI
pyyaml>=6.0.0
typer>=0.9.0
```

---

*Confidential — EQ Gateway Project*
