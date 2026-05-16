#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path
from types import ModuleType
from typing import Any


def load_registry_module() -> ModuleType:
    path = Path(__file__).with_name("check_diag_scripts_registry.py")
    spec = importlib.util.spec_from_file_location("check_diag_scripts_registry", path)
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


REGISTRY = load_registry_module()


class DiagScriptRegistryLintTests(unittest.TestCase):
    def write_script(self, root: Path, rel_path: str, steps: list[dict[str, Any]]) -> None:
        self.write_script_with_meta(root, rel_path, steps, {})

    def write_script_with_meta(
        self,
        root: Path,
        rel_path: str,
        steps: list[dict[str, Any]],
        meta: dict[str, Any],
    ) -> None:
        path = root / rel_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            json.dumps({"schema_version": 2, "meta": meta, "steps": steps}, indent=2) + "\n",
            encoding="utf-8",
        )

    def registry_for(self, rel_path: str, suites: list[str]) -> dict[str, Any]:
        return {
            "schema_version": 1,
            "kind": "diag_script_registry",
            "scope": "suites+prelude",
            "scripts": [
                {
                    "id": Path(rel_path).stem,
                    "path": rel_path,
                    "tags": [],
                    "target_hints": [],
                    "required_capabilities": [],
                    "required_launch_features": [],
                    "suite_memberships": suites,
                }
            ],
        }

    def test_page_local_motion_preset_selector_requires_page_entry(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            rel = "tools/diag-scripts/ui-gallery/motion-presets/bad.json"
            self.write_script(
                root,
                rel,
                [
                    {"type": "reset_diagnostics"},
                    {
                        "type": "wait_until",
                        "predicate": {
                            "kind": "exists",
                            "target": {
                                "kind": "test_id",
                                "id": "ui-gallery-motion-presets-environment-probe",
                            },
                        },
                    },
                ],
            )

            violations = REGISTRY.lint_strict_page_entry(
                root, self.registry_for(rel, ["ui-gallery-motion-pilot"])
            )

            self.assertEqual(1, len(violations))
            self.assertIn("ui-gallery-page-motion-presets", violations[0])
            self.assertIn("ui-gallery-motion-presets-environment-probe", violations[0])

    def test_page_entry_allows_later_motion_preset_selector(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            rel = "tools/diag-scripts/ui-gallery/motion-presets/good.json"
            self.write_script(
                root,
                rel,
                [
                    {"type": "reset_diagnostics"},
                    {
                        "type": "wait_until",
                        "predicate": {
                            "kind": "exists",
                            "target": {
                                "kind": "test_id",
                                "id": "ui-gallery-page-motion-presets",
                            },
                        },
                    },
                    {
                        "type": "wait_until",
                        "predicate": {
                            "kind": "exists",
                            "target": {
                                "kind": "test_id",
                                "id": "ui-gallery-motion-presets-environment-probe",
                            },
                        },
                    },
                ],
            )

            violations = REGISTRY.lint_strict_page_entry(
                root, self.registry_for(rel, ["ui-gallery-motion-pilot"])
            )

            self.assertEqual([], violations)

    def test_motion_preset_shell_trigger_does_not_require_page_entry(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            rel = "tools/diag-scripts/ui-gallery/motion-presets/shell-trigger.json"
            self.write_script(
                root,
                rel,
                [
                    {"type": "reset_diagnostics"},
                    {
                        "type": "click_stable",
                        "target": {
                            "kind": "test_id",
                            "id": "ui-gallery-motion-preset-trigger",
                        },
                    },
                ],
            )

            violations = REGISTRY.lint_strict_page_entry(
                root, self.registry_for(rel, ["ui-gallery-motion-pilot"])
            )

            self.assertEqual([], violations)

    def test_select_page_local_selector_requires_page_entry(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            rel = "tools/diag-scripts/ui-gallery/select/bad.json"
            self.write_script(
                root,
                rel,
                [
                    {"type": "reset_diagnostics"},
                    {
                        "type": "click_stable",
                        "target": {
                            "kind": "test_id",
                            "id": "ui-gallery-select-demo-trigger",
                        },
                    },
                ],
            )

            violations = REGISTRY.lint_strict_page_entry(
                root, self.registry_for(rel, ["ui-gallery-select"])
            )

            self.assertEqual(1, len(violations))
            self.assertIn("ui-gallery-page-select", violations[0])
            self.assertIn("ui-gallery-select-demo-trigger", violations[0])

    def test_select_page_entry_allows_later_page_local_selector(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            rel = "tools/diag-scripts/ui-gallery/select/good.json"
            self.write_script(
                root,
                rel,
                [
                    {"type": "reset_diagnostics"},
                    {
                        "type": "wait_until",
                        "predicate": {
                            "kind": "exists",
                            "target": {
                                "kind": "test_id",
                                "id": "ui-gallery-page-select",
                            },
                        },
                    },
                    {
                        "type": "click_stable",
                        "target": {
                            "kind": "test_id",
                            "id": "ui-gallery-select-demo-trigger",
                        },
                    },
                ],
            )

            violations = REGISTRY.lint_strict_page_entry(
                root, self.registry_for(rel, ["ui-gallery-select"])
            )

            self.assertEqual([], violations)

    def test_combobox_start_page_default_allows_page_local_selector(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            rel = "tools/diag-scripts/ui-gallery/combobox/start-page.json"
            self.write_script_with_meta(
                root,
                rel,
                [
                    {"type": "reset_diagnostics"},
                    {
                        "type": "click_stable",
                        "target": {
                            "kind": "test_id",
                            "id": "ui-gallery-combobox-input-group-trigger",
                        },
                    },
                ],
                {
                    "env_defaults": {
                        "FRET_UI_GALLERY_START_PAGE": "combobox",
                        "FRET_UI_GALLERY_START_SECTION": "Input Group",
                    }
                },
            )

            violations = REGISTRY.lint_strict_page_entry(
                root, self.registry_for(rel, ["ui-gallery-combobox"])
            )

            self.assertEqual([], violations)

    def test_combobox_without_start_page_or_page_root_fails(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            rel = "tools/diag-scripts/ui-gallery/combobox/bad.json"
            self.write_script(
                root,
                rel,
                [
                    {"type": "reset_diagnostics"},
                    {
                        "type": "click_stable",
                        "target": {
                            "kind": "test_id",
                            "id": "ui-gallery-combobox-input-group-trigger",
                        },
                    },
                ],
            )

            violations = REGISTRY.lint_strict_page_entry(
                root, self.registry_for(rel, ["ui-gallery-combobox"])
            )

            self.assertEqual(1, len(violations))
            self.assertIn("ui-gallery-page-combobox", violations[0])

    def test_data_table_page_entry_allows_variant_root(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            rel = "tools/diag-scripts/ui-gallery/data-table/good.json"
            self.write_script(
                root,
                rel,
                [
                    {"type": "reset_diagnostics"},
                    {
                        "type": "wait_until",
                        "predicate": {
                            "kind": "exists",
                            "target": {
                                "kind": "test_id",
                                "id": "ui-gallery-data-table-default-root",
                            },
                        },
                    },
                    {
                        "type": "click_stable",
                        "target": {
                            "kind": "test_id",
                            "id": "ui-gallery-data-table-default-next",
                        },
                    },
                ],
            )

            violations = REGISTRY.lint_strict_page_entry(
                root, self.registry_for(rel, ["ui-gallery-data-table"])
            )

            self.assertEqual([], violations)

    def test_data_table_page_local_selector_requires_page_entry(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            rel = "tools/diag-scripts/ui-gallery/data-table/bad.json"
            self.write_script(
                root,
                rel,
                [
                    {"type": "reset_diagnostics"},
                    {
                        "type": "click_stable",
                        "target": {
                            "kind": "test_id",
                            "id": "ui-gallery-data-table-default-next",
                        },
                    },
                ],
            )

            violations = REGISTRY.lint_strict_page_entry(
                root, self.registry_for(rel, ["ui-gallery-data-table"])
            )

            self.assertEqual(1, len(violations))
            self.assertIn("ui-gallery-data-table-component", violations[0])
            self.assertIn("ui-gallery-data-table-default-next", violations[0])

    def test_data_table_torture_start_page_default_allows_page_local_selector(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            rel = "tools/diag-scripts/ui-gallery/data-table/torture-start-page.json"
            self.write_script_with_meta(
                root,
                rel,
                [
                    {"type": "reset_diagnostics"},
                    {
                        "type": "click_stable",
                        "target": {
                            "kind": "test_id",
                            "id": "ui-gallery-data-table-torture-reset-state",
                        },
                    },
                ],
                {"env_defaults": {"FRET_UI_GALLERY_START_PAGE": "data_table_torture"}},
            )

            violations = REGISTRY.lint_strict_page_entry(
                root, self.registry_for(rel, ["ui-gallery-data-table-retained"])
            )

            self.assertEqual([], violations)

    def test_sidebar_long_page_click_requires_content_scroll_visibility(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            rel = "tools/diag-scripts/ui-gallery/sidebar/bad.json"
            self.write_script(
                root,
                rel,
                [
                    {"type": "reset_diagnostics"},
                    {
                        "type": "wait_until",
                        "predicate": {
                            "kind": "exists",
                            "target": {
                                "kind": "test_id",
                                "id": "ui-gallery-page-sidebar",
                            },
                        },
                    },
                    {
                        "type": "click_stable",
                        "target": {
                            "kind": "test_id",
                            "id": "ui-gallery-sidebar-demo-toggle",
                        },
                    },
                ],
            )

            violations = REGISTRY.lint_strict_click_visibility(
                root, self.registry_for(rel, ["ui-gallery-motion-pilot"])
            )

            self.assertEqual(1, len(violations))
            self.assertIn("ui-gallery-sidebar-demo-toggle", violations[0])
            self.assertIn("scroll_into_view", violations[0])


if __name__ == "__main__":
    unittest.main()
