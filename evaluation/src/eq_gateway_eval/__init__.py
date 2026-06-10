"""
EQ Gateway Evaluation Harness.

Validates EQ State JSON payloads against the TypeScript Zod schema
(converted to JSON Schema). Used for integration testing between
the Rust engine, Python eval tools, and mobile stack.
"""

from .validator import EQStateValidator, ValidationResult

__all__ = ["EQStateValidator", "ValidationResult"]
__version__ = "0.1.0"
