#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path
from types import ModuleType


def load_surface_policy_module() -> ModuleType:
    path = Path(__file__).with_name("check_surface_policy.py")
    spec = importlib.util.spec_from_file_location("check_surface_policy", path)
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


POLICY = load_surface_policy_module()


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(textwrap.dedent(text).lstrip(), encoding="utf-8")


class SurfacePolicyTests(unittest.TestCase):
    def test_default_tutorial_raw_runtime_import_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "docs/first-hour.md",
                """
                # Tutorial

                ```rust
                use fret_ui::ElementContext;
                ```
                """,
            )

            violations = POLICY.check_surface_policy(
                root,
                default_surfaces=[
                    POLICY.SurfacePath(
                        "docs/first-hour.md",
                        "default_app_clean",
                        "fixture default tutorial",
                    )
                ],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertEqual(1, len(violations))
            self.assertEqual("default-app-clean", violations[0].rule)
            self.assertIn("fret_ui", violations[0].message)

    def test_prose_mentions_do_not_trip_default_code_scan(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "docs/first-hour.md",
                """
                Prefer AppComponentCx when you intentionally want an app-hosted
                ElementContext<App> lane later.

                ```bash
                rg ElementContext
                ```
                """,
            )

            violations = POLICY.check_surface_policy(
                root,
                default_surfaces=[
                    POLICY.SurfacePath(
                        "docs/first-hour.md",
                        "default_app_clean",
                        "fixture default tutorial",
                    )
                ],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertEqual([], violations)

    def test_manual_surface_is_allowed_when_classified_outside_default_scan(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/manual.rs",
                """
                use fret_launch::FnDriver;

                fn main() {
                    let _driver = FnDriver::new;
                }
                """,
            )

            default_violations = POLICY.check_surface_policy(
                root,
                default_surfaces=[
                    POLICY.SurfacePath(
                        "apps/manual.rs",
                        "default_app_clean",
                        "fixture unclassified default surface",
                    )
                ],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )
            advanced_violations = POLICY.check_surface_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/manual.rs",
                        "advanced_manual",
                        "fixture manual assembly surface",
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertGreaterEqual(len(default_violations), 1)
            self.assertEqual([], advanced_violations)

    def test_policy_recipe_crate_may_consume_runtime_mechanisms(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "ecosystem/fret-ui-kit/src/lib.rs",
                """
                use fret_ui::{ElementContext, UiHost};

                pub fn helper<H: UiHost>(_cx: &mut ElementContext<'_, H>) {}
                """,
            )

            violations = POLICY.check_surface_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[
                    POLICY.SurfacePath(
                        "ecosystem/fret-ui-kit/src",
                        "policy_recipe",
                        "fixture policy crate",
                    )
                ],
                mechanism_root_surfaces=[],
            )

            self.assertEqual([], violations)

    def test_policy_coded_fret_ui_root_export_is_rejected_without_classification(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "crates/fret-ui/src/lib.rs",
                """
                pub mod element;
                pub use dialog::Dialog;
                """,
            )

            violations = POLICY.check_surface_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[
                    POLICY.SurfacePath(
                        "crates/fret-ui/src/lib.rs",
                        "mechanism_crate_root",
                        "fixture mechanism root",
                    )
                ],
            )

            self.assertEqual(1, len(violations))
            self.assertEqual("mechanism-root-policy-vocabulary", violations[0].rule)
            self.assertIn("Dialog", violations[0].message)

    def test_resizable_chrome_style_root_export_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "crates/fret-ui/src/lib.rs",
                """
                pub mod element;
                pub use resizable_panel_group::ResizablePanelGroupStyle;
                """,
            )

            violations = POLICY.check_surface_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[
                    POLICY.SurfacePath(
                        "crates/fret-ui/src/lib.rs",
                        "mechanism_crate_root",
                        "fixture mechanism root",
                    )
                ],
            )

            self.assertEqual(1, len(violations))
            self.assertEqual("mechanism-root-policy-vocabulary", violations[0].rule)
            self.assertIn("ResizablePanelGroupStyle", violations[0].source)

    def test_scroll_dismiss_public_mechanism_member_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "crates/fret-ui/src/tree/layers/impls.rs",
                """
                impl UiTree {
                    pub fn set_layer_scroll_dismiss_elements(&mut self) {}
                }
                """,
            )

            violations = POLICY.check_surface_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertEqual(1, len(violations))
            self.assertEqual(
                "mechanism-public-member-policy-vocabulary:scroll-dismiss",
                violations[0].rule,
            )
            self.assertIn("scroll_dismiss", violations[0].source)

    def test_dismiss_public_action_hook_is_rejected_in_mechanism_crate(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "crates/fret-ui/src/action.rs",
                """
                pub enum DismissReason {
                    Escape,
                }

                pub type OnDismissRequest = ();
                """,
            )

            violations = POLICY.check_surface_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertEqual(2, len(violations))
            self.assertTrue(
                all(
                    v.rule
                    == "mechanism-public-member-policy-vocabulary:dismiss-action-hook"
                    for v in violations
                )
            )

    def test_classified_surface_paths_must_exist(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)

            violations = POLICY.check_surface_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/missing.rs",
                        "advanced_manual",
                        "fixture missing manual surface",
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertEqual(1, len(violations))
            self.assertEqual("classified-surface-exists", violations[0].rule)


if __name__ == "__main__":
    unittest.main()
