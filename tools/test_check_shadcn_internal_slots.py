#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path
from types import ModuleType


def load_lint_module() -> ModuleType:
    path = Path(__file__).with_name("check_shadcn_internal_slots.py")
    spec = importlib.util.spec_from_file_location("check_shadcn_internal_slots", path)
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


LINT = load_lint_module()


class ShadcnInternalSlotLintTests(unittest.TestCase):
    def write_source(self, root: Path, text: str) -> Path:
        path = root / "ecosystem/fret-ui-shadcn/src/example.rs"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def lint_text(self, text: str) -> list[object]:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            source = self.write_source(root, text)
            return LINT.lint_source(source, source.relative_to(root))

    def test_component_slot_usage_passes(self) -> None:
        violations = self.lint_text(
            """
const ITEM_MEDIA_SLOT: &str = "fret-ui-shadcn.item-media";

fn build(el: AnyElement) -> AnyElement {
    el.component_slot(ITEM_MEDIA_SLOT)
}
"""
        )

        self.assertEqual([], violations)

    def test_internal_marker_const_must_be_named_slot(self) -> None:
        violations = self.lint_text(
            """
const ITEM_MEDIA_MARKER_PREFIX: &str = "fret-ui-shadcn.item-media";
"""
        )

        self.assertEqual(1, len(violations))
        self.assertIn("_SLOT", violations[0].message)

    def test_internal_slot_constant_cannot_be_used_as_test_id(self) -> None:
        violations = self.lint_text(
            """
const ITEM_MEDIA_SLOT: &str = "fret-ui-shadcn.item-media";

fn build(el: AnyElement) -> AnyElement {
    el.test_id(ITEM_MEDIA_SLOT)
}
"""
        )

        self.assertEqual(1, len(violations))
        self.assertIn("test_id/key_context", violations[0].message)

    def test_internal_slot_literal_cannot_be_used_as_key_context(self) -> None:
        violations = self.lint_text(
            """
fn build(el: AnyElement) -> AnyElement {
    el.key_context("fret-ui-shadcn.item-media")
}
"""
        )

        self.assertEqual(1, len(violations))
        self.assertIn("literal", violations[0].message)


if __name__ == "__main__":
    unittest.main()
