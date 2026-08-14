from __future__ import annotations

import json
import re
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = ROOT / 'docs' / 'governance' / 'identifier-registry.json'
PLAN_PATH = ROOT / 'docs' / 'Apex 功能开发原子化执行计划.md'
DESIGN_PATH = ROOT / 'docs' / 'Apex 设计文档.md'
IDENTIFIER_PATTERN = re.compile(r'\b(?:RQ|AC)-[0-9]{3}\b|\bEP-[0-9]{4}\b|\bVAL-[0-9]{2,3}[A-Z]?\b')


class RegistryError(ValueError):
    """Raised when the identifier registry or its source documents are invalid."""


@dataclass(frozen=True)
class Sequence:
    """Describe one contiguous numeric identifier sequence."""

    prefix: str
    start: int
    end: int
    minimum_digits: int

    def expected(self) -> set[str]:
        """Return the identifiers required by this sequence."""
        return {
            f'{self.prefix}{number:0{max(self.minimum_digits, len(str(number)))}d}'
            for number in range(self.start, self.end + 1)
        }


def load_registry() -> dict:
    """Load the canonical identifier registry."""
    try:
        value = json.loads(REGISTRY_PATH.read_text(encoding='utf-8'))
    except (OSError, json.JSONDecodeError) as error:
        raise RegistryError(f'cannot read registry: {error}') from error
    if not isinstance(value, dict):
        raise RegistryError('registry root must be an object')
    return value


def identifiers(value: Iterable[dict]) -> list[str]:
    """Extract identifier values from registry entries."""
    result: list[str] = []
    for entry in value:
        identifier = entry.get('id')
        if not isinstance(identifier, str):
            raise RegistryError('every registry entry must have a string id')
        result.append(identifier)
    return result


def validate_unique(values: Iterable[str], label: str) -> None:
    """Reject duplicate identifiers in one registry scope."""
    duplicates = sorted(identifier for identifier, count in Counter(values).items() if count > 1)
    if duplicates:
        raise RegistryError(f'{label} contains duplicate identifiers: {duplicates}')


def validate_sequences(registry: dict, entry_ids: set[str]) -> None:
    """Reject gaps and unregistered identifiers in declared sequences."""
    expected: set[str] = set()
    for namespace in registry.get('namespaces', []):
        for declaration in namespace.get('sequences', []):
            prefix = declaration.get('prefix')
            start = declaration.get('start')
            end = declaration.get('end')
            minimum_digits = declaration.get('minimum_digits')
            if (
                not isinstance(prefix, str)
                or not isinstance(start, int)
                or not isinstance(end, int)
                or not isinstance(minimum_digits, int)
            ):
                raise RegistryError('sequence declarations require prefix, start, and end')
            if start > end:
                raise RegistryError(f'invalid sequence bounds for {prefix}')
            sequence = Sequence(prefix, start, end, minimum_digits)
            sequence_ids = sequence.expected()
            missing = sorted(sequence_ids - entry_ids)
            if missing:
                raise RegistryError(f'{prefix} sequence has gaps: {missing}')
            expected.update(sequence_ids)
        for extension in namespace.get('extensions', []):
            if not isinstance(extension, str):
                raise RegistryError('sequence extensions must be strings')
            expected.add(extension)
    unregistered = sorted(entry_ids - expected)
    if unregistered:
        raise RegistryError(f'unregistered identifiers: {unregistered}')


def validate_source_document(path: Path, registered: set[str]) -> None:
    """Reject identifiers used by a source document but absent from the registry."""
    text = path.read_text(encoding='utf-8')
    used = set(IDENTIFIER_PATTERN.findall(text))
    missing = sorted(used - registered)
    if missing:
        raise RegistryError(f'{path.relative_to(ROOT)} uses unregistered identifiers: {missing}')


def validate_ep_rows(registry: dict) -> None:
    """Keep the canonical EP registry aligned with the plan table."""
    text = PLAN_PATH.read_text(encoding='utf-8')
    start = text.index('## 8. EP 总表')
    end = text.index('## 9. Active EP 完整执行卡')
    table_ids = re.findall(r'^\| (EP-\d{4}) \|', text[start:end], re.MULTILINE)
    registry_ids = {
        entry['id']
        for entry in registry['entries']
        if entry['id'].startswith('EP-')
    }
    if table_ids != sorted(table_ids, key=lambda identifier: int(identifier[3:])):
        raise RegistryError('EP table is not in numeric order')
    if len(table_ids) != len(set(table_ids)):
        raise RegistryError('EP table contains duplicate identifiers')
    if set(table_ids) != registry_ids:
        raise RegistryError('EP registry does not match the plan EP table')


def validate_registry(registry: dict) -> None:
    """Validate registry policy, entries, sequences, and source documents."""
    if registry.get('schema') != 'apex.identifier-registry.v1':
        raise RegistryError('unsupported registry schema')
    policy = registry.get('policy')
    if policy != {
        'append_only': True,
        'reuse_forbidden': True,
        'canonical_order': ['RQ', 'AC', 'EP', 'VAL'],
    }:
        raise RegistryError('append-only identifier policy is not enabled')
    entries = registry.get('entries')
    if not isinstance(entries, list) or not entries:
        raise RegistryError('registry entries must be a non-empty array')
    entry_ids = identifiers(entries)
    validate_unique(entry_ids, 'registry')
    expected_namespaces = set(policy['canonical_order'])
    for identifier in entry_ids:
        namespace = identifier.split('-', maxsplit=1)[0]
        if namespace not in expected_namespaces:
            raise RegistryError(f'unsupported identifier namespace: {identifier}')
    validate_sequences(registry, set(entry_ids))
    validate_ep_rows(registry)
    registered = set(entry_ids)
    for source in (PLAN_PATH, DESIGN_PATH, ROOT / 'docs' / 'Apex 原子模块系分文档.md'):
        validate_source_document(source, registered)


def main() -> int:
    """Validate the canonical registry and its source documents."""
    try:
        validate_registry(load_registry())
    except (OSError, RegistryError) as error:
        print(f'FAIL: {error}', file=sys.stderr)
        return 1
    print('PASS: identifier registry, sequence policy, and source references')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
