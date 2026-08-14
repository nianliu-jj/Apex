#!/usr/bin/env python3
"""Validate Apex Feature Spec template frontmatter without third-party packages."""

from __future__ import annotations

import json
import re
import sys
from datetime import datetime
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "schemas" / "feature-spec-frontmatter.schema.json"
TEMPLATE_DIR = ROOT / "specs" / "_templates"
FIXTURE_DIR = ROOT / "schemas" / "fixtures" / "spec-frontmatter"
DOCUMENTS = ("requirements", "design", "tasks", "verification")


class ValidationError(ValueError):
    """Raised when a frontmatter value violates the supported schema subset."""


def value_type_matches(value: Any, expected: str) -> bool:
    """Return whether a Python value matches one JSON Schema primitive type."""
    return {
        "object": isinstance(value, dict),
        "array": isinstance(value, list),
        "string": isinstance(value, str),
        "integer": isinstance(value, int) and not isinstance(value, bool),
        "boolean": isinstance(value, bool),
        "null": value is None,
    }.get(expected, False)


def validate(value: Any, schema: dict[str, Any], path: str = "$") -> None:
    """Validate the JSON Schema keywords used by EP-0001."""
    if "oneOf" in schema:
        matches = 0
        reasons: list[str] = []
        for candidate in schema["oneOf"]:
            try:
                validate(value, candidate, path)
                matches += 1
            except ValidationError as error:
                reasons.append(str(error))
        if matches != 1:
            detail = "; ".join(reasons[:2])
            raise ValidationError(f"{path}: expected exactly one schema match, got {matches}; {detail}")
        return

    expected_type = schema.get("type")
    if expected_type is not None and not value_type_matches(value, expected_type):
        raise ValidationError(f"{path}: expected {expected_type}, got {type(value).__name__}")

    if "const" in schema and value != schema["const"]:
        raise ValidationError(f"{path}: expected constant {schema['const']!r}")
    if "enum" in schema and value not in schema["enum"]:
        raise ValidationError(f"{path}: value {value!r} is not in {schema['enum']!r}")

    if isinstance(value, str):
        if len(value) < schema.get("minLength", 0):
            raise ValidationError(f"{path}: string is shorter than minLength")
        pattern = schema.get("pattern")
        if pattern is not None and re.fullmatch(pattern, value) is None:
            raise ValidationError(f"{path}: value {value!r} does not match {pattern!r}")
        if schema.get("format") == "date-time":
            try:
                datetime.fromisoformat(value.replace("Z", "+00:00"))
            except ValueError as error:
                raise ValidationError(f"{path}: invalid RFC3339 date-time") from error

    if isinstance(value, int) and not isinstance(value, bool):
        if value < schema.get("minimum", value):
            raise ValidationError(f"{path}: integer is below minimum")

    if isinstance(value, list):
        if len(value) < schema.get("minItems", 0):
            raise ValidationError(f"{path}: array is shorter than minItems")
        if schema.get("uniqueItems"):
            encoded = [json.dumps(item, sort_keys=True, ensure_ascii=False) for item in value]
            if len(encoded) != len(set(encoded)):
                raise ValidationError(f"{path}: array items must be unique")
        item_schema = schema.get("items")
        if item_schema is not None:
            for index, item in enumerate(value):
                validate(item, item_schema, f"{path}[{index}]")

    if isinstance(value, dict):
        required = schema.get("required", [])
        missing = [name for name in required if name not in value]
        if missing:
            raise ValidationError(f"{path}: missing required fields {missing!r}")
        properties = schema.get("properties", {})
        if schema.get("additionalProperties") is False:
            extra = sorted(set(value) - set(properties))
            if extra:
                raise ValidationError(f"{path}: unknown fields {extra!r}")
        for name, child_schema in properties.items():
            if name in value:
                validate(value[name], child_schema, f"{path}.{name}")


def load_json(path: Path) -> Any:
    """Load one UTF-8 JSON document."""
    with path.open(encoding="utf-8") as file:
        return json.load(file)


def load_template_frontmatter(path: Path) -> dict[str, Any]:
    """Extract the JSON-compatible YAML frontmatter from one Markdown template."""
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    if len(lines) < 3 or lines[0] != "---":
        raise ValidationError(f"{path}: frontmatter must start with ---")
    try:
        end = lines.index("---", 1)
    except ValueError as error:
        raise ValidationError(f"{path}: frontmatter closing delimiter is missing") from error
    try:
        value = json.loads("\n".join(lines[1:end]))
    except json.JSONDecodeError as error:
        raise ValidationError(f"{path}: frontmatter must be JSON-compatible YAML: {error}") from error
    if not isinstance(value, dict):
        raise ValidationError(f"{path}: frontmatter root must be an object")
    return value


def main() -> int:
    """Validate templates plus positive and negative schema fixtures."""
    schema = load_json(SCHEMA_PATH)
    failures: list[str] = []

    for document in DOCUMENTS:
        template = TEMPLATE_DIR / f"{document}.md"
        valid_fixture = FIXTURE_DIR / "valid" / f"{document}.json"
        invalid_fixture = FIXTURE_DIR / "invalid" / f"{document}.json"
        try:
            validate(load_template_frontmatter(template), schema)
            validate(load_json(valid_fixture), schema)
        except (OSError, json.JSONDecodeError, ValidationError) as error:
            failures.append(f"positive {document}: {error}")
        try:
            validate(load_json(invalid_fixture), schema)
        except ValidationError:
            pass
        except (OSError, json.JSONDecodeError) as error:
            failures.append(f"negative {document}: fixture unreadable: {error}")
        else:
            failures.append(f"negative {document}: invalid fixture unexpectedly passed")

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        return 1
    print("PASS: 4 templates, 4 valid fixtures, and 4 invalid fixtures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
