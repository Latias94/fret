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

    def test_custom_effect_owner_test_raw_seams_do_not_count_against_surface_policy(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/custom_effect_v2_web_owner.rs",
                """
                use fret_core::scene::EffectParamsV1;

                fn make_params() -> EffectParamsV1 {
                    EffectParamsV1::ZERO
                }

                #[cfg(test)]
                mod tests {
                    use fret_ui::UiTree;

                    fn render_harness() {
                        let _tree: Option<UiTree<()>> = None;
                    }
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/custom_effect_v2_web_owner.rs",
                        "internal_harness",
                        "fixture custom-effect owner helper",
                        owner="fixture-owner",
                        allowed_raw_seams=("fret_core",),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertEqual([], violations)

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
        query_paths = {
            "apps/fret-examples/src/query_demo.rs",
            "apps/fret-examples/src/query_async_tokio_demo.rs",
        }
        for path in query_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        self.assertIn(
            "apps/fret-examples/src/assets_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        utility_window_paths = {
            "apps/fret-examples/src/launcher_utility_window_demo.rs",
            "apps/fret-examples/src/launcher_utility_window_materials_demo.rs",
        }
        for path in utility_window_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        self.assertIn(
            "apps/fret-examples/src/container_queries_docking_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/async_playground_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/plot_stress_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/echarts_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/echarts_multi_grid_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/chart_multi_axis_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/chart_stress_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        manual_chart_paths = {
            "apps/fret-examples/src/area_demo.rs",
            "apps/fret-examples/src/bars_demo.rs",
            "apps/fret-examples/src/candlestick_demo.rs",
            "apps/fret-examples/src/category_line_demo.rs",
            "apps/fret-examples/src/chart_demo.rs",
            "apps/fret-examples/src/error_bars_demo.rs",
            "apps/fret-examples/src/grouped_bars_demo.rs",
            "apps/fret-examples/src/heatmap_demo.rs",
            "apps/fret-examples/src/histogram2d_demo.rs",
            "apps/fret-examples/src/histogram_demo.rs",
            "apps/fret-examples/src/horizontal_bars_demo.rs",
            "apps/fret-examples/src/inf_lines_demo.rs",
            "apps/fret-examples/src/linked_cursor_demo.rs",
            "apps/fret-examples/src/plot3d_demo.rs",
            "apps/fret-examples/src/shaded_demo.rs",
            "apps/fret-examples/src/stacked_bars_demo.rs",
            "apps/fret-examples/src/stairs_demo.rs",
            "apps/fret-examples/src/stems_demo.rs",
        }
        for path in manual_chart_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        self.assertIn(
            "apps/fret-examples/src/virtual_list_stress_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/table_stress_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/canvas_datagrid_stress_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/datatable_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        self.assertIn(
            "apps/fret-examples/src/embedded_viewport_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        editor_notes_paths = {
            "apps/fret-examples/src/editor_notes_demo.rs",
            "apps/fret-examples/src/editor_notes_device_shell_demo.rs",
        }
        for path in editor_notes_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        external_import_paths = {
            "apps/fret-examples/src/external_texture_imports_demo.rs",
            "apps/fret-examples/src/external_texture_imports_web_demo.rs",
            "apps/fret-examples/src/external_video_imports_avf_demo.rs",
            "apps/fret-examples/src/external_video_imports_mf_demo.rs",
        }
        for path in external_import_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        self.assertIn(
            "apps/fret-examples/src/window_hit_test_probe_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        custom_effect_v2_web_paths = {
            "apps/fret-examples/src/custom_effect_v2_web_demo.rs",
            "apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs",
            "apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs",
            "apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs",
        }
        custom_effect_reference_paths = {
            "apps/fret-examples/src/custom_effect_v1_demo.rs",
            "apps/fret-examples/src/custom_effect_v2_demo.rs",
            "apps/fret-examples/src/custom_effect_v3_demo.rs",
            "apps/fret-examples/src/custom_effect_v3_web_demo.rs",
        }
        streaming_import_paths = {
            "apps/fret-examples/src/streaming_i420_demo.rs",
            "apps/fret-examples/src/streaming_image_demo.rs",
            "apps/fret-examples/src/streaming_nv12_demo.rs",
        }
        smoke_effects_paths = {
            "apps/fret-examples/src/effects_demo.rs",
            "apps/fret-examples/src/first_frame_smoke_demo.rs",
        }
        renderer_media_lab_paths = {
            "apps/fret-examples/src/alpha_mode_demo.rs",
            "apps/fret-examples/src/drop_shadow_demo.rs",
            "apps/fret-examples/src/image_upload_demo.rs",
        }
        text_input_conformance_paths = {
            "apps/fret-examples/src/cjk_conformance_demo.rs",
            "apps/fret-examples/src/emoji_conformance_demo.rs",
            "apps/fret-examples/src/ime_smoke_demo.rs",
        }
        memory_perf_harness_paths = {
            "apps/fret-examples/src/extras_marquee_perf_demo.rs",
            "apps/fret-examples/src/image_heavy_memory_demo.rs",
            "apps/fret-examples/src/text_heavy_memory_demo.rs",
        }
        effect_reference_paths = {
            "apps/fret-examples/src/liquid_glass_demo.rs",
            "apps/fret-examples/src/postprocess_theme_demo.rs",
        }
        for path in custom_effect_v2_web_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        for path in custom_effect_reference_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        for path in streaming_import_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        for path in smoke_effects_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        for path in renderer_media_lab_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        for path in text_input_conformance_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        for path in memory_perf_harness_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        for path in effect_reference_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/plot_stress_demo.rs"
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
            )
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/chart_stress_demo.rs"
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
            )
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/virtual_list_stress_demo.rs"
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
            )
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/table_stress_demo.rs"
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
            )
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/canvas_datagrid_stress_demo.rs"
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
            )
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/echarts_demo.rs"
                for spec in POLICY.COMPARISON_SURFACES
            )
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/embedded_viewport_demo.rs"
                for spec in POLICY.ADVANCED_MANUAL_SURFACES
            )
        )
        for path in utility_window_paths:
            spec = next(
                (
                    spec
                    for spec in POLICY.ADVANCED_MANUAL_SURFACES
                    if spec.path == path
                ),
                None,
            )
            self.assertIsNotNone(
                spec, f"{path} should be classified as an advanced utility-window surface"
            )
            self.assertIn("utility-window", spec.reason)
            self.assertTrue(spec.allowed_raw_seams)
            self.assertTrue(spec.retirement)
        async_playground_spec = next(
            (
                spec
                for spec in POLICY.ADVANCED_MANUAL_SURFACES
                if spec.path == "apps/fret-examples/src/async_playground_demo.rs"
            ),
            None,
        )
        self.assertIsNotNone(
            async_playground_spec,
            "async_playground_demo should be classified as an advanced query playground",
        )
        self.assertIn("pressable", async_playground_spec.reason)
        self.assertIn("AnyElement", async_playground_spec.allowed_raw_seams)
        self.assertTrue(async_playground_spec.retirement)
        for path in editor_notes_paths:
            self.assertTrue(
                any(spec.path == path for spec in POLICY.ADVANCED_MANUAL_SURFACES),
                f"{path} should be classified as an advanced editor notes surface",
            )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/datatable_demo.rs"
                for spec in POLICY.ADVANCED_MANUAL_SURFACES
            )
        )
        for path in external_import_paths:
            self.assertTrue(
                any(spec.path == path for spec in POLICY.ADVANCED_MANUAL_SURFACES),
                f"{path} should be classified as an advanced external import surface",
            )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/external_imports_owner.rs"
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
            )
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/window_hit_test_probe_demo.rs"
                for spec in POLICY.ADVANCED_MANUAL_SURFACES
            )
        )
        echarts_spec = next(
            spec
            for spec in POLICY.COMPARISON_SURFACES
            if spec.path == "apps/fret-examples/src/echarts_demo.rs"
        )
        self.assertNotIn("fret_runtime", echarts_spec.allowed_raw_seams)
        for chart_path in manual_chart_paths:
            self.assertTrue(
                any(spec.path == chart_path for spec in POLICY.ADVANCED_MANUAL_SURFACES),
                f"{chart_path} should be classified as an advanced manual chart surface",
            )
        for chart_path in {
            "apps/fret-examples/src/echarts_multi_grid_demo.rs",
            "apps/fret-examples/src/chart_multi_axis_demo.rs",
        }:
            self.assertTrue(
                any(spec.path == chart_path for spec in POLICY.ADVANCED_MANUAL_SURFACES),
                f"{chart_path} should be classified as an advanced chart surface",
            )
        multi_grid_spec = next(
            spec
            for spec in POLICY.ADVANCED_MANUAL_SURFACES
            if spec.path == "apps/fret-examples/src/echarts_multi_grid_demo.rs"
        )
        self.assertIn("ChartCanvasMultiGridBinding", multi_grid_spec.reason)
        for path in custom_effect_v2_web_paths:
            spec = next(
                (
                    spec
                    for spec in POLICY.ADVANCED_MANUAL_SURFACES
                    if spec.path == path
                ),
                None,
            )
            self.assertIsNotNone(spec, f"{path} should be classified as advanced/manual")
            self.assertIn("custom-effect parameter/control binding", spec.retirement)
            self.assertTrue(spec.allowed_raw_seams)
        for path in custom_effect_reference_paths:
            spec = next(
                (
                    spec
                    for spec in POLICY.ADVANCED_MANUAL_SURFACES
                    if spec.path == path
                ),
                None,
            )
            self.assertIsNotNone(
                spec, f"{path} should be classified as an advanced custom-effect reference"
            )
            self.assertIn("bounded custom-effect contract", spec.reason)
            self.assertTrue(spec.allowed_raw_seams)
        for path in streaming_import_paths:
            spec = next(
                (
                    spec
                    for spec in POLICY.ADVANCED_MANUAL_SURFACES
                    if spec.path == path
                ),
                None,
            )
            self.assertIsNotNone(
                spec, f"{path} should be classified as an advanced streaming import surface"
            )
            self.assertIn("streaming image upload", spec.reason)
            self.assertTrue(spec.allowed_raw_seams)
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/effects_demo.rs"
                for spec in POLICY.RENDERER_LAB_SURFACES
            )
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/assets_demo.rs"
                for spec in POLICY.RENDERER_LAB_SURFACES
            )
        )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/first_frame_smoke_demo.rs"
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
            )
        )
        for path in text_input_conformance_paths:
            spec = next(
                (
                    spec
                    for spec in POLICY.INTERNAL_HARNESS_SURFACES
                    if spec.path == path
                ),
                None,
            )
            self.assertIsNotNone(
                spec, f"{path} should be classified as a text/input conformance harness"
            )
            self.assertIn("conformance", spec.reason)
            self.assertTrue(spec.allowed_raw_seams)
        for path in memory_perf_harness_paths:
            spec = next(
                (
                    spec
                    for spec in POLICY.INTERNAL_HARNESS_SURFACES
                    if spec.path == path
                ),
                None,
            )
            self.assertIsNotNone(
                spec, f"{path} should be classified as a memory/perf harness"
            )
            self.assertTrue(
                "memory" in spec.reason or "perf" in spec.reason,
                f"{path} should explain its memory/perf harness role",
            )
            self.assertTrue(spec.allowed_raw_seams)
        for path in effect_reference_paths:
            spec = next(
                (
                    spec
                    for spec in POLICY.ADVANCED_MANUAL_SURFACES
                    if spec.path == path
                ),
                None,
            )
            self.assertIsNotNone(
                spec, f"{path} should be classified as an advanced effect reference"
            )
            self.assertIn("renderer", spec.reason)
            self.assertTrue(spec.retirement)
            self.assertTrue(spec.allowed_raw_seams)
        helper_spec = next(
            (
                spec
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
                if spec.path == "apps/fret-examples/src/custom_effect_v2_web_owner.rs"
            ),
            None,
        )
        self.assertIsNotNone(
            helper_spec,
            "custom-effect v2 web shared owner helper should be classified as an internal harness",
        )
        self.assertIn("ModelStore", helper_spec.allowed_raw_seams)
        self.assertIn("fret_core", helper_spec.allowed_raw_seams)
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
        container_query_docking_spec = next(
            (
                spec
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
                if spec.path == "apps/fret-examples/src/container_queries_docking_demo.rs"
            ),
            None,
        )
        self.assertIsNotNone(
            container_query_docking_spec,
            "container_queries_docking_demo should be classified as an internal harness",
        )
        self.assertIn("docking", container_query_docking_spec.reason)
        self.assertIn("FnDriver", container_query_docking_spec.allowed_raw_seams)
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
        for path in query_paths:
            self.assertTrue(
                any(spec.path == path for spec in POLICY.DEFAULT_AUTHORING_SURFACES),
                f"{path} should stay classified as default-clean query authoring",
            )
            self.assertFalse(
                any(spec.path == path for spec in POLICY.ADVANCED_MANUAL_SURFACES),
                f"{path} should not require advanced/manual quarantine",
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

    def test_custom_effect_v2_web_legacy_owner_surface_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/custom_effect_v2_web_demo.rs",
                """
                use crate::custom_effect_v2_web_owner::{
                    CustomEffectV2WebControlReset,
                    CustomEffectV2WebModelOwner,
                };

                struct DemoControls {
                    enabled: fret_runtime::Model<bool>,
                }

                impl CustomEffectV2WebControlReset for DemoControls {
                    fn reset_controls(&self, owner: &mut CustomEffectV2WebModelOwner<'_>) -> bool {
                        owner.set_model(&self.enabled, true)
                    }
                }

                fn bad(
                    app: &mut fret_app::App,
                    controls: &DemoControls,
                    show: &fret_runtime::Model<bool>,
                ) {
                    CustomEffectV2WebModelOwner::new(app.models_mut()).reset_controls(controls);
                    let _ = CustomEffectV2WebModelOwner::new(app.models_mut()).toggle_surface(show);
                    let _ = app.models_mut().update(show, |value| {
                        *value = !*value;
                        true
                    });
                    let _ = fret_runtime::ModelStore::update(app.models_mut(), show, |value| {
                        *value = !*value;
                        true
                    });
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/custom_effect_v2_web_demo.rs",
                        "advanced_manual",
                        "fixture custom-effect v2 web surface",
                        owner="examples-custom-effect-v2-web",
                        allowed_raw_seams=("fret_app", "fret_runtime", "ModelStore"),
                        retirement=POLICY.CUSTOM_EFFECT_V2_WEB_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_boundary_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-custom-effect-owner-boundary"
            ]
            self.assertGreaterEqual(len(owner_boundary_violations), 1)
            messages = "\n".join(violation.message for violation in owner_boundary_violations)
            self.assertIn("binding", messages)
            self.assertIn("legacy", messages)

    def test_custom_effect_v2_web_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/custom_effect_v2_web_demo.rs",
                """
                use crate::custom_effect_v2_web_owner::{
                    CustomEffectV2ScalarControl,
                    CustomEffectV2ScalarSpec,
                    CustomEffectV2WebControlBinding,
                    CustomEffectV2WebVariantControls,
                    CustomEffectV2WebVariantReset,
                };

                struct DemoControls {
                    strength: CustomEffectV2ScalarControl,
                }

                impl CustomEffectV2WebVariantControls for DemoControls {
                    fn reset_variant_controls(
                        &self,
                        reset: &mut CustomEffectV2WebVariantReset<'_, '_>,
                    ) -> bool {
                        self.strength.reset(reset)
                    }
                }

                struct WindowState {
                    binding: CustomEffectV2WebControlBinding,
                    controls: DemoControls,
                }

                fn build(app: &mut fret_app::App) -> WindowState {
                    WindowState {
                        binding: CustomEffectV2WebControlBinding::new(app.models_mut()),
                        controls: DemoControls {
                            strength: CustomEffectV2ScalarControl::new(
                                app.models_mut(),
                                CustomEffectV2ScalarSpec::new(1.0, 0.0, 2.0, 0.01),
                            ),
                        },
                    }
                }

                fn read(controls: &DemoControls, values: &[f32]) -> f32 {
                    let _ = controls.strength.values();
                    controls.strength.clamped_value(values)
                }

                fn ok(app: &mut fret_app::App, state: &WindowState) {
                    let _ = state.binding.toggle_surface_in(app.models_mut());
                    state.binding.reset_controls_in(app.models_mut(), &state.controls);
                }
                """,
            )
            write(
                root / "apps/fret-examples/src/custom_effect_v2_web_owner.rs",
                """
                use std::sync::Arc;
                use fret_core::scene::EffectParamsV1;
                use fret_runtime::{Model, ModelStore};
                use fret_ui_shadcn::facade::{IntoFloatVecModel, Slider};

                pub(crate) struct CustomEffectV2ParamSlot {
                    vec4: usize,
                    lane: usize,
                }

                impl CustomEffectV2ParamSlot {
                    pub(crate) const fn new(vec4: usize, lane: usize) -> Self {
                        assert!(vec4 < 4);
                        assert!(lane < 4);
                        Self { vec4, lane }
                    }

                    fn write(self, params: &mut EffectParamsV1, value: f32) {
                        params.vec4s[self.vec4][self.lane] = value;
                    }
                }

                pub(crate) struct CustomEffectV2ParamPack {
                    params: EffectParamsV1,
                }

                impl CustomEffectV2ParamPack {
                    pub(crate) fn new() -> Self {
                        Self {
                            params: EffectParamsV1::ZERO,
                        }
                    }

                    pub(crate) fn with_value(
                        mut self,
                        slot: CustomEffectV2ParamSlot,
                        value: f32,
                    ) -> Self {
                        slot.write(&mut self.params, value);
                        self
                    }

                    pub(crate) fn with_flag(
                        self,
                        slot: CustomEffectV2ParamSlot,
                        value: bool,
                    ) -> Self {
                        self.with_value(slot, if value { 1.0 } else { 0.0 })
                    }

                    pub(crate) fn finish(self) -> EffectParamsV1 {
                        self.params
                    }
                }

                pub(crate) struct CustomEffectV2ScalarSpec {
                    default: f32,
                    min: f32,
                    max: f32,
                    step: f32,
                }

                impl CustomEffectV2ScalarSpec {
                    pub(crate) fn new(default: f32, min: f32, max: f32, step: f32) -> Self {
                        Self { default, min, max, step }
                    }
                }

                pub(crate) struct CustomEffectV2ScalarControl {
                    model: Model<Vec<f32>>,
                    spec: CustomEffectV2ScalarSpec,
                }

                impl CustomEffectV2ScalarControl {
                    pub(crate) fn new(models: &mut ModelStore, spec: CustomEffectV2ScalarSpec) -> Self {
                        Self {
                            model: models.insert(vec![spec.default]),
                            spec,
                        }
                    }

                    pub(crate) fn values(&self) -> &Model<Vec<f32>> {
                        &self.model
                    }

                    pub(crate) fn clamped_value(
                        &self,
                        values: &[f32],
                    ) -> f32 {
                        values.first().copied().unwrap_or(self.spec.default).clamp(self.spec.min, self.spec.max)
                    }

                    pub(crate) fn rounded_u32_value(
                        &self,
                        values: &[f32],
                    ) -> u32 {
                        self.clamped_value(values).round() as u32
                    }

                    pub(crate) fn slider(&self) -> Slider {
                        Slider::new(self).range(self.spec.min, self.spec.max).step(self.spec.step)
                    }

                    pub(crate) fn reset(
                        &self,
                        reset: &mut CustomEffectV2WebVariantReset<'_, '_>,
                    ) -> bool {
                        reset.set_model(&self.model, vec![self.spec.default])
                    }
                }

                impl IntoFloatVecModel for &CustomEffectV2ScalarControl {
                    fn into_float_vec_model(self) -> Model<Vec<f32>> {
                        self.model.clone()
                    }
                }

                struct CustomEffectV2WebModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> CustomEffectV2WebModelOwner<'a> {
                    fn new(models: &'a mut ModelStore) -> Self {
                        Self { models }
                    }

                    fn set_model<T: std::any::Any>(&mut self, model: &Model<T>, value: T) -> bool {
                        self.models.update(model, |current| {
                            *current = value;
                            true
                        }).unwrap_or(false)
                    }
                }

                pub(crate) struct CustomEffectV2WebControlBinding {
                    show: Model<bool>,
                    common: CustomEffectV2WebCommonControls,
                }

                impl CustomEffectV2WebControlBinding {
                    pub(crate) fn new(models: &mut ModelStore) -> Self {
                        Self {
                            show: models.insert(true),
                            common: CustomEffectV2WebCommonControls {
                                enabled: models.insert(true),
                                mode: models.insert(Some(Arc::from("backdrop"))),
                            },
                        }
                    }

                    pub(crate) fn toggle_surface_in(&self, models: &mut ModelStore) -> bool {
                        CustomEffectV2WebModelOwner::new(models).set_model(&self.show, true)
                    }

                    pub(crate) fn reset_controls_in<C: CustomEffectV2WebVariantControls>(
                        &self,
                        models: &mut ModelStore,
                        controls: &C,
                    ) -> bool {
                        let mut owner = CustomEffectV2WebModelOwner::new(models);
                        let mut reset = CustomEffectV2WebVariantReset { owner: &mut owner };
                        controls.reset_variant_controls(&mut reset)
                    }
                }

                struct CustomEffectV2WebCommonControls {
                    enabled: Model<bool>,
                    mode: Model<Option<Arc<str>>>,
                }

                pub(crate) struct CustomEffectV2WebVariantReset<'a, 'models> {
                    owner: &'a mut CustomEffectV2WebModelOwner<'models>,
                }

                impl<'a, 'models> CustomEffectV2WebVariantReset<'a, 'models> {
                    pub(crate) fn set_model<T: std::any::Any>(
                        &mut self,
                        model: &Model<T>,
                        value: T,
                    ) -> bool {
                        self.owner.set_model(model, value)
                    }
                }

                pub(crate) trait CustomEffectV2WebVariantControls {
                    fn reset_variant_controls(
                        &self,
                        reset: &mut CustomEffectV2WebVariantReset<'_, '_>,
                    ) -> bool;
                }

                #[cfg(test)]
                mod tests {
                    fn scalar_control_uses_app_model_store() {
                        let _ = fret_app::App::new();
                    }
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/custom_effect_v2_web_demo.rs",
                        "advanced_manual",
                        "fixture custom-effect v2 web surface",
                        owner="examples-custom-effect-v2-web",
                        allowed_raw_seams=("fret_app",),
                        retirement=POLICY.CUSTOM_EFFECT_V2_WEB_RETIREMENT,
                    )
                ],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/custom_effect_v2_web_owner.rs",
                        "internal_harness",
                        "fixture custom-effect v2 web owner helper",
                        owner="examples-custom-effect-v2-web",
                        allowed_raw_seams=("fret_core", "fret_runtime", "ModelStore"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertEqual([], violations)
            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "advanced-surface-custom-effect-owner-boundary"
                ]
            )

    def test_gizmo3d_direct_demo_model_updates_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/gizmo3d_demo.rs",
                """
                struct Gizmo3dDemoModel;
                struct Gizmo3dDemoModelBinding {
                    model: fret_runtime::Model<Gizmo3dDemoModel>,
                }

                fn handle_required_markers(
                    app: &mut fret_app::App,
                    state: &WindowState,
                    model: Gizmo3dDemoModelBinding,
                ) {
                    let _ = model.handle_viewport_input(app, &event);
                    let _ = state.demo.step_frame_animation(app, Instant::now());
                    let _ = state.demo.frame_render_snapshot(app, size);
                }

                impl Gizmo3dDemoModelBinding {
                    fn handle_viewport_input(
                        &self,
                    ) {}

                    fn step_frame_animation(
                        &self,
                    ) {}

                    fn frame_render_snapshot(
                        &self,
                    ) {}
                }

                fn bad(app: &mut fret_app::App, state: &WindowState, model: Gizmo3dDemoModelBinding) {
                    let _ = state.demo.update(app, |m, _cx| m.cancel());
                    let _ = model.update(app, |m, _cx| m.cancel());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/gizmo3d_demo.rs",
                        "advanced_manual",
                        "fixture gizmo3d proof surface",
                        owner="examples-gizmo3d",
                        allowed_raw_seams=("fret_app", "fret_runtime"),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            gizmo_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-gizmo3d-owner-boundary"
            ]
            self.assertEqual(2, len(gizmo_violations))
            messages = "\n".join(violation.message for violation in gizmo_violations)
            self.assertIn("state.demo.update", messages)
            self.assertIn("model.update", messages)

    def test_gizmo3d_binding_owner_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/gizmo3d_demo.rs",
                """
                struct Gizmo3dDemoModel;
                struct Gizmo3dFrameRenderSnapshot;
                struct Gizmo3dDemoModelBinding {
                    model: fret_runtime::Model<Gizmo3dDemoModel>,
                }

                impl Gizmo3dDemoModelBinding {
                    fn handle_viewport_input(
                        &self,
                        app: &mut fret_app::App,
                    ) {
                        let _ = self.update(app, |model, _cx| model.cancel());
                    }

                    fn step_frame_animation(
                        &self,
                        app: &mut fret_app::App,
                    ) {
                        let _ = self.update(app, |model, _cx| model.cancel());
                    }

                    fn frame_render_snapshot(
                        &self,
                        app: &mut fret_app::App,
                    ) -> Gizmo3dFrameRenderSnapshot {
                        self.update(app, |model, _cx| model.snapshot()).unwrap()
                    }
                }

                fn ok(app: &mut fret_app::App, state: &WindowState, model: Gizmo3dDemoModelBinding) {
                    let _ = model.handle_viewport_input(app, &event);
                    let _ = state.demo.step_frame_animation(app, Instant::now());
                    let _ = state.demo.frame_render_snapshot(app, size);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/gizmo3d_demo.rs",
                        "advanced_manual",
                        "fixture gizmo3d proof surface",
                        owner="examples-gizmo3d",
                        allowed_raw_seams=("fret_app", "fret_runtime"),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "advanced-surface-gizmo3d-owner-boundary"
                ]
            )

    def test_embedded_viewport_direct_model_updates_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/embedded_viewport_demo.rs",
                """
                use fret_runtime::{FrameId, ModelStore, TickId};

                struct EmbeddedViewportDemoModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> EmbeddedViewportDemoModelOwner<'a> {
                    fn set_last_input(
                        &mut self,
                    ) {
                        let _ = self.models.update(&models.last_input, |_| true);
                    }
                }

                impl embedded::EmbeddedViewportView for EmbeddedViewportDemoView {
                    fn record_embedded_viewport(
                        &mut self,
                    ) {}
                }

                fn run() {
                    FretApp::new("embedded-viewport-demo")
                        .view_with_hooks::<EmbeddedViewportDemoView>(|d| d.drive_embedded_viewport())?;
                }

                fn init(app: &mut App, models: &embedded::EmbeddedViewportModels) {
                    let _ = EmbeddedViewportDemoModelOwner::new(app.models_mut()).set_last_input(
                        &models,
                        "ready",
                    );
                    let _ = app.models_mut().update(&models.last_input, |value| {
                        *value = "bad".into();
                        true
                    });
                    let _ = ModelStore::update(app.models_mut(), &models.last_input, |value| {
                        *value = "bad".into();
                        true
                    });
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/embedded_viewport_demo.rs",
                        "advanced_manual",
                        "fixture embedded viewport proof surface",
                        owner="examples-embedded-viewport",
                        allowed_raw_seams=("fret_runtime", "ModelStore"),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-embedded-viewport-owner-boundary"
            ]
            self.assertEqual(2, len(owner_violations))
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("models_mut().update", messages)
            self.assertIn("ModelStore::update", messages)

    def test_embedded_viewport_owner_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/embedded_viewport_demo.rs",
                """
                use fret_runtime::{FrameId, ModelStore, TickId};

                struct EmbeddedViewportDemoModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> EmbeddedViewportDemoModelOwner<'a> {
                    fn set_last_input(
                        &mut self,
                    ) {
                        let _ = self.models.update(&models.last_input, |_| true);
                    }
                }

                impl embedded::EmbeddedViewportView for EmbeddedViewportDemoView {
                    fn record_embedded_viewport(
                        &mut self,
                    ) {}
                }

                fn run() {
                    FretApp::new("embedded-viewport-demo")
                        .view_with_hooks::<EmbeddedViewportDemoView>(|d| d.drive_embedded_viewport())?;
                }

                fn init(app: &mut App, models: &embedded::EmbeddedViewportModels) {
                    let _ = EmbeddedViewportDemoModelOwner::new(app.models_mut()).set_last_input(
                        &models,
                        "ready",
                    );
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/embedded_viewport_demo.rs",
                        "advanced_manual",
                        "fixture embedded viewport proof surface",
                        owner="examples-embedded-viewport",
                        allowed_raw_seams=("fret_runtime", "ModelStore"),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule
                    == "advanced-surface-embedded-viewport-owner-boundary"
                ]
            )

    def test_external_imports_direct_visibility_writes_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/external_texture_imports_demo.rs",
                """
                use fret_runtime::ModelStore;

                use crate::external_imports_owner::ExternalImportsModelOwner;

                struct ViewState {
                    show: fret_runtime::Model<bool>,
                }

                fn toggle(app: &mut App, st: &ViewState) {
                    ExternalImportsModelOwner::new(app.models_mut()).toggle_surface(&st.show);
                    let _ = app.models_mut().update(&st.show, |show| {
                        *show = !*show;
                        true
                    });
                    let _ = ModelStore::update(app.models_mut(), &st.show, |show| {
                        *show = !*show;
                        true
                    });
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/external_texture_imports_demo.rs",
                        "advanced_manual",
                        "fixture external imports proof surface",
                        owner="examples-external-imports",
                        allowed_raw_seams=("fret_runtime", "ModelStore"),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-external-imports-owner-boundary"
            ]
            self.assertEqual(2, len(owner_violations))
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("models_mut().update", messages)
            self.assertIn("ModelStore::update", messages)

    def test_external_imports_owner_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/external_imports_owner.rs",
                """
                use fret_runtime::{Model, ModelStore};

                pub(crate) struct ExternalImportsModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> ExternalImportsModelOwner<'a> {
                    pub(crate) fn toggle_surface(&mut self, show: &Model<bool>) -> bool {
                        self.models
                            .update(show, |show| {
                                *show = !*show;
                                true
                            })
                            .unwrap_or(false)
                    }
                }
                """,
            )
            write(
                root / "apps/fret-examples/src/external_texture_imports_demo.rs",
                """
                use fret_runtime::PlatformCapabilities;

                use crate::external_imports_owner::ExternalImportsModelOwner;

                struct ViewState {
                    show: fret_runtime::Model<bool>,
                }

                fn toggle(app: &mut App, st: &ViewState) {
                    ExternalImportsModelOwner::new(app.models_mut()).toggle_surface(&st.show);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/external_texture_imports_demo.rs",
                        "advanced_manual",
                        "fixture external imports proof surface",
                        owner="examples-external-imports",
                        allowed_raw_seams=("fret_runtime",),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/external_imports_owner.rs",
                        "internal_harness",
                        "fixture external imports owner helper",
                        owner="examples-external-imports",
                        allowed_raw_seams=("fret_runtime", "ModelStore"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule
                    == "advanced-surface-external-imports-owner-boundary"
                ]
            )

    def test_window_hit_test_probe_broad_manual_prelude_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/window_hit_test_probe_demo.rs",
                """
                use fret::advanced::prelude::*;
                use fret::advanced::KernelApp;
                use fret::advanced::interop::run_native_with_compat_driver;
                use fret_app::{CreateWindowKind, CreateWindowRequest, Effect, WindowRequest};
                use fret_bootstrap::ui_app_driver::{self, ViewElements};
                use fret_runtime::Model;

                fn run() -> anyhow::Result<()> {
                    let driver = ui_app_driver::UiAppDriver::new(
                        "window-hit-test-probe-demo",
                        init_window,
                        view,
                    )
                    .into_fn_driver();
                    run_native_with_compat_driver(config, KernelApp::new(), driver)?;
                    Ok(())
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/window_hit_test_probe_demo.rs",
                        "advanced_manual",
                        "fixture window hit-test probe surface",
                        owner="examples-window-hit-test-probe",
                        allowed_raw_seams=(
                            "fret::advanced",
                            "fret_app",
                            "fret_runtime",
                        ),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            manual_driver_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-window-hit-test-probe-boundary"
            ]
            self.assertEqual(1, len(manual_driver_violations))
            self.assertIn("advanced::prelude", manual_driver_violations[0].message)

    def test_window_hit_test_probe_explicit_manual_driver_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/window_hit_test_probe_demo.rs",
                """
                use fret::advanced::KernelApp;
                use fret::advanced::interop::run_native_with_compat_driver;
                use fret_app::{CreateWindowKind, CreateWindowRequest, Effect, WindowRequest};
                use fret_bootstrap::ui_app_driver::{self, ViewElements};
                use fret_runtime::Model;

                fn run() -> anyhow::Result<()> {
                    let driver = ui_app_driver::UiAppDriver::new(
                        "window-hit-test-probe-demo",
                        init_window,
                        view,
                    )
                    .into_fn_driver();
                    run_native_with_compat_driver(config, KernelApp::new(), driver)?;
                    Ok(())
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/window_hit_test_probe_demo.rs",
                        "advanced_manual",
                        "fixture window hit-test probe surface",
                        owner="examples-window-hit-test-probe",
                        allowed_raw_seams=(
                            "fret::advanced",
                            "fret_app",
                            "fret_runtime",
                        ),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "advanced-surface-window-hit-test-probe-boundary"
                ]
            )

    def test_components_gallery_direct_model_writes_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/components_gallery.rs",
                """
                use fret_runtime::{ModelStore, PlatformCapabilities};

                struct ComponentsGalleryModelBundle;
                impl ComponentsGalleryModelBundle {
                    fn new(models: &mut ModelStore) -> Self {
                        Self
                    }
                }

                struct ComponentsGalleryModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> ComponentsGalleryModelOwner<'a> {
                    fn update<T: Any, R>(&mut self, model: &Model<T>, f: impl FnOnce(&mut T) -> R) -> Option<R> {
                        None
                    }
                    fn set<T: Any>(&mut self, model: &Model<T>, value: T) -> bool {
                        false
                    }
                    fn set_last_action(&mut self) {}
                    fn open_command_palette(&mut self) {}
                    fn close_transient_surfaces(&mut self) {}
                }

                fn components_gallery_set_last_action() {}
                fn components_gallery_open_command_palette() {}
                fn components_gallery_close_transient_surfaces() {}
                fn route_helpers(app: &mut App, state: &State) {
                    components_gallery_close_transient_surfaces(app, state);
                    components_gallery_open_command_palette(app, state);
                    components_gallery_set_last_action(app, state, "context_menu.action");
                }

                fn build_ui(app: &mut App) {
                    let _ = ComponentsGalleryModelBundle::new(app.models_mut());
                    let _ = app.models_mut().insert(35.0f32);
                }

                fn bad(app: &mut App, state: &State) {
                    ComponentsGalleryModelOwner::new(app.models_mut()).update(&state.progress, |_| {});
                    let _ = app.models_mut().update(&state.progress, |_| true);
                    let _ = ModelStore::update(app.models_mut(), &state.progress, |_| true);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/components_gallery.rs",
                        "advanced_manual",
                        "fixture components gallery surface",
                        owner="examples-components-gallery",
                        allowed_raw_seams=("fret_runtime", "ModelStore"),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-components-gallery-owner-boundary"
            ]
            self.assertEqual(3, len(owner_violations))
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("models_mut().insert", messages)
            self.assertIn("models_mut().update", messages)
            self.assertIn("ModelStore::update", messages)

    def test_components_gallery_owner_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/components_gallery.rs",
                """
                use fret_runtime::{ModelStore, PlatformCapabilities};

                struct ComponentsGalleryModelBundle;
                impl ComponentsGalleryModelBundle {
                    fn new(models: &mut ModelStore) -> Self {
                        let _ = models.insert(35.0f32);
                        Self
                    }
                }

                struct ComponentsGalleryModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> ComponentsGalleryModelOwner<'a> {
                    fn update<T: Any, R>(&mut self, model: &Model<T>, f: impl FnOnce(&mut T) -> R) -> Option<R> {
                        None
                    }
                    fn set<T: Any>(&mut self, model: &Model<T>, value: T) -> bool {
                        false
                    }
                    fn set_last_action(&mut self) {}
                    fn open_command_palette(&mut self) {}
                    fn close_transient_surfaces(&mut self) {}
                }

                fn components_gallery_set_last_action() {}
                fn components_gallery_open_command_palette() {}
                fn components_gallery_close_transient_surfaces() {}
                fn route_helpers(app: &mut App, state: &State) {
                    components_gallery_close_transient_surfaces(app, state);
                    components_gallery_open_command_palette(app, state);
                    components_gallery_set_last_action(app, state, "context_menu.action");
                }

                fn build_ui(app: &mut App) {
                    let _ = ComponentsGalleryModelBundle::new(app.models_mut());
                }

                fn ok(app: &mut App, state: &State) {
                    ComponentsGalleryModelOwner::new(app.models_mut()).update(&state.progress, |_| {});
                    components_gallery_set_last_action();
                    components_gallery_open_command_palette();
                    components_gallery_close_transient_surfaces();
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/components_gallery.rs",
                        "advanced_manual",
                        "fixture components gallery surface",
                        owner="examples-components-gallery",
                        allowed_raw_seams=("fret_runtime", "ModelStore"),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule
                    == "advanced-surface-components-gallery-owner-boundary"
                ]
            )

    def test_virtual_list_stress_direct_model_writes_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/virtual_list_stress_demo.rs",
                """
                use fret_runtime::{ModelStore, PlatformCapabilities};

                struct VirtualListStressControls {
                    tall_rows_enabled: Model<bool>,
                    reversed: Model<bool>,
                    items_revision: Model<u64>,
                }

                struct VirtualListStressSnapshot;

                impl VirtualListStressControls {
                    fn new(models: &mut ModelStore) -> Self {
                        Self {
                            tall_rows_enabled: models.insert(false),
                            reversed: models.insert(false),
                            items_revision: models.insert(0u64),
                        }
                    }

                    fn toggle_rows_enabled(&self, models: &mut ModelStore) -> bool {
                        false
                    }

                    fn toggle_reversed_and_bump_revision(&self, models: &mut ModelStore) -> bool {
                        false
                    }

                    fn layout_snapshot(&self, cx: &mut ElementContext<'_, App>) -> VirtualListStressSnapshot {
                        VirtualListStressSnapshot
                    }
                }

                struct VirtualListStressWindowState {
                    controls: VirtualListStressControls,
                    progress: Model<u64>,
                }

                fn build_ui(app: &mut App) {
                    let controls = VirtualListStressControls::new(app.models_mut());
                    let _ = app.models_mut().insert(false);
                }

                fn handle_event(app: &mut App, state: &mut VirtualListStressWindowState) {
                    let _ = state.controls.toggle_rows_enabled(app.models_mut());
                    let _ = state.controls.toggle_reversed_and_bump_revision(app.models_mut());
                    let _ = app.models_mut().update(&state.progress, |_| true);
                    let _ = ModelStore::update(app.models_mut(), &state.progress, |_| true);
                }

                fn render(cx: &mut ElementContext<'_, App>, state: &VirtualListStressWindowState) {
                    let controls = state.controls.layout_snapshot(cx);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/virtual_list_stress_demo.rs",
                        "internal_harness",
                        "fixture virtual-list stress harness",
                        owner="examples-virtual-list-stress",
                        allowed_raw_seams=(
                            "fret_app",
                            "fret_runtime",
                            "ElementContext",
                            "ModelStore",
                        ),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "internal_harness-virtual-list-stress-controls-boundary"
            ]
            self.assertEqual(3, len(owner_violations))
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("models_mut().insert", messages)
            self.assertIn("models_mut().update", messages)
            self.assertIn("ModelStore::update", messages)

    def test_virtual_list_stress_controls_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/virtual_list_stress_demo.rs",
                """
                use fret_runtime::{ModelStore, PlatformCapabilities};

                struct VirtualListStressControls {
                    tall_rows_enabled: Model<bool>,
                    reversed: Model<bool>,
                    items_revision: Model<u64>,
                }

                struct VirtualListStressSnapshot;

                impl VirtualListStressControls {
                    fn new(models: &mut ModelStore) -> Self {
                        Self {
                            tall_rows_enabled: models.insert(false),
                            reversed: models.insert(false),
                            items_revision: models.insert(0u64),
                        }
                    }

                    fn toggle_rows_enabled(&self, models: &mut ModelStore) -> bool {
                        false
                    }

                    fn toggle_reversed_and_bump_revision(&self, models: &mut ModelStore) -> bool {
                        false
                    }

                    fn layout_snapshot(&self, cx: &mut ElementContext<'_, App>) -> VirtualListStressSnapshot {
                        VirtualListStressSnapshot
                    }
                }

                struct VirtualListStressWindowState {
                    controls: VirtualListStressControls,
                }

                fn build_ui(app: &mut App) {
                    let controls = VirtualListStressControls::new(app.models_mut());
                    let _ = controls;
                }

                fn handle_event(app: &mut App, state: &mut VirtualListStressWindowState) {
                    let _ = state.controls.toggle_rows_enabled(app.models_mut());
                    let _ = state.controls.toggle_reversed_and_bump_revision(app.models_mut());
                }

                fn render(cx: &mut ElementContext<'_, App>, state: &VirtualListStressWindowState) {
                    let controls = state.controls.layout_snapshot(cx);
                    let _ = controls;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/virtual_list_stress_demo.rs",
                        "internal_harness",
                        "fixture virtual-list stress harness",
                        owner="examples-virtual-list-stress",
                        allowed_raw_seams=(
                            "fret_app",
                            "fret_runtime",
                            "ElementContext",
                            "ModelStore",
                        ),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule
                    == "internal_harness-virtual-list-stress-controls-boundary"
                ]
            )

    def test_table_stress_direct_model_writes_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/table_stress_demo.rs",
                """
                use fret_runtime::{ModelStore, PlatformCapabilities};

                struct TableStressControls {
                    table_state: Model<TableState>,
                    items_revision: Model<u64>,
                }

                struct TableStressSnapshot;

                struct TableStressModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> TableStressModelOwner<'a> {
                    fn new(models: &'a mut ModelStore) -> Self {
                        Self { models }
                    }

                    fn update_table_state(&mut self, state: &Model<TableState>, f: impl FnOnce(&mut TableState)) -> bool {
                        false
                    }

                    fn toggle_sorting(&mut self, state: &Model<TableState>) -> bool { false }
                    fn toggle_role_filter(&mut self, state: &Model<TableState>) -> bool { false }
                    fn toggle_global_filter(&mut self, state: &Model<TableState>) -> bool { false }
                    fn clear_filters(&mut self, state: &Model<TableState>) -> bool { false }
                    fn bump_items_revision(&mut self, revision: &Model<u64>) -> bool { false }
                }

                impl TableStressControls {
                    fn new(models: &mut ModelStore, row_count: usize) -> Self {
                        Self {
                            table_state: models.insert(TableState::default()),
                            items_revision: models.insert(1u64),
                        }
                    }

                    fn table_model(&self) -> Model<TableState> {
                        self.table_state.clone()
                    }

                    fn toggle_sorting(&self, app: &mut App) -> bool {
                        TableStressModelOwner::new(app.models_mut()).toggle_sorting(&self.table_state)
                    }

                    fn toggle_role_filter(&self, app: &mut App) -> bool {
                        TableStressModelOwner::new(app.models_mut()).toggle_role_filter(&self.table_state)
                    }

                    fn toggle_global_filter(&self, app: &mut App) -> bool {
                        TableStressModelOwner::new(app.models_mut()).toggle_global_filter(&self.table_state)
                    }

                    fn clear_filters(&self, app: &mut App) -> bool {
                        TableStressModelOwner::new(app.models_mut()).clear_filters(&self.table_state)
                    }

                    fn bump_items_revision(&self, app: &mut App) -> bool {
                        TableStressModelOwner::new(app.models_mut()).bump_items_revision(&self.items_revision)
                    }

                    fn render_snapshot(&self, cx: &mut ElementContext<'_, App>) -> TableStressSnapshot {
                        TableStressSnapshot
                    }
                }

                struct TableStressWindowState {
                    controls: TableStressControls,
                    progress: Model<u64>,
                }

                fn build_ui(app: &mut App) {
                    let controls = TableStressControls::new(app.models_mut(), 10);
                    let _ = app.models_mut().insert(0u64);
                }

                fn handle_event(app: &mut App, state: &mut TableStressWindowState) {
                    let _ = state.controls.toggle_sorting(app);
                    let _ = state.controls.toggle_role_filter(app);
                    let _ = state.controls.toggle_global_filter(app);
                    let _ = state.controls.clear_filters(app);
                    let _ = state.controls.bump_items_revision(app);
                    let _ = app.models_mut().update(&state.progress, |_| true);
                    let _ = ModelStore::update(app.models_mut(), &state.progress, |_| true);
                }

                fn render(cx: &mut ElementContext<'_, App>, state: &TableStressWindowState) {
                    let table_state = state.controls.table_model();
                    let controls = state.controls.render_snapshot(cx);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/table_stress_demo.rs",
                        "internal_harness",
                        "fixture table stress harness",
                        owner="examples-table-stress",
                        allowed_raw_seams=(
                            "fret_app",
                            "fret_runtime",
                            "ElementContext",
                            "ModelStore",
                        ),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "internal_harness-table-stress-controls-boundary"
            ]
            self.assertEqual(3, len(owner_violations))
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("models_mut().insert", messages)
            self.assertIn("models_mut().update", messages)
            self.assertIn("ModelStore::update", messages)

    def test_table_stress_controls_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/table_stress_demo.rs",
                """
                use fret_runtime::{ModelStore, PlatformCapabilities};

                struct TableStressControls {
                    table_state: Model<TableState>,
                    items_revision: Model<u64>,
                }

                struct TableStressSnapshot;

                struct TableStressModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> TableStressModelOwner<'a> {
                    fn new(models: &'a mut ModelStore) -> Self {
                        Self { models }
                    }

                    fn update_table_state(&mut self, state: &Model<TableState>, f: impl FnOnce(&mut TableState)) -> bool {
                        false
                    }

                    fn toggle_sorting(&mut self, state: &Model<TableState>) -> bool { false }
                    fn toggle_role_filter(&mut self, state: &Model<TableState>) -> bool { false }
                    fn toggle_global_filter(&mut self, state: &Model<TableState>) -> bool { false }
                    fn clear_filters(&mut self, state: &Model<TableState>) -> bool { false }
                    fn bump_items_revision(&mut self, revision: &Model<u64>) -> bool { false }
                }

                impl TableStressControls {
                    fn new(models: &mut ModelStore, row_count: usize) -> Self {
                        Self {
                            table_state: models.insert(TableState::default()),
                            items_revision: models.insert(1u64),
                        }
                    }

                    fn table_model(&self) -> Model<TableState> {
                        self.table_state.clone()
                    }

                    fn toggle_sorting(&self, app: &mut App) -> bool {
                        TableStressModelOwner::new(app.models_mut()).toggle_sorting(&self.table_state)
                    }

                    fn toggle_role_filter(&self, app: &mut App) -> bool {
                        TableStressModelOwner::new(app.models_mut()).toggle_role_filter(&self.table_state)
                    }

                    fn toggle_global_filter(&self, app: &mut App) -> bool {
                        TableStressModelOwner::new(app.models_mut()).toggle_global_filter(&self.table_state)
                    }

                    fn clear_filters(&self, app: &mut App) -> bool {
                        TableStressModelOwner::new(app.models_mut()).clear_filters(&self.table_state)
                    }

                    fn bump_items_revision(&self, app: &mut App) -> bool {
                        TableStressModelOwner::new(app.models_mut()).bump_items_revision(&self.items_revision)
                    }

                    fn render_snapshot(&self, cx: &mut ElementContext<'_, App>) -> TableStressSnapshot {
                        TableStressSnapshot
                    }
                }

                struct TableStressWindowState {
                    controls: TableStressControls,
                }

                fn build_ui(app: &mut App) {
                    let controls = TableStressControls::new(app.models_mut(), 10);
                    let _ = controls;
                }

                fn handle_event(app: &mut App, state: &mut TableStressWindowState) {
                    let _ = state.controls.toggle_sorting(app);
                    let _ = state.controls.toggle_role_filter(app);
                    let _ = state.controls.toggle_global_filter(app);
                    let _ = state.controls.clear_filters(app);
                    let _ = state.controls.bump_items_revision(app);
                }

                fn render(cx: &mut ElementContext<'_, App>, state: &TableStressWindowState) {
                    let table_state = state.controls.table_model();
                    let controls = state.controls.render_snapshot(cx);
                    let _ = (table_state, controls);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/table_stress_demo.rs",
                        "internal_harness",
                        "fixture table stress harness",
                        owner="examples-table-stress",
                        allowed_raw_seams=(
                            "fret_app",
                            "fret_runtime",
                            "ElementContext",
                            "ModelStore",
                        ),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "internal_harness-table-stress-controls-boundary"
                ]
            )

    def test_canvas_datagrid_stress_raw_control_plumbing_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/canvas_datagrid_stress_demo.rs",
                """
                use fret::app::{AppLocalStateExt as _, AppLocalStateTxnExt as _, LocalState};

                struct CanvasDataGridStressWindowState {
                    ui: UiTree<App>,
                    rows: Arc<Vec<u64>>,
                    cols: Arc<Vec<u64>>,
                    cell_texts: Arc<Vec<Arc<str>>>,
                    controls: CanvasDataGridStressControls,
                    variable_sizes: Model<bool>,
                    clamp_rows: Model<bool>,
                    revision: Model<u64>,
                    grid_output: Model<shadcn::DataGridCanvasOutput>,
                }

                struct CanvasDataGridStressControls {
                    variable_sizes: Model<bool>,
                    clamp_rows: Model<bool>,
                    revision: Model<u64>,
                }

                impl CanvasDataGridStressControls {
                    fn new(app: &mut App) -> Self {
                        Self {
                            variable_sizes: app.models_mut().insert(false),
                            clamp_rows: app.models_mut().insert(false),
                            revision: app.models_mut().insert(1u64),
                        }
                    }

                    fn layout_snapshot(&self, cx: &mut ElementContext<'_, App>) -> CanvasDataGridStressControlsSnapshot {
                        CanvasDataGridStressControlsSnapshot
                    }
                }

                fn build_ui(app: &mut App) {
                    let controls = CanvasDataGridStressControls::new(app);
                    let grid_output = app.local_state(shadcn::DataGridCanvasOutput::default());
                    let variable_sizes = app.models_mut().insert(false);
                    let clamp_rows = app.models_mut().insert(false);
                    let revision = app.models_mut().insert(1u64);
                    let raw_grid_output = app.models_mut().insert(shadcn::DataGridCanvasOutput::default());
                    let _ = (controls, grid_output, variable_sizes, clamp_rows, revision, raw_grid_output);
                }

                fn gpu_frame_prepare(app: &mut App, state: &mut CanvasDataGridStressWindowState) {
                    let grid = app.local_state_txn(|tx| tx.value_or_default(&state.grid_output));
                }

                fn render(cx: &mut ElementContext<'_, App>, state: &mut CanvasDataGridStressWindowState) {
                    let controls = state.controls.layout_snapshot(cx);
                    let grid = state.grid_output.layout_value(cx);
                    let mut axis = shadcn::DataGridCanvasAxis::new(Arc::clone(&rows), controls.revision, Px(24.0));
                    let mut axis = shadcn::DataGridCanvasAxis::new(Arc::clone(&cols), controls.revision, Px(120.0));
                    let grid = shadcn::DataGrid::new(rows_axis, cols_axis)
                        .output_model(state.grid_output.clone());
                    let _ = (&state.variable_sizes, &state.clamp_rows, &state.revision);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/canvas_datagrid_stress_demo.rs",
                        "internal_harness",
                        "fixture canvas datagrid stress harness",
                        owner="examples-canvas-datagrid-stress",
                        allowed_raw_seams=(
                            "fret::advanced",
                            "fret_app",
                            "fret_core",
                            "fret_launch",
                            "fret_runtime",
                            "fret_ui",
                            "AnyElement",
                            "ElementContext",
                            "FnDriver",
                            "UiTree",
                        ),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "internal_harness-canvas-datagrid-stress-controls-boundary"
            ]
            self.assertEqual(7, len(owner_violations))
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("legacy-window-state-control-field", messages)
            self.assertIn("legacy-grid-output-model", messages)
            self.assertIn("direct-variable-size-model-insert", messages)
            self.assertIn("direct-clamp-rows-model-insert", messages)
            self.assertIn("direct-revision-model-insert", messages)
            self.assertIn("legacy-state-control-reference", messages)

    def test_canvas_datagrid_stress_controls_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/canvas_datagrid_stress_demo.rs",
                """
                use fret::app::{AppLocalStateExt as _, AppLocalStateTxnExt as _, LocalState};

                struct CanvasDataGridStressWindowState {
                    ui: UiTree<App>,
                    rows: Arc<Vec<u64>>,
                    cols: Arc<Vec<u64>>,
                    cell_texts: Arc<Vec<Arc<str>>>,
                    controls: CanvasDataGridStressControls,
                    grid_output: LocalState<shadcn::DataGridCanvasOutput>,
                }

                struct CanvasDataGridStressControls {
                    variable_sizes: Model<bool>,
                    clamp_rows: Model<bool>,
                    revision: Model<u64>,
                }

                impl CanvasDataGridStressControls {
                    fn new(app: &mut App) -> Self {
                        Self {
                            variable_sizes: app.models_mut().insert(false),
                            clamp_rows: app.models_mut().insert(false),
                            revision: app.models_mut().insert(1u64),
                        }
                    }

                    fn layout_snapshot(&self, cx: &mut ElementContext<'_, App>) -> CanvasDataGridStressControlsSnapshot {
                        CanvasDataGridStressControlsSnapshot
                    }
                }

                fn build_ui(app: &mut App) {
                    let controls = CanvasDataGridStressControls::new(app);
                    let grid_output = app.local_state(shadcn::DataGridCanvasOutput::default());
                    let _ = (controls, grid_output);
                }

                fn gpu_frame_prepare(app: &mut App, state: &mut CanvasDataGridStressWindowState) {
                    let grid = app.local_state_txn(|tx| tx.value_or_default(&state.grid_output));
                }

                fn render(cx: &mut ElementContext<'_, App>, state: &mut CanvasDataGridStressWindowState) {
                    let controls = state.controls.layout_snapshot(cx);
                    let grid = state.grid_output.layout_value(cx);
                    let mut axis = shadcn::DataGridCanvasAxis::new(Arc::clone(&rows), controls.revision, Px(24.0));
                    let mut axis = shadcn::DataGridCanvasAxis::new(Arc::clone(&cols), controls.revision, Px(120.0));
                    let grid = shadcn::DataGrid::new(rows_axis, cols_axis)
                        .output_model(state.grid_output.clone());
                    let _ = (controls, grid, axis);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/canvas_datagrid_stress_demo.rs",
                        "internal_harness",
                        "fixture canvas datagrid stress harness",
                        owner="examples-canvas-datagrid-stress",
                        allowed_raw_seams=(
                            "fret::advanced",
                            "fret_app",
                            "fret_core",
                            "fret_launch",
                            "fret_runtime",
                            "fret_ui",
                            "AnyElement",
                            "ElementContext",
                            "FnDriver",
                            "UiTree",
                        ),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule
                    == "internal_harness-canvas-datagrid-stress-controls-boundary"
                ]
            )

    def test_datatable_raw_output_model_plumbing_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/datatable_demo.rs",
                """
                use fret_app::{App, CommandId, Effect, Model, WindowRequest};

                struct DemoWindowState {
                    table_output: Model<shadcn::DataTableViewOutput>,
                }

                fn build_ui(app: &mut App) {
                    let table_output = app.models_mut().insert(shadcn::DataTableViewOutput::default());
                    let _ = table_output;
                }

                fn render(cx: &mut ElementContext<'_, App>, state: &DemoWindowState) {
                    let table_output = state.table_output.clone();
                    cx.observe_model(&table_output, Invalidation::Layout);
                    let _ = table_output.layout_value(cx);
                    shadcn::DataTablePagination::new(&table_state, table_output.clone());
                    shadcn::DataTable::new(rows, columns).output_model(table_output.clone());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/datatable_demo.rs",
                        "advanced_manual",
                        "fixture datatable demo surface",
                        owner="examples-datatable",
                        allowed_raw_seams=(
                            "fret_app",
                            "fret_core",
                            "fret_launch",
                            "fret_runtime",
                            "fret_ui",
                            "AnyElement",
                            "ElementContext",
                            "FnDriver",
                            "UiTree",
                        ),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-datatable-output-boundary"
            ]
            self.assertEqual(5, len(owner_violations))
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("legacy-output-model", messages)
            self.assertIn("legacy-output-model-insert", messages)
            self.assertIn("legacy-observe-model", messages)
            self.assertIn("legacy-model-import", messages)

    def test_datatable_local_state_output_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/datatable_demo.rs",
                """
                use fret::app::AppLocalStateExt as _;
                use fret::app::LocalState;

                struct DemoWindowState {
                    table_output: LocalState<shadcn::DataTableViewOutput>,
                }

                fn build_ui(app: &mut App) {
                    let table_output = app.local_state(shadcn::DataTableViewOutput::default());
                    let _ = table_output;
                }

                fn render(cx: &mut ElementContext<'_, App>, state: &DemoWindowState) {
                    let table_output = state.table_output.clone();
                    let _ = table_output.layout_value(cx);
                    shadcn::DataTablePagination::new(&table_state, table_output.clone());
                    shadcn::DataTable::new(rows, columns).output_model(table_output.clone());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/datatable_demo.rs",
                        "advanced_manual",
                        "fixture datatable demo surface",
                        owner="examples-datatable",
                        allowed_raw_seams=(
                            "fret_app",
                            "fret_core",
                            "fret_launch",
                            "fret_runtime",
                            "fret_ui",
                            "AnyElement",
                            "ElementContext",
                            "FnDriver",
                            "UiTree",
                        ),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "advanced-surface-datatable-output-boundary"
                ]
            )

    def test_editor_notes_direct_model_writes_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/editor_notes_demo.rs",
                """
                use fret_runtime::ModelStore;

                struct EditorNotesModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> EditorNotesModelOwner<'a> {
                    fn new(models: &'a mut ModelStore) -> Self {
                        Self { models }
                    }

                    fn set_text(&mut self, model: &Model<String>, value: impl Into<String>) -> bool {
                        false
                    }
                }

                struct EditorAssetModels {
                    name: Model<String>,
                    notes: Model<String>,
                    notes_outcome: Model<String>,
                    summary_status: Model<String>,
                }

                impl EditorAssetModels {
                    fn new(models: &mut ModelStore, title: &str, name: &str, notes: &str) -> Self {
                        Self {
                            name: models.insert(name.to_string()),
                            notes: models.insert(notes.to_string()),
                            notes_outcome: models.insert("Idle".to_string()),
                            summary_status: models.insert(format!("Ready to copy summary for {title}.")),
                        }
                    }

                    fn set_notes_outcome(&self, models: &mut ModelStore, value: impl Into<String>) -> bool {
                        EditorNotesModelOwner::new(models).set_text(&self.notes_outcome, value)
                    }

                    fn set_summary_status(&self, models: &mut ModelStore, value: impl Into<String>) -> bool {
                        EditorNotesModelOwner::new(models).set_text(&self.summary_status, value)
                    }
                }

                struct EditorThemePresetBinding {
                    preset: Model<EditorThemePresetV1>,
                }

                impl EditorThemePresetBinding {
                    fn picker_model(&self) -> Model<EditorThemePresetV1> {
                        self.preset.clone()
                    }
                }

                struct EditorAssetState {
                    models: EditorAssetModels,
                }

                struct EditorNotesDemoView {
                    theme: EditorThemePresetBinding,
                }

                fn editor_asset_paint_snapshot(cx: &mut AppUi<'_, '_>, asset: &EditorAssetState) {}

                fn init(app: &mut App) {
                    let theme = EditorThemePresetBinding::new(app);
                    let _ = theme;
                }

                fn render(cx: &mut AppUi<'_, '_>, asset: EditorAssetState, theme: EditorThemePresetBinding) {
                    editor_asset_paint_snapshot(cx, &asset);
                    EditorThemePresetPicker::new(theme.picker_model());
                    let models = asset.models.clone();
                    models.set_notes_outcome(host.models_mut(), next);
                    models.set_notes_outcome(host.models_mut(), "Committed");
                    models.set_notes_outcome(host.models_mut(), "Canceled");
                    models.set_summary_status(host.models_mut(), draft_commit_status.clone());
                    models.set_summary_status(host.models_mut(), draft_discard_status.clone());
                    models.set_summary_status(host.models_mut(), summary_status_next.clone());
                    let _ = app.models_mut().update(&models.notes_outcome, |_| true);
                    let _ = ModelStore::update(app.models_mut(), &models.summary_status, |_| true);
                    let _ = asset.notes_outcome_model.clone();
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/editor_notes_demo.rs",
                        "advanced_manual",
                        "fixture editor notes surface",
                        owner="examples-editor-notes",
                        allowed_raw_seams=(
                            "fret_app",
                            "fret_core",
                            "fret_runtime",
                            "fret_ui",
                            "AnyElement",
                            "ElementContext",
                            "ModelStore",
                        ),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-editor-notes-bindings-boundary"
            ]
            self.assertEqual(3, len(owner_violations))
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("models_mut().update", messages)
            self.assertIn("ModelStore::update", messages)
            self.assertIn("legacy-model-field", messages)

    def test_editor_notes_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/editor_notes_demo.rs",
                """
                use fret_runtime::ModelStore;

                struct EditorNotesModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> EditorNotesModelOwner<'a> {
                    fn new(models: &'a mut ModelStore) -> Self {
                        Self { models }
                    }

                    fn set_text(&mut self, model: &Model<String>, value: impl Into<String>) -> bool {
                        false
                    }
                }

                struct EditorAssetModels {
                    name: Model<String>,
                    notes: Model<String>,
                    notes_outcome: Model<String>,
                    summary_status: Model<String>,
                }

                impl EditorAssetModels {
                    fn new(models: &mut ModelStore, title: &str, name: &str, notes: &str) -> Self {
                        Self {
                            name: models.insert(name.to_string()),
                            notes: models.insert(notes.to_string()),
                            notes_outcome: models.insert("Idle".to_string()),
                            summary_status: models.insert(format!("Ready to copy summary for {title}.")),
                        }
                    }

                    fn set_notes_outcome(&self, models: &mut ModelStore, value: impl Into<String>) -> bool {
                        EditorNotesModelOwner::new(models).set_text(&self.notes_outcome, value)
                    }

                    fn set_summary_status(&self, models: &mut ModelStore, value: impl Into<String>) -> bool {
                        EditorNotesModelOwner::new(models).set_text(&self.summary_status, value)
                    }
                }

                struct EditorThemePresetBinding {
                    preset: Model<EditorThemePresetV1>,
                }

                impl EditorThemePresetBinding {
                    fn picker_model(&self) -> Model<EditorThemePresetV1> {
                        self.preset.clone()
                    }
                }

                struct EditorAssetState {
                    models: EditorAssetModels,
                }

                struct EditorNotesDemoView {
                    theme: EditorThemePresetBinding,
                }

                fn editor_asset_paint_snapshot(cx: &mut AppUi<'_, '_>, asset: &EditorAssetState) {}

                fn init(app: &mut App) {
                    let theme = EditorThemePresetBinding::new(app);
                    let _ = theme;
                }

                fn render(cx: &mut AppUi<'_, '_>, asset: EditorAssetState, theme: EditorThemePresetBinding) {
                    editor_asset_paint_snapshot(cx, &asset);
                    EditorThemePresetPicker::new(theme.picker_model());
                    let models = asset.models.clone();
                    models.set_notes_outcome(host.models_mut(), next);
                    models.set_notes_outcome(host.models_mut(), "Committed");
                    models.set_notes_outcome(host.models_mut(), "Canceled");
                    models.set_summary_status(host.models_mut(), draft_commit_status.clone());
                    models.set_summary_status(host.models_mut(), draft_discard_status.clone());
                    models.set_summary_status(host.models_mut(), summary_status_next.clone());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/editor_notes_demo.rs",
                        "advanced_manual",
                        "fixture editor notes surface",
                        owner="examples-editor-notes",
                        allowed_raw_seams=(
                            "fret_app",
                            "fret_core",
                            "fret_runtime",
                            "fret_ui",
                            "AnyElement",
                            "ElementContext",
                            "ModelStore",
                        ),
                        retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "advanced-surface-editor-notes-bindings-boundary"
                ]
            )

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
            "apps/fret-examples/src/alpha_mode_demo.rs",
            "apps/fret-examples/src/assets_demo.rs",
            "apps/fret-examples/src/drop_shadow_demo.rs",
            "apps/fret-examples/src/effects_demo.rs",
            "apps/fret-examples/src/image_upload_demo.rs",
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
