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
        self.assertIn(
            "apps/fret-examples/src/embedded_viewport_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
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
        for path in custom_effect_v2_web_paths:
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
