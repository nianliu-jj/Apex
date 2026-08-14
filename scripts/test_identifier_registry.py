from __future__ import annotations

import copy
import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = ROOT / 'scripts' / 'validate_identifier_registry.py'
SPEC = importlib.util.spec_from_file_location('validate_identifier_registry', MODULE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f'cannot load {MODULE_PATH}')
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class IdentifierRegistryTests(unittest.TestCase):
    """Exercise the VAL-02 duplicate, gap, and unregistered-ID failures."""

    def setUp(self) -> None:
        self.registry = MODULE.load_registry()

    def test_repository_registry_passes(self) -> None:
        MODULE.validate_registry(self.registry)

    def test_duplicate_identifier_fails(self) -> None:
        registry = copy.deepcopy(self.registry)
        registry['entries'].append(copy.deepcopy(registry['entries'][0]))
        with self.assertRaisesRegex(MODULE.RegistryError, 'duplicate identifiers'):
            MODULE.validate_registry(registry)

    def test_sequence_gap_fails(self) -> None:
        registry = copy.deepcopy(self.registry)
        registry['entries'] = [
            entry for entry in registry['entries'] if entry['id'] != 'RQ-124'
        ]
        with self.assertRaisesRegex(MODULE.RegistryError, 'sequence has gaps'):
            MODULE.validate_registry(registry)

    def test_unregistered_identifier_fails(self) -> None:
        registry = copy.deepcopy(self.registry)
        registry['entries'].append(
            {
                'id': 'RQ-125',
                'lifecycle': 'active',
                'declared_in': 'test fixture',
            }
        )
        with self.assertRaisesRegex(MODULE.RegistryError, 'unregistered identifiers'):
            MODULE.validate_registry(registry)


if __name__ == '__main__':
    unittest.main()
