"""
EQ State JSON Schema Validator.

Provides validation of EQ State payloads against the canonical schema
derived from eq-state.schema.ts (Zod).
"""

from __future__ import annotations

import json
import logging
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import jsonschema

logger = logging.getLogger(__name__)

# Path to the JSON Schema file bundled with the package
_SCHEMA_DIR = Path(__file__).resolve().parent.parent.parent / "schema"
_DEFAULT_SCHEMA_PATH = _SCHEMA_DIR / "eq_state_schema.json"


@dataclass
class ValidationResult:
    """Result of an EQ State validation."""

    valid: bool
    errors: list[str] = field(default_factory=list)
    data: dict[str, Any] | None = None

    def __bool__(self) -> bool:
        return self.valid

    def __str__(self) -> str:
        if self.valid:
            return "✓ EQ State validates against schema"
        return f"✗ EQ State validation failed ({len(self.errors)} errors):\n  " + "\n  ".join(self.errors)


class EQStateValidator:
    """
    Validates EQ State JSON payloads against the canonical schema.

    Usage:
        validator = EQStateValidator()
        result = validator.validate_state({"schema_version": "0.1", ...})
        assert result.valid
    """

    def __init__(self, schema_path: str | Path | None = None) -> None:
        """
        Initialize the validator with a JSON Schema file.

        Args:
            schema_path: Path to the JSON Schema file. Defaults to the
                         bundled eq_state_schema.json.
        """
        path = Path(schema_path) if schema_path else _DEFAULT_SCHEMA_PATH
        if not path.exists():
            raise FileNotFoundError(f"EQ State schema not found at: {path}")

        with open(path, "r", encoding="utf-8") as f:
            self._schema: dict[str, Any] = json.load(f)

        # Compile the schema for performance
        self._validator = jsonschema.Draft7Validator(self._schema)
        logger.info("Loaded EQ State schema from %s", path)

    @property
    def schema(self) -> dict[str, Any]:
        """Return the loaded JSON Schema."""
        return self._schema

    def validate_state(self, state: dict[str, Any]) -> ValidationResult:
        """
        Validate an EQ State dictionary against the schema.

        Args:
            state: The EQ State payload as a Python dict.

        Returns:
            A ValidationResult with valid flag and any error messages.
        """
        errors: list[str] = []
        for error in self._validator.iter_errors(state):
            path = " → ".join(str(p) for p in error.absolute_path) or "(root)"
            errors.append(f"{path}: {error.message}")

        return ValidationResult(
            valid=len(errors) == 0,
            errors=errors,
            data=state if len(errors) == 0 else None,
        )

    def validate_json(self, json_str: str) -> ValidationResult:
        """
        Validate an EQ State JSON string against the schema.

        Args:
            json_str: The EQ State payload as a JSON string.

        Returns:
            A ValidationResult with valid flag and any error messages.
        """
        try:
            state = json.loads(json_str)
        except json.JSONDecodeError as e:
            return ValidationResult(valid=False, errors=[f"Invalid JSON: {e}"])

        return self.validate_state(state)

    def __repr__(self) -> str:
        return f"<EQStateValidator schema_version={self._schema.get('$id', 'unknown')}>"
