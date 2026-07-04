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


def check_fixture_policy(root: Path, **kwargs):
    kwargs.setdefault("comparison_surfaces", [])
    kwargs.setdefault("internal_harness_surfaces", [])
    kwargs.setdefault("renderer_lab_surfaces", [])
    return POLICY.check_surface_policy(root, **kwargs)


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

            violations = check_fixture_policy(
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

    def test_default_tutorial_raw_pointer_region_mechanisms_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/tutorial.rs",
                """
                use fret::app::prelude::*;

                fn main() {
                    let _ = "UiPointerActionHost";
                    let _ = "PointerRegionProps";
                    let _ = "DefaultAction::FocusOnPointerDown";
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[
                    POLICY.SurfacePath(
                        "apps/tutorial.rs",
                        "default_app_clean",
                        "fixture default tutorial",
                    )
                ],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            messages = "\n".join(violation.message for violation in violations)
            self.assertIn("UiPointerActionHost", messages)
            self.assertIn("PointerRegionProps", messages)
            self.assertIn("DefaultAction::FocusOnPointerDown", messages)

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

            violations = check_fixture_policy(
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

            default_violations = check_fixture_policy(
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
            advanced_violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/manual.rs",
                        "advanced_manual",
                        "fixture manual assembly surface",
                        owner="fixture-owner",
                        allowed_raw_seams=("FnDriver", "fret_launch"),
                        retirement="fixture retires once public launch wrappers cover this path",
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertGreaterEqual(len(default_violations), 1)
            self.assertEqual([], advanced_violations)

    def test_advanced_manual_surface_requires_quarantine_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(root / "apps/manual.rs", "use fret_launch::FnDriver;\n")

            violations = check_fixture_policy(
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

            self.assertEqual(
                {
                    "advanced-surface-classification-owner",
                    "advanced-surface-classification-retirement",
                    "advanced-surface-classification-raw-seams",
                    "advanced-surface-unlisted-raw-seam",
                },
                {violation.rule for violation in violations},
            )

    def test_advanced_manual_surface_rejects_unlisted_raw_seam(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/manual.rs",
                """
                use fret_ui::UiTree;

                fn main() {
                    let _tree: Option<UiTree<()>> = None;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/manual.rs",
                        "advanced_manual",
                        "fixture manual assembly surface",
                        owner="fixture-owner",
                        allowed_raw_seams=("fret_ui",),
                        retirement="fixture retires once public UI wrappers cover this path",
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertGreaterEqual(len(violations), 1)
            self.assertTrue(
                all(v.rule == "advanced-surface-unlisted-raw-seam" for v in violations)
            )
            self.assertTrue(any("UiTree" in v.message for v in violations))

    def test_advanced_manual_surface_rejects_unused_allowed_raw_seam(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/manual.rs",
                """
                use fret_ui::element::AnyElement;
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/manual.rs",
                        "advanced_manual",
                        "fixture manual assembly surface",
                        owner="fixture-owner",
                        allowed_raw_seams=("fret_ui", "AnyElement", "UiTree"),
                        retirement="fixture retires once public UI wrappers cover this path",
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertEqual(len(violations), 1)
            self.assertEqual(violations[0].rule, "advanced-surface-unused-allowed-raw-seam")
            self.assertIn("UiTree", violations[0].message)

    def test_default_starter_raw_advanced_import_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "crates/fretboard/src/scaffold/templates.rs",
                """
                pub const MAIN_RS: &str = r#"
                use fret::{advanced::prelude::*, FretApp};
                use fret_ui::element::AnyElement;
                "#;
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[
                    POLICY.SurfacePath(
                        "crates/fretboard/src/scaffold/templates.rs",
                        "default_app_clean",
                        "fixture generated starter",
                    )
                ],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertGreaterEqual(len(violations), 1)
            self.assertTrue(all(v.rule == "default-app-clean" for v in violations))
            self.assertTrue(any("advanced" in v.message for v in violations))

    def test_default_cookbook_raw_local_state_constructor_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-cookbook/examples/data_table_basics.rs",
                """
                use fret::app::prelude::*;

                fn init(app: &mut App) {
                    let _state = LocalState::new_in(app.models_mut(), TableState::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-cookbook/examples/data_table_basics.rs",
                        "default_app_clean",
                        "fixture default cookbook",
                    )
                ],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertGreaterEqual(len(violations), 1)
            self.assertTrue(all(v.rule == "default-app-clean" for v in violations))
            self.assertTrue(any("app.local_state" in v.message for v in violations))

    def test_default_cookbook_raw_runtime_model_import_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-cookbook/examples/data_table_basics.rs",
                """
                use fret::app::prelude::*;
                use fret_runtime::Model;

                struct View {
                    output: Model<shadcn::DataTableViewOutput>,
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-cookbook/examples/data_table_basics.rs",
                        "default_app_clean",
                        "fixture default cookbook",
                    )
                ],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertGreaterEqual(len(violations), 1)
            self.assertTrue(all(v.rule == "default-app-clean" for v in violations))
            self.assertTrue(any("fret_runtime" in v.message for v in violations))

    def test_default_cookbook_raw_action_notify_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-cookbook/examples/toast_basics.rs",
                """
                use fret::advanced::raw::AppUiRawActionNotifyExt as _;
                use fret::app::prelude::*;

                fn render(cx: &mut AppUi<'_, '_>) {
                    cx.on_action_notify::<act::Toast>(|_host, _acx| true);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-cookbook/examples/toast_basics.rs",
                        "default_app_clean",
                        "fixture default cookbook",
                    )
                ],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertGreaterEqual(len(violations), 1)
            self.assertTrue(all(v.rule == "default-app-clean" for v in violations))
            self.assertTrue(any("cx.actions()" in v.message for v in violations))

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

            violations = check_fixture_policy(
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

    def test_public_example_raw_seam_requires_surface_classification(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-cookbook/examples/raw_demo.rs",
                """
                use fret::{FretApp, advanced::prelude::*};

                struct RawDemo {
                    value: Model<i32>,
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
                public_example_scan_roots=["apps/fret-cookbook/examples"],
            )

            self.assertGreaterEqual(len(violations), 1)
            self.assertTrue(
                all(v.rule == "public-example-unclassified-raw-seam" for v in violations)
            )
            self.assertTrue(any("advanced-facade" in v.message for v in violations))

    def test_classified_public_example_raw_seam_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-cookbook/examples/raw_demo.rs",
                """
                use fret::{FretApp, advanced::prelude::*};
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-cookbook/examples/raw_demo.rs",
                        "advanced_manual",
                        "fixture classified advanced cookbook example",
                        owner="fixture-cookbook",
                        allowed_raw_seams=("fret::advanced",),
                        retirement="fixture retires when the raw demo has an app-facing wrapper",
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
                public_example_scan_roots=["apps/fret-cookbook/examples"],
            )

            self.assertEqual([], violations)

    def test_comparison_surface_allows_classified_raw_seam_without_retirement(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/compare_demo.rs",
                """
                use fret::{FretApp, advanced::prelude::*};
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                comparison_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/compare_demo.rs",
                        "comparison_surface",
                        "fixture comparison surface",
                        owner="fixture-comparison",
                        allowed_raw_seams=("fret::advanced",),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
                public_example_scan_roots=["apps/fret-examples/src/compare_demo.rs"],
            )

            self.assertEqual([], violations)

    def test_comparison_surface_rejects_unlisted_raw_seam(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/compare_demo.rs",
                """
                use fret_ui::UiTree;
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                comparison_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/compare_demo.rs",
                        "comparison_surface",
                        "fixture comparison surface",
                        owner="fixture-comparison",
                        allowed_raw_seams=("fret_ui",),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertGreaterEqual(len(violations), 1)
            self.assertTrue(
                all(v.rule == "comparison_surface-unlisted-raw-seam" for v in violations)
            )
            self.assertTrue(any("UiTree" in v.message for v in violations))

    def test_internal_harness_allows_classified_raw_seam_without_retirement(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/internal_harness.rs",
                """
                use fret_launch::FnDriver;
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/internal_harness.rs",
                        "internal_harness",
                        "fixture internal harness",
                        owner="fixture-internal-harness",
                        allowed_raw_seams=("fret_launch", "FnDriver"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
                public_example_scan_roots=["apps/fret-examples/src/internal_harness.rs"],
            )

            self.assertEqual([], violations)

    def test_internal_harness_rejects_unlisted_raw_seam(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/internal_harness.rs",
                """
                use fret_ui::UiTree;
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/internal_harness.rs",
                        "internal_harness",
                        "fixture internal harness",
                        owner="fixture-internal-harness",
                        allowed_raw_seams=("fret_ui",),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertGreaterEqual(len(violations), 1)
            self.assertTrue(
                all(v.rule == "internal_harness-unlisted-raw-seam" for v in violations)
            )
            self.assertTrue(any("UiTree" in v.message for v in violations))

    def test_renderer_lab_allows_classified_raw_seam_without_retirement(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-cookbook/examples/renderer_lab.rs",
                """
                use fret_launch::FnDriver;
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                renderer_lab_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-cookbook/examples/renderer_lab.rs",
                        "renderer_lab",
                        "fixture renderer lab",
                        owner="fixture-renderer-lab",
                        allowed_raw_seams=("fret_launch", "FnDriver"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
                public_example_scan_roots=["apps/fret-cookbook/examples/renderer_lab.rs"],
            )

            self.assertEqual([], violations)

    def test_renderer_lab_rejects_unlisted_raw_seam(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-cookbook/examples/renderer_lab.rs",
                """
                use fret_ui::UiTree;
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                renderer_lab_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-cookbook/examples/renderer_lab.rs",
                        "renderer_lab",
                        "fixture renderer lab",
                        owner="fixture-renderer-lab",
                        allowed_raw_seams=("fret_ui",),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertGreaterEqual(len(violations), 1)
            self.assertTrue(all(v.rule == "renderer_lab-unlisted-raw-seam" for v in violations))
            self.assertTrue(any("UiTree" in v.message for v in violations))

    def test_public_example_scan_root_can_target_exact_file(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/public_proof.rs",
                """
                use fret::{FretApp, advanced::prelude::*};
                """,
            )
            write(
                root / "apps/fret-examples/src/internal_harness.rs",
                """
                use fret_launch::FnDriver;
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
                public_example_scan_roots=["apps/fret-examples/src/public_proof.rs"],
            )

            self.assertEqual(1, len(violations))
            self.assertEqual("public-example-unclassified-raw-seam", violations[0].rule)
            self.assertEqual(
                root / "apps/fret-examples/src/public_proof.rs",
                violations[0].path,
            )

    def test_fret_examples_public_scan_roots_stay_precise(self) -> None:
        self.assertIn(
            "apps/fret-examples/src/lib.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/simple_todo_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/todo_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/plot_stress_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/plot_stress_demo.rs"
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
            )
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/simple_todo_demo/driver.rs"
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
            )
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/lib.rs"
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
            )
        )
        self.assertFalse(
            any(
                spec.path == "apps/fret-examples/src/plot_stress_demo.rs"
                for spec in POLICY.ADVANCED_MANUAL_SURFACES
            )
        )
        self.assertFalse(
            any(
                spec.path == "apps/fret-examples/src/lib.rs"
                for spec in POLICY.ADVANCED_MANUAL_SURFACES
            )
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/simple_todo_demo.rs"
                for spec in POLICY.DEFAULT_AUTHORING_SURFACES
            )
        )
        self.assertFalse(
            any(
                spec.path == "apps/fret-examples/src/simple_todo_demo.rs"
                for spec in POLICY.ADVANCED_MANUAL_SURFACES
            )
        )
        self.assertFalse(
            any(
                spec.path == "apps/fret-examples/src/simple_todo_demo/driver.rs"
                for spec in POLICY.DEFAULT_AUTHORING_SURFACES
            )
        )
        self.assertNotIn("apps/fret-examples/src", POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)

    def test_migrated_cookbook_examples_are_default_clean(self) -> None:
        migrated = {
            "apps/fret-cookbook/examples/async_inbox_basics.rs",
            "apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs",
            "apps/fret-cookbook/examples/chart_interactions_basics.rs",
            "apps/fret-cookbook/examples/commands_keymap_basics.rs",
            "apps/fret-cookbook/examples/drag_basics.rs",
            "apps/fret-cookbook/examples/form_basics.rs",
            "apps/fret-cookbook/examples/gizmo_basics.rs",
            "apps/fret-cookbook/examples/imui_action_basics.rs",
            "apps/fret-cookbook/examples/imui_editor_controls_basics.rs",
            "apps/fret-cookbook/examples/imui_plot_basics.rs",
            "apps/fret-cookbook/examples/router_basics.rs",
            "apps/fret-cookbook/examples/text_input_basics.rs",
            "apps/fret-cookbook/examples/undo_basics.rs",
            "apps/fret-cookbook/examples/virtual_list_basics.rs",
        }
        default_paths = {spec.path for spec in POLICY.DEFAULT_AUTHORING_SURFACES}
        advanced_paths = {spec.path for spec in POLICY.ADVANCED_MANUAL_SURFACES}

        self.assertTrue(migrated.issubset(default_paths))
        self.assertTrue(migrated.isdisjoint(advanced_paths))

    def test_renderer_labs_do_not_count_as_advanced_manual_quarantine(self) -> None:
        renderer_lab_paths = {spec.path for spec in POLICY.RENDERER_LAB_SURFACES}
        advanced_paths = {spec.path for spec in POLICY.ADVANCED_MANUAL_SURFACES}

        expected = {
            "apps/fret-cookbook/examples/compositing_alpha_basics.rs",
            "apps/fret-cookbook/examples/customv1_basics.rs",
            "apps/fret-cookbook/examples/image_asset_cache_basics.rs",
        }

        self.assertTrue(expected.issubset(renderer_lab_paths))
        self.assertTrue(expected.isdisjoint(advanced_paths))

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

            violations = check_fixture_policy(
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

            violations = check_fixture_policy(
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

    def test_roving_typeahead_root_exports_are_rejected_but_mechanism_members_are_allowed(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "crates/fret-ui/src/lib.rs",
                """
                pub use element::RovingFocusProps;
                pub use action::RovingTypeaheadCx;
                """,
            )
            write(
                root / "crates/fret-ui/src/action.rs",
                """
                pub struct RovingTypeaheadCx;
                pub type OnRovingTypeahead = ();
                """,
            )

            violations = check_fixture_policy(
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

            self.assertEqual(2, len(violations))
            self.assertTrue(
                all(v.rule == "mechanism-root-policy-vocabulary" for v in violations)
            )
            self.assertTrue(any("RovingFocusProps" in v.source for v in violations))
            self.assertTrue(any("RovingTypeaheadCx" in v.source for v in violations))

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

            violations = check_fixture_policy(
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

            violations = check_fixture_policy(
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

    def test_dismissible_public_member_is_rejected_in_mechanism_crate(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "crates/fret-ui/src/elements/access.rs",
                """
                pub fn dismissible_has_pointer_move_handler() {}
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertEqual(1, len(violations))
            self.assertEqual(
                "mechanism-public-member-policy-vocabulary:dismissible-public-member",
                violations[0].rule,
            )

    def test_auto_focus_public_action_hook_is_rejected_in_mechanism_crate(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "crates/fret-ui/src/action.rs",
                """
                pub struct AutoFocusRequestCx;

                pub type OnOpenAutoFocus = ();
                pub type OnCloseAutoFocus = ();
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertEqual(3, len(violations))
            self.assertTrue(
                all(
                    v.rule
                    == "mechanism-public-member-policy-vocabulary:auto-focus-action-hook"
                    for v in violations
                )
            )

    def test_classified_surface_paths_must_exist(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)

            violations = check_fixture_policy(
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

            self.assertIn("classified-surface-exists", {v.rule for v in violations})


if __name__ == "__main__":
    unittest.main()
