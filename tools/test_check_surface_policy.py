#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import inspect
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


EDITOR_NOTES_APP_FACING_FIXTURE = """
use fret::app::editor::{
    EditorThemePresetPickerLocalStateExt as _, InspectorTextFieldBinding,
    InspectorTextFieldSnapshot, TextFieldLocalStateExt as _,
};
use fret::app::LocalState;

struct EditorAssetState {
    name: LocalState<String>,
    notes: InspectorTextFieldBinding,
}

struct EditorNotesDemoView {
    theme: LocalState<EditorThemePresetV1>,
}

fn make_asset(app: &mut App, name: &str, notes: &str) -> EditorAssetState {
    EditorAssetState {
        name: app.local_state(name.to_string()),
        notes: InspectorTextFieldBinding::new(app, notes, "Ready")
            .outcome_statuses("Committed", "Canceled"),
    }
}

fn editor_asset_paint_snapshot(
    cx: &mut AppUi<'_, '_>,
    asset: &EditorAssetState,
) -> InspectorTextFieldSnapshot {
    let _ = asset.name.paint_value(cx);
    asset.notes.paint_snapshot(cx)
}

fn render(
    cx: &mut AppUi<'_, '_>,
    asset: EditorAssetState,
    theme: LocalState<EditorThemePresetV1>,
) {
    let notes_snapshot = editor_asset_paint_snapshot(cx, &asset);
    let committed_label = "1 line committed".to_string();
    let _ = notes_snapshot.draft_status_label(&committed_label);
    let _ = asset.name.editor_text_field();
    let _ = asset.notes.text_field(TextFieldOptions { ..Default::default() });
    let _ = asset.notes.commit_activate();
    let _ = asset.notes.discard_activate();
    let _ = asset.notes.status_activate("Copied");
    let _ = theme.editor_theme_preset_picker();
}
"""


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

    def test_gpui_real_app_probes_have_plan_tracked_retirements(self) -> None:
        specs = {spec.path: spec for spec in POLICY.ADVANCED_MANUAL_SURFACES}
        probe_paths = {
            "apps/fret-examples/src/workspace_shell_demo",
        }

        for path in probe_paths:
            spec = specs[path]
            self.assertIn("Temporary real-app probe allowance", spec.retirement)
            self.assertIn(POLICY.GPUI_ERGONOMICS_BOUNDARY_PLAN, spec.retirement)
            self.assertTrue(spec.owner)
            self.assertTrue(spec.allowed_raw_seams)

        workspace_spec = specs["apps/fret-examples/src/workspace_shell_demo"]
        self.assertEqual(POLICY.WORKSPACE_SHELL_OWNER, workspace_spec.owner)
        self.assertNotIn("FnDriver", workspace_spec.allowed_raw_seams)
        self.assertIn("UiTree", workspace_spec.allowed_raw_seams)
        self.assertIn("WorkspaceApp", workspace_spec.retirement)
        self.assertIn("typed workspace commands", workspace_spec.retirement)

        default_specs = {spec.path: spec for spec in POLICY.DEFAULT_AUTHORING_SURFACES}
        datatable_spec = default_specs["apps/fret-examples/src/datatable_demo.rs"]
        self.assertEqual(POLICY.DATATABLE_OWNER, datatable_spec.owner)
        self.assertFalse(datatable_spec.allowed_raw_seams)
        self.assertFalse(datatable_spec.retirement)
        self.assertNotIn("apps/fret-examples/src/datatable_demo.rs", specs)

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

    def test_default_plot_overlay_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/plot_image_demo.rs",
                """
                use fret::app::prelude::*;
                use fret::app::{RenderContextAccess as _, ui_assets};
                use fret::advanced::raw::Model;
                use fret_plot::LinePlotPanelBinding;
                use fret_plot::declarative::{LinePlotPanelProps, line_plot_panel_in};
                use fret_plot::models::{LinePlotModel, LineSeries, YAxis};
                use fret_plot::plot::axis::{AxisLabelFormatter, AxisNumberFormat};
                use fret_plot::retained;
                use fret_plot::series::Series;
                use fret_plot::state::{PlotImage, PlotImageLayer};
                use fret_runtime::Model;

                mod driver;
                pub use driver::{build_app, build_fn_driver, build_runner_config, run};

                struct PlotImageDemoView {
                    plot: LinePlotPanelBinding,
                    model: Model<LinePlotModel>,
                    plot_state: Model<PlotState>,
                    plot_output: Model<PlotOutput>,
                    image: Option<ui_assets::ImageId>,
                    image_size: (u32, u32),
                    image_bytes: Vec<u8>,
                }

                impl View for PlotImageDemoView {
                    fn init(app: &mut App, _window: WindowId) -> Self {
                        let model = LinePlotModel::from_series(vec![LineSeries::new(
                            "signal",
                            Series::from_points_sorted(points, true),
                        )]);
                        Self {
                            plot: LinePlotPanelBinding::new(app, model),
                            model: app.models_mut().insert(LinePlotModel::from_series(vec![])),
                            plot_state: app.models_mut().insert(PlotState::default()),
                            plot_output: app.models_mut().insert(PlotOutput::default()),
                            image: None,
                            image_size: (1, 1),
                            image_bytes: vec![],
                        }
                    }

                    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
                        let (_key, image, _status) = ui_assets::rgba8_image_state(
                            cx,
                            self.image_size.0,
                            self.image_size.1,
                            self.image_bytes.as_slice(),
                            ui_assets::ImageColorSpace::Srgb,
                        );
                        let _ = cx.app_mut().models_mut().update(&self.plot_state, |_| {});
                        let _ = ImageAssetCacheHostExt::default();
                        let _ = ImageAssetKey;
                        with_image_asset_cache();
                        use_image_asset();
                        let _ = fret_core::ImageColorSpace::Srgb;
                        let _ = fret_plot::retained::legacy();
                        let _ = retained::legacy();
                        let _ = LinePlotCanvas;
                        let _ = PlotCanvas;
                        create_node_retained();
                        let _props = LinePlotPanelProps::new(self.model.clone())
                            .state(self.plot_state.clone())
                            .output(self.plot_output.clone());
                        self.image = image;
                        let _ = self.plot.update_state(cx.app_mut(), |state| {
                            state.overlays.images.push(
                                PlotImage::new(image.unwrap(), rect, YAxis::Left)
                                    .layer(PlotImageLayer::BelowGrid),
                            );
                        });
                        let props = self
                            .plot
                            .panel_props()
                            .y_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)));
                        line_plot_panel_in(cx, props).into()
                    }
                }
                """,
            )
            write(
                root / "apps/fret-examples/src/plot_image_demo/driver.rs",
                """
                use anyhow::Context as _;
                use fret::app::prelude::*;
                use fret_launch::{FnDriver, WinitRunnerConfig};
                use fret_runtime::PlatformCapabilities;
                use fret_ui::UiTree;

                use super::PlotImageDemoView;

                pub fn build_app() -> fret::app::App {
                    crate::build_default_view_demo_app()
                }

                pub fn build_runner_config() -> fret_launch::WinitRunnerConfig {
                    crate::build_default_view_demo_runner_config("fret-demo plot_image_demo", (960.0, 640.0))
                }

                pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
                    crate::build_default_view_demo_fn_driver::<PlotImageDemoView>("plot-image-demo");
                    FnDriver::new()
                }

                fn render(_cx: WinitRenderContext<'_, PlotImageDemoView>) {}

                pub fn run() -> anyhow::Result<()> {
                    FretApp::new("plot-image-demo")
                        .window("plot_image_demo", (960.0, 640.0))
                        .view::<PlotImageDemoView>()?
                        .run()
                        .context("run plot_image_demo app")
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/plot_image_demo.rs",
                        "default_app_clean",
                        "fixture default plot image demo",
                    )
                ],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/plot_image_demo/driver.rs",
                        "internal_harness",
                        "fixture plot image driver",
                        owner="examples-plot-image-driver",
                        allowed_raw_seams=("fret_launch",),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            boundary_violations = [
                violation
                for violation in violations
                if violation.rule
                in {
                    "default-app-plot-overlay-binding-boundary",
                    "internal_harness-default-view-plot-driver-boundary",
                }
            ]
            self.assertGreaterEqual(len(boundary_violations), 8)
            messages = "\n".join(violation.message for violation in boundary_violations)
            self.assertIn("LinePlotPanelProps", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("FnDriver", messages)
            self.assertIn("WinitRenderContext", messages)

    def test_default_plot_overlay_binding_and_driver_surfaces_are_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/plot_declarative_demo.rs",
                """
                use fret::app::prelude::*;
                use fret_plot::LinePlotPanelBinding;
                use fret_plot::cartesian::AxisScale;
                use fret_plot::declarative::line_plot_panel_in;
                use fret_plot::models::{LinePlotModel, LineSeries};
                use fret_plot::series::Series;

                struct PlotDeclarativeView {
                    plot: LinePlotPanelBinding,
                }

                pub fn run() -> anyhow::Result<()> {
                    FretApp::new("plot-declarative-demo")
                        .window("plot_declarative_demo", (960.0, 640.0))
                        .view::<PlotDeclarativeView>()?
                        .run()
                }

                impl View for PlotDeclarativeView {
                    fn init(app: &mut App, _window: WindowId) -> Self {
                        let model = LinePlotModel::from_series(vec![LineSeries::new(
                            "signal",
                            Series::from_points_sorted(points, true),
                        )]);
                        Self {
                            plot: LinePlotPanelBinding::new(app, model),
                        }
                    }

                    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
                        let props = self.plot.panel_props().x_scale(AxisScale::Linear);
                        line_plot_panel_in(cx, props).into()
                    }
                }
                """,
            )
            write(
                root / "apps/fret-examples/src/tags_demo.rs",
                """
                use fret::app::prelude::*;
                use fret_plot::LinePlotPanelBinding;
                use fret_plot::declarative::line_plot_panel_in;
                use fret_plot::models::{LinePlotModel, LineSeries};
                use fret_plot::series::Series;
                use fret_plot::state::{PlotOverlays, PlotState};

                mod driver;
                pub use driver::{build_app, build_fn_driver, build_runner_config, run};

                struct TagsDemoView {
                    plot: LinePlotPanelBinding,
                }

                impl View for TagsDemoView {
                    fn init(app: &mut App, _window: WindowId) -> Self {
                        let model = LinePlotModel::from_series(vec![LineSeries::new(
                            "signal",
                            Series::from_points_sorted(points, true),
                        )]);
                        let mut state = PlotState::default();
                        state.overlays = PlotOverlays {
                            tags_x: vec![fret_plot::state::TagX::new(25.0).label("T1")],
                            tags_y: vec![fret_plot::state::TagY::new(0.5, fret_plot::models::YAxis::Left).label("limit")],
                            text: vec![fret_plot::state::PlotText::new(50.0, -0.75, fret_plot::models::YAxis::Left, "PlotText at (50, -0.75)")],
                        };
                        Self {
                            plot: LinePlotPanelBinding::new_with_state(app, model, state),
                        }
                    }

                    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
                        let props = self.plot.panel_props();
                        line_plot_panel_in(cx, props).into()
                    }
                }
                """,
            )
            write(
                root / "apps/fret-examples/src/tags_demo/driver.rs",
                """
                use anyhow::Context as _;
                use fret::app::prelude::*;

                use super::TagsDemoView;

                pub fn build_app() -> fret::app::App {
                    crate::build_default_view_demo_app()
                }

                pub fn build_runner_config() -> fret_launch::WinitRunnerConfig {
                    crate::build_default_view_demo_runner_config("fret-demo tags_demo", (960.0, 640.0))
                }

                pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
                    crate::build_default_view_demo_fn_driver::<TagsDemoView>("tags-demo")
                }

                pub fn run() -> anyhow::Result<()> {
                    FretApp::new("tags-demo")
                        .window("tags_demo", (960.0, 640.0))
                        .view::<TagsDemoView>()?
                        .run()
                        .context("run tags_demo app")
                }
                """,
            )
            write(
                root / "apps/fret-examples/src/plot_image_demo.rs",
                """
                use fret::app::prelude::*;
                use fret::app::{RenderContextAccess as _, ui_assets};
                use fret_plot::LinePlotPanelBinding;
                use fret_plot::cartesian::{AxisScale, DataRect};
                use fret_plot::declarative::line_plot_panel_in;
                use fret_plot::models::{LinePlotModel, LineSeries, YAxis};
                use fret_plot::plot::axis::{AxisLabelFormatter, AxisNumberFormat};
                use fret_plot::series::Series;
                use fret_plot::state::{PlotImage, PlotImageLayer};

                mod driver;
                pub use driver::{build_app, build_fn_driver, build_runner_config, run};

                struct PlotImageDemoView {
                    plot: LinePlotPanelBinding,
                    image: Option<ui_assets::ImageId>,
                    image_size: (u32, u32),
                    image_bytes: Vec<u8>,
                }

                impl View for PlotImageDemoView {
                    fn init(app: &mut App, _window: WindowId) -> Self {
                        let model = LinePlotModel::from_series(vec![LineSeries::new(
                            "signal",
                            Series::from_points_sorted(points, true),
                        )]);
                        Self {
                            plot: LinePlotPanelBinding::new(app, model),
                            image: None,
                            image_size: (1, 1),
                            image_bytes: vec![],
                        }
                    }

                    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
                        let (_key, image, _status) = ui_assets::rgba8_image_state(
                            cx,
                            self.image_size.0,
                            self.image_size.1,
                            self.image_bytes.as_slice(),
                            ui_assets::ImageColorSpace::Srgb,
                        );
                        let _ = self.plot.update_state(cx.app_mut(), |state| {
                            state.overlays.images.push(
                                PlotImage::new(
                                    image.unwrap(),
                                    DataRect { x_min: 0.0, x_max: 1.0, y_min: 0.0, y_max: 1.0 },
                                    YAxis::Left,
                                )
                                .layer(PlotImageLayer::BelowGrid),
                            );
                        });
                        let props = self
                            .plot
                            .panel_props()
                            .y_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)))
                            .x_scale(AxisScale::Linear)
                            .y_scale(AxisScale::Linear);
                        line_plot_panel_in(cx, props).into()
                    }
                }
                """,
            )
            write(
                root / "apps/fret-examples/src/plot_image_demo/driver.rs",
                """
                use anyhow::Context as _;
                use fret::app::prelude::*;

                use super::PlotImageDemoView;

                pub fn build_app() -> fret::app::App {
                    crate::build_default_view_demo_app()
                }

                pub fn build_runner_config() -> fret_launch::WinitRunnerConfig {
                    crate::build_default_view_demo_runner_config("fret-demo plot_image_demo", (960.0, 640.0))
                }

                pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
                    crate::build_default_view_demo_fn_driver::<PlotImageDemoView>("plot-image-demo")
                }

                pub fn run() -> anyhow::Result<()> {
                    FretApp::new("plot-image-demo")
                        .window("plot_image_demo", (960.0, 640.0))
                        .view::<PlotImageDemoView>()?
                        .run()
                        .context("run plot_image_demo app")
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/plot_declarative_demo.rs",
                        "default_app_clean",
                        "fixture default plot demo",
                    ),
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/tags_demo.rs",
                        "default_app_clean",
                        "fixture default tags demo",
                    ),
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/plot_image_demo.rs",
                        "default_app_clean",
                        "fixture default plot image demo",
                    ),
                ],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/tags_demo/driver.rs",
                        "internal_harness",
                        "fixture tags demo driver",
                        owner="examples-plot-tags-driver",
                        allowed_raw_seams=("fret_launch",),
                    ),
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/plot_image_demo/driver.rs",
                        "internal_harness",
                        "fixture plot image driver",
                        owner="examples-plot-image-driver",
                        allowed_raw_seams=("fret_launch",),
                    ),
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule
                    in {
                        "default-app-plot-overlay-binding-boundary",
                        "internal_harness-default-view-plot-driver-boundary",
                    }
                ]
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
            "apps/fret-examples/src/plot_declarative_demo.rs",
            POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS,
        )
        shadcn_rich_demo_paths = {
            "apps/fret-examples/src/date_picker_demo.rs",
            "apps/fret-examples/src/form_demo.rs",
            "apps/fret-examples/src/table_demo.rs",
            "apps/fret-examples/src/sonner_demo.rs",
        }
        for path in shadcn_rich_demo_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        default_plot_overlay_paths = {
            "apps/fret-examples/src/plot_declarative_demo.rs",
            "apps/fret-examples/src/plot_image_demo.rs",
            "apps/fret-examples/src/tags_demo.rs",
        }
        for path in default_plot_overlay_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
        tail_advanced_paths = {
            "apps/fret-examples/src/drag_demo.rs",
            "apps/fret-examples/src/markdown_demo.rs",
            "apps/fret-examples/src/genui_demo.rs",
            "apps/fret-examples/src/imui_editor_proof_demo.rs",
            "apps/fret-examples/src/imui_node_graph_demo.rs",
        }
        for path in tail_advanced_paths:
            self.assertIn(path, POLICY.PUBLIC_EXAMPLE_SCAN_ROOTS)
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
            "apps/fret-examples/src/custom_effect_v1_demo.rs": (
                "examples-custom-effect-v1-native",
                "EffectParamsV1",
            ),
            "apps/fret-examples/src/custom_effect_v2_demo.rs": (
                "examples-custom-effect-v2-native",
                "user-image sampling",
            ),
            "apps/fret-examples/src/custom_effect_v3_demo.rs": (
                "examples-custom-effect-v3-native",
                "diagnostic source binding",
            ),
            "apps/fret-examples/src/custom_effect_v3_web_demo.rs": (
                "examples-custom-effect-v3-web",
                "manual web runner/bootstrap",
            ),
        }
        streaming_import_paths = {
            "apps/fret-examples/src/streaming_i420_demo.rs": (
                "examples-streaming-i420",
                "ImageUpdateI420",
            ),
            "apps/fret-examples/src/streaming_image_demo.rs": (
                "examples-streaming-image",
                "ImageUpdateRgba8",
            ),
            "apps/fret-examples/src/streaming_nv12_demo.rs": (
                "examples-streaming-nv12",
                "ImageUpdateNv12",
            ),
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
        table_stress_spec = next(
            spec
            for spec in POLICY.INTERNAL_HARNESS_SURFACES
            if spec.path == "apps/fret-examples/src/table_stress_demo.rs"
        )
        self.assertNotIn("AnyElement", table_stress_spec.allowed_raw_seams)
        self.assertIn("ElementContext", table_stress_spec.allowed_raw_seams)
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
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/async_playground_demo.rs"
                for spec in POLICY.DEFAULT_AUTHORING_SURFACES
            ),
            "async_playground_demo should be classified as default-clean app authoring",
        )
        self.assertFalse(
            any(
                spec.path == "apps/fret-examples/src/async_playground_demo.rs"
                for spec in POLICY.ADVANCED_MANUAL_SURFACES
            ),
            "async_playground_demo should not require advanced/manual quarantine",
        )
        for path in shadcn_rich_demo_paths:
            spec = next(
                (
                    spec
                    for spec in POLICY.ADVANCED_MANUAL_SURFACES
                    if spec.path == path
                ),
                None,
            )
            self.assertIsNotNone(
                spec, f"{path} should be classified as an advanced shadcn behavior proof"
            )
            self.assertIn("shadcn", spec.reason)
            self.assertTrue(spec.allowed_raw_seams)
            self.assertTrue(spec.retirement)
        for path in tail_advanced_paths:
            spec = next(
                (
                    spec
                    for spec in POLICY.ADVANCED_MANUAL_SURFACES
                    if spec.path == path
                ),
                None,
            )
            self.assertIsNotNone(
                spec, f"{path} should be classified as an advanced/manual tail surface"
            )
            self.assertTrue(spec.allowed_raw_seams)
            self.assertTrue(spec.retirement)
        markdown_spec = next(
            spec
            for spec in POLICY.ADVANCED_MANUAL_SURFACES
            if spec.path == "apps/fret-examples/src/markdown_demo.rs"
        )
        self.assertIn("image/SVG", markdown_spec.reason)
        self.assertNotIn("AnyElement", markdown_spec.allowed_raw_seams)
        self.assertNotIn("ElementContext", markdown_spec.allowed_raw_seams)
        postprocess_spec = next(
            spec
            for spec in POLICY.ADVANCED_MANUAL_SURFACES
            if spec.path == "apps/fret-examples/src/postprocess_theme_demo.rs"
        )
        self.assertIn("postprocess", postprocess_spec.reason)
        self.assertNotIn("AnyElement", postprocess_spec.allowed_raw_seams)
        editor_device_shell_spec = next(
            spec
            for spec in POLICY.ADVANCED_MANUAL_SURFACES
            if spec.path == "apps/fret-examples/src/editor_notes_device_shell_demo.rs"
        )
        self.assertIn("device-shell", editor_device_shell_spec.reason)
        self.assertNotIn("AnyElement", editor_device_shell_spec.allowed_raw_seams)
        self.assertNotIn("ElementContext", editor_device_shell_spec.allowed_raw_seams)
        date_picker_spec = next(
            spec
            for spec in POLICY.ADVANCED_MANUAL_SURFACES
            if spec.path == "apps/fret-examples/src/date_picker_demo.rs"
        )
        self.assertIn("date-picker", date_picker_spec.reason)
        self.assertNotIn("AnyElement", date_picker_spec.allowed_raw_seams)
        self.assertNotIn("ElementContext", date_picker_spec.allowed_raw_seams)
        imui_node_graph_spec = next(
            spec
            for spec in POLICY.ADVANCED_MANUAL_SURFACES
            if spec.path == "apps/fret-examples/src/imui_node_graph_demo.rs"
        )
        self.assertIn("node-graph", imui_node_graph_spec.reason)
        self.assertIn("fret_runtime", imui_node_graph_spec.allowed_raw_seams)
        self.assertNotIn("fret_ui", imui_node_graph_spec.allowed_raw_seams)
        self.assertNotIn("AnyElement", imui_node_graph_spec.allowed_raw_seams)
        self.assertNotIn("ElementContext", imui_node_graph_spec.allowed_raw_seams)
        for path in default_plot_overlay_paths:
            self.assertTrue(
                any(spec.path == path for spec in POLICY.DEFAULT_AUTHORING_SURFACES),
                f"{path} should be back on the default app authoring surface",
            )
            self.assertFalse(
                any(spec.path == path for spec in POLICY.ADVANCED_MANUAL_SURFACES),
                f"{path} should not require advanced/manual quarantine",
            )
        for path in {
            "apps/fret-examples/src/plot_image_demo/driver.rs",
            "apps/fret-examples/src/tags_demo/driver.rs",
        }:
            spec = next(
                (
                    spec
                    for spec in POLICY.INTERNAL_HARNESS_SURFACES
                    if spec.path == path
                ),
                None,
            )
            self.assertIsNotNone(
                spec, f"{path} should own the launch seam as an internal harness"
            )
            self.assertEqual(spec.allowed_raw_seams, ("fret_launch",))
        for path in editor_notes_paths:
            self.assertTrue(
                any(spec.path == path for spec in POLICY.ADVANCED_MANUAL_SURFACES),
                f"{path} should be classified as an advanced editor notes surface",
            )
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/datatable_demo.rs"
                for spec in POLICY.DEFAULT_AUTHORING_SURFACES
            )
        )
        datatable_spec = next(
            spec
            for spec in POLICY.DEFAULT_AUTHORING_SURFACES
            if spec.path == "apps/fret-examples/src/datatable_demo.rs"
        )
        self.assertEqual(POLICY.DATATABLE_OWNER, datatable_spec.owner)
        self.assertFalse(datatable_spec.allowed_raw_seams)
        table_demo_spec = next(
            spec
            for spec in POLICY.ADVANCED_MANUAL_SURFACES
            if spec.path == "apps/fret-examples/src/table_demo.rs"
        )
        self.assertIn("table", table_demo_spec.reason)
        self.assertNotIn("AnyElement", table_demo_spec.allowed_raw_seams)
        self.assertNotIn("ElementContext", table_demo_spec.allowed_raw_seams)
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
        for path, (owner, reason_token) in custom_effect_reference_paths.items():
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
            self.assertEqual(owner, spec.owner)
            self.assertIn("bounded custom-effect contract", spec.reason)
            self.assertIn(reason_token, spec.reason)
            self.assertTrue(spec.allowed_raw_seams)
        for path, (owner, effect_name) in streaming_import_paths.items():
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
            self.assertEqual(owner, spec.owner)
            self.assertIn(effect_name, spec.reason)
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
        simple_todo_driver_spec = next(
            (
                spec
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
                if spec.path == "apps/fret-examples/src/simple_todo_demo/driver.rs"
            ),
            None,
        )
        self.assertIsNotNone(
            simple_todo_driver_spec,
            "simple todo driver should stay classified as an internal harness",
        )
        self.assertEqual(simple_todo_driver_spec.allowed_raw_seams, ("fret_launch",))
        todo_runtime_tests_spec = next(
            (
                spec
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
                if spec.path == "apps/fret-examples/src/todo_demo_runtime_tests.rs"
            ),
            None,
        )
        self.assertIsNotNone(
            todo_runtime_tests_spec,
            "todo demo runtime tests should own the raw view-runtime harness",
        )
        self.assertIn("fret::advanced", todo_runtime_tests_spec.allowed_raw_seams)
        self.assertIn("fret_runtime", todo_runtime_tests_spec.allowed_raw_seams)
        self.assertIn("UiTree", todo_runtime_tests_spec.allowed_raw_seams)
        self.assertTrue(
            any(
                spec.path == "apps/fret-examples/src/todo_demo.rs"
                for spec in POLICY.DEFAULT_AUTHORING_SURFACES
            ),
            "todo demo should be back on the default app authoring surface",
        )
        self.assertFalse(
            any(
                spec.path == "apps/fret-examples/src/todo_demo.rs"
                for spec in POLICY.ADVANCED_MANUAL_SURFACES
            ),
            "todo demo app source should no longer need advanced/manual quarantine",
        )
        harness_root_spec = next(
            (
                spec
                for spec in POLICY.INTERNAL_HARNESS_SURFACES
                if spec.path == "apps/fret-examples/src/lib.rs"
            ),
            None,
        )
        self.assertIsNotNone(
            harness_root_spec,
            "examples crate root should be classified as an internal harness",
        )
        self.assertIn("fret_runtime", harness_root_spec.allowed_raw_seams)
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
        for path in [
            *query_paths,
            "apps/fret-examples/src/node_graph_demo.rs",
        ]:
            self.assertTrue(
                any(spec.path == path for spec in POLICY.DEFAULT_AUTHORING_SURFACES),
                f"{path} should stay classified as default-clean app authoring",
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

    def test_manual_chart_demo_classification_requires_explicit_owner(self) -> None:
        with self.assertRaisesRegex(
            AssertionError,
            "manual chart/plot surface owner",
        ):
            POLICY._fret_examples_manual_chart_surface("future_chart_demo.rs")

    def test_fret_examples_classified_helpers_require_explicit_owners(self) -> None:
        for helper in (
            POLICY._fret_examples_advanced_surface,
            POLICY._fret_examples_comparison_surface,
            POLICY._fret_examples_internal_harness,
            POLICY._fret_examples_renderer_lab_surface,
        ):
            owner_parameter = inspect.signature(helper).parameters["owner"]
            self.assertEqual(inspect.Parameter.KEYWORD_ONLY, owner_parameter.kind)
            self.assertIs(inspect.Parameter.empty, owner_parameter.default)

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

    def test_plot3d_raw_model_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/plot3d_demo.rs",
                """
                use fret_plot3d::{Plot3dPanelBinding, Plot3dStyle, Plot3dViewport, plot3d_panel};
                use fret_plot3d::{Plot3dModel, Plot3dPanelProps, Plot3dStyle, Plot3dViewport, plot3d_panel};
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct Plot3dDemoWindowState {
                    ui: UiTree<App>,
                    plot: Plot3dPanelBinding,
                    target: ViewportRenderTarget,
                }

                impl Plot3dDemoDriver {
                    fn build_ui(app: &mut App) -> Plot3dDemoWindowState {
                        let plot = Plot3dPanelBinding::new(
                            app,
                            Plot3dViewport {
                                target: RenderTargetId::default(),
                                target_px_size: (960, 540),
                                fit: fret_core::ViewportFit::Contain,
                                opacity: 1.0,
                            },
                        );

                        Plot3dDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                            target: ViewportRenderTarget::new(
                                wgpu::TextureFormat::Bgra8UnormSrgb,
                                RenderTargetColorSpace::Srgb,
                            ),
                        }
                    }

                    fn ensure_target(
                        app: &mut App,
                        state: &mut Plot3dDemoWindowState,
                    ) {
                        let desired_size = state.plot.viewport_untracked(app).target_px_size;
                        let (id, view) = state.target.ensure_size(
                            context,
                            renderer,
                            desired_size,
                            Some("plot3d demo target"),
                        );
                        let new_size = state.target.size();
                        let _ = state.plot.sync_viewport_target(app, id, new_size);
                    }
                }

                fn record_engine_frame() -> EngineFrameUpdate {
                    EngineFrameUpdate {
                        target_updates: Vec::new(),
                        command_buffers: vec![encoder.finish()],
                        keepalive: Vec::new(),
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut Plot3dDemoWindowState,
                ) {
                    declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("plot3d-demo", |cx| {
                            let style = Plot3dStyle::default();
                            vec![plot3d_panel(cx, state.plot.panel_props().style(style))]
                        });
                }

                struct LegacyPlot3dState {
                    plot: fret_runtime::Model<Plot3dModel>,
                }

                fn bad(app: &mut App, state: &mut LegacyPlot3dState) {
                    let plot = app.models_mut().insert(Plot3dModel {
                        viewport: Plot3dViewport::default(),
                    });
                    let _ = state.plot.read(app, |_app, model| model.viewport);
                    let _ = state.plot.update(app, |model, _cx| {
                        model.viewport.opacity = 0.5;
                        true
                    });
                    let _props = Plot3dPanelProps::new(state.plot.clone());
                    let _ = plot3d_panel_with_model(cx, plot.clone());
                    let _ = Plot3dPanelBinding::from_model(plot);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/plot3d_demo.rs",
                        "advanced_manual",
                        "fixture plot3d demo",
                        owner="examples-plot3d",
                        allowed_raw_seams=(
                            "fret_core",
                            "fret_runtime",
                            "fret_ui",
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
                if violation.rule == "advanced-surface-plot3d-panel-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 6)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_runtime::Model<Plot3dModel>", messages)
            self.assertIn("Plot3dPanelProps::new", messages)
            self.assertIn("plot3d_panel_with_model", messages)
            self.assertIn("Plot3dPanelBinding::from_model", messages)

    def test_plot3d_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/plot3d_demo.rs",
                """
                use fret_plot3d::{Plot3dPanelBinding, Plot3dStyle, Plot3dViewport, plot3d_panel};
                use fret_ui::{UiTree, declarative};

                struct Plot3dDemoWindowState {
                    ui: UiTree<App>,
                    plot: Plot3dPanelBinding,
                    target: ViewportRenderTarget,
                }

                impl Plot3dDemoDriver {
                    fn build_ui(app: &mut App) -> Plot3dDemoWindowState {
                        let plot = Plot3dPanelBinding::new(
                            app,
                            Plot3dViewport {
                                target: RenderTargetId::default(),
                                target_px_size: (960, 540),
                                fit: fret_core::ViewportFit::Contain,
                                opacity: 1.0,
                            },
                        );

                        Plot3dDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                            target: ViewportRenderTarget::new(
                                wgpu::TextureFormat::Bgra8UnormSrgb,
                                RenderTargetColorSpace::Srgb,
                            ),
                        }
                    }

                    fn ensure_target(
                        app: &mut App,
                        state: &mut Plot3dDemoWindowState,
                    ) {
                        let desired_size = state.plot.viewport_untracked(app).target_px_size;
                        let (id, view) = state.target.ensure_size(
                            context,
                            renderer,
                            desired_size,
                            Some("plot3d demo target"),
                        );
                        let new_size = state.target.size();
                        let _ = state.plot.sync_viewport_target(app, id, new_size);
                    }
                }

                fn record_engine_frame() -> EngineFrameUpdate {
                    EngineFrameUpdate {
                        target_updates: Vec::new(),
                        command_buffers: vec![encoder.finish()],
                        keepalive: Vec::new(),
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut Plot3dDemoWindowState,
                ) {
                    declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("plot3d-demo", |cx| {
                            let style = Plot3dStyle::default();
                            vec![plot3d_panel(cx, state.plot.panel_props().style(style))]
                        });
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/plot3d_demo.rs",
                        "advanced_manual",
                        "fixture plot3d demo",
                        owner="examples-plot3d",
                        allowed_raw_seams=("fret_core", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "advanced-surface-plot3d-panel-binding-boundary"
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
                use fret::advanced::interop::run_native_with_driver;
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
                    run_native_with_driver(config, KernelApp::new(), driver)?;
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
                use fret::advanced::interop::run_native_with_driver;
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
                    run_native_with_driver(config, KernelApp::new(), driver)?;
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

    def test_hotpatch_smoke_direct_model_writes_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-demo/src/bin/hotpatch_smoke_demo.rs",
                """
                use fret_runtime::{Model, ModelStore};
                use std::sync::Arc;

                struct State {
                    counter: Model<i64>,
                    debug: Model<Arc<str>>,
                }

                struct HotpatchSmokeModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> HotpatchSmokeModelOwner<'a> {
                    fn new(models: &'a mut ModelStore) -> Self {
                        Self { models }
                    }

                    fn increment_counter(&mut self, model: &Model<i64>) -> Option<i64> {
                        self.models.update(model, |value| {
                            *value += 1;
                            *value
                        }).ok()
                    }

                    fn set_debug(&mut self, model: &Model<Arc<str>>, message: &str) -> bool {
                        self.models.update(model, |value| *value = Arc::from(message)).is_ok()
                    }
                }

                fn on_event(app: &mut App, state: &mut State) {
                    let msg = "pointer down";
                    let _ = HotpatchSmokeModelOwner::new(app.models_mut()).set_debug(&state.debug, &msg);
                    let _ = app.models_mut().update(&state.debug, |_| true);
                }

                fn on_command(app: &mut App, state: &mut State) {
                    let msg = "command";
                    let _ = HotpatchSmokeModelOwner::new(app.models_mut())
                        .increment_counter(&state.counter);
                    let _ = HotpatchSmokeModelOwner::new(app.models_mut()).set_debug(&state.debug, &msg);
                    let _ = <ModelStore>::update_any::<u64>(app.models_mut(), &state.debug, |_| true);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-demo/src/bin/hotpatch_smoke_demo.rs",
                        "internal_harness",
                        "fixture hotpatch smoke harness",
                        owner="demo-hotpatch-smoke",
                        allowed_raw_seams=("fret_runtime", "ModelStore"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "internal_harness-hotpatch-smoke-owner-boundary"
            ]
            self.assertEqual(2, len(owner_violations))
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("models_mut().update", messages)
            self.assertIn("ModelStore::update", messages)

    def test_hotpatch_smoke_owner_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-demo/src/bin/hotpatch_smoke_demo.rs",
                """
                use fret_runtime::{Model, ModelStore};
                use std::sync::Arc;

                struct State {
                    counter: Model<i64>,
                    debug: Model<Arc<str>>,
                }

                struct HotpatchSmokeModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> HotpatchSmokeModelOwner<'a> {
                    fn new(models: &'a mut ModelStore) -> Self {
                        Self { models }
                    }

                    fn increment_counter(&mut self, model: &Model<i64>) -> Option<i64> {
                        self.models.update(model, |value| {
                            *value += 1;
                            *value
                        }).ok()
                    }

                    fn set_debug(&mut self, model: &Model<Arc<str>>, message: &str) -> bool {
                        self.models.update(model, |value| *value = Arc::from(message)).is_ok()
                    }
                }

                fn on_event(app: &mut App, state: &mut State) {
                    let msg = "pointer down";
                    let _ = HotpatchSmokeModelOwner::new(app.models_mut()).set_debug(&state.debug, &msg);
                }

                fn on_command(app: &mut App, state: &mut State) {
                    let msg = "command";
                    let _ = HotpatchSmokeModelOwner::new(app.models_mut())
                        .increment_counter(&state.counter);
                    let _ = HotpatchSmokeModelOwner::new(app.models_mut()).set_debug(&state.debug, &msg);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-demo/src/bin/hotpatch_smoke_demo.rs",
                        "internal_harness",
                        "fixture hotpatch smoke harness",
                        owner="demo-hotpatch-smoke",
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
                    if violation.rule == "internal_harness-hotpatch-smoke-owner-boundary"
                ]
            )

    def test_docking_arbitration_direct_model_writes_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/docking_arbitration_demo.rs",
                """
                use fret_runtime::{ModelStore, PlatformCapabilities};

                struct DockingArbitrationControls {}
                struct DockingArbitrationControlsService {}

                impl DockingArbitrationControlsService {
                    fn default() -> Self { Self }
                    fn set(&mut self, window: AppWindowId, controls: DockingArbitrationControls) {}
                }

                impl DockingArbitrationControls {
                    fn new(models: &mut ModelStore) -> Self { Self }
                    fn toggle_drop_mask_disallow_left_edge(&self, host: &mut Host) -> bool { false }
                    fn set_synth_pointer_debug(&self, app: &mut App, msg: Arc<str>) -> bool { true }
                    fn set_last_viewport_input(&self, app: &mut App, msg: Arc<str>) -> bool { true }
                }

                fn build_ui(app: &mut App, window: AppWindowId) {
                    let controls = DockingArbitrationControls::new(app.models_mut());
                    app.with_global_mut(
                        DockingArbitrationControlsService::default,
                        move |svc, _app| {
                            svc.set(window, controls);
                        },
                    );
                }

                fn controls_panel(host: &mut Host, controls: &DockingArbitrationControls) {
                    let next = controls.toggle_drop_mask_disallow_left_edge(host);
                    let _ = next;
                }

                fn update_synth_debug(app: &mut App, controls: DockingArbitrationControls, msg: Arc<str>) {
                    controls.set_synth_pointer_debug(app, msg);
                }

                fn viewport_input(app: &mut App, controls: DockingArbitrationControls, msg: Arc<str>) {
                    controls.set_last_viewport_input(app, msg);
                }

                fn dispatch_synth(synth: Synth, pressed: bool) {
                    if !synth.enabled && pressed { return; }
                }

                fn bad(app: &mut App, state: &State) {
                    let _ = app.models_mut().update(&state.debug, |_| true);
                    let _ = ModelStore::update(app.models_mut(), &state.debug, |_| true);
                    let mut models = app.models_mut();
                    let _ = models.update(&state.debug, |_| true);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/docking_arbitration_demo.rs",
                        "internal_harness",
                        "fixture docking arbitration harness",
                        owner="examples-docking-arbitration",
                        allowed_raw_seams=("fret_runtime", "ModelStore"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "internal_harness-docking-arbitration-controls-boundary"
            ]
            self.assertEqual(3, len(owner_violations))
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("models_mut().update", messages)
            self.assertIn("ModelStore::update", messages)
            self.assertIn("ModelStore alias", messages)

    def test_docking_arbitration_controls_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/docking_arbitration_demo.rs",
                """
                use fret_runtime::{ModelStore, PlatformCapabilities};

                struct DockingArbitrationControls {}
                struct DockingArbitrationControlsService {}

                impl DockingArbitrationControlsService {
                    fn default() -> Self { Self }
                    fn set(&mut self, window: AppWindowId, controls: DockingArbitrationControls) {}
                }

                impl DockingArbitrationControls {
                    fn new(models: &mut ModelStore) -> Self { Self }
                    fn toggle_drop_mask_disallow_left_edge(&self, host: &mut Host) -> bool { false }
                    fn set_synth_pointer_debug(&self, app: &mut App, msg: Arc<str>) -> bool { true }
                    fn set_last_viewport_input(&self, app: &mut App, msg: Arc<str>) -> bool { true }
                }

                fn build_ui(app: &mut App, window: AppWindowId) {
                    let controls = DockingArbitrationControls::new(app.models_mut());
                    app.with_global_mut(
                        DockingArbitrationControlsService::default,
                        move |svc, _app| {
                            svc.set(window, controls);
                        },
                    );
                }

                fn controls_panel(host: &mut Host, controls: &DockingArbitrationControls) {
                    let next = controls.toggle_drop_mask_disallow_left_edge(host);
                    let _ = next;
                }

                fn update_synth_debug(app: &mut App, controls: DockingArbitrationControls, msg: Arc<str>) {
                    controls.set_synth_pointer_debug(app, msg);
                }

                fn viewport_input(app: &mut App, controls: DockingArbitrationControls, msg: Arc<str>) {
                    controls.set_last_viewport_input(app, msg);
                }

                fn dispatch_synth(synth: Synth, pressed: bool) {
                    if !synth.enabled && pressed { return; }
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/docking_arbitration_demo.rs",
                        "internal_harness",
                        "fixture docking arbitration harness",
                        owner="examples-docking-arbitration",
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
                    == "internal_harness-docking-arbitration-controls-boundary"
                ]
            )

    def test_plot_drag_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/drag_demo.rs",
                """
                use fret_plot::LinePlotPanelBinding;
                use fret_plot::declarative::line_plot_panel_in;
                use fret_plot::declarative::LinePlotPanelProps;
                use fret_plot::models::{LinePlotModel, LineSeries};
                use fret_plot::retained;
                use fret_plot::retained::LinePlotCanvas;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                pub struct DragDemoWindowState {
                    ui: UiTree<App>,
                    plot: LinePlotPanelBinding,
                    plot_state: Model<PlotState>,
                }

                impl DragDemoDriver {
                    fn apply_drag(state: &mut PlotState, drag: PlotDragOutput) {
                        match drag {
                            PlotDragOutput::LineX { .. } => {}
                            PlotDragOutput::LineY { .. } => {}
                            PlotDragOutput::Point { .. } => {}
                            PlotDragOutput::Rect { .. } => {}
                        }
                    }

                    fn build_ui(app: &mut App) -> DragDemoWindowState {
                        let model = LinePlotModel::from_series(vec![
                            LineSeries::new("signal", data),
                        ]);
                        let state = PlotState::default();
                        let plot = LinePlotPanelBinding::new_with_state(app, model, state);
                        DragDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                            plot_state: app.models_mut().insert(PlotState::default()),
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut DragDemoWindowState) {
                    let output = state.plot.output_untracked(app);
                    if let Some(drag) = output.snapshot.drag {
                        let _ = state.plot.update_state(app, |s| {
                            DragDemoDriver::apply_drag(s, drag);
                        });
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut DragDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("drag-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![line_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyDragPlot {
                    plot: fret_runtime::Model<LinePlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<LinePlotModel>,
                    plot_state: Model<PlotState>,
                    plot_output: Model<PlotOutput>,
                    state: &mut DragDemoWindowState,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = LinePlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = LinePlotPanelProps::new(plot.clone())
                        .state(plot_state.clone())
                        .output(plot_output.clone());
                    state.plot_state.update(app, |_| {});
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/drag_demo.rs",
                        "advanced_manual",
                        "fixture plot drag demo",
                        owner="examples-plot-drag-demo",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "advanced-surface-plot-drag-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("LinePlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("LinePlotPanelProps", messages)
            self.assertIn("state.plot_state.update", messages)

    def test_plot_drag_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/drag_demo.rs",
                """
                use fret_plot::LinePlotPanelBinding;
                use fret_plot::declarative::line_plot_panel_in;
                use fret_plot::models::{LinePlotModel, LineSeries};
                use fret_ui::{UiTree, declarative};

                pub struct DragDemoWindowState {
                    ui: UiTree<App>,
                    plot: LinePlotPanelBinding,
                }

                impl DragDemoDriver {
                    fn apply_drag(state: &mut PlotState, drag: PlotDragOutput) {
                        match drag {
                            PlotDragOutput::LineX { .. } => {}
                            PlotDragOutput::LineY { .. } => {}
                            PlotDragOutput::Point { .. } => {}
                            PlotDragOutput::Rect { .. } => {}
                        }
                    }

                    fn build_ui(app: &mut App) -> DragDemoWindowState {
                        let model = LinePlotModel::from_series(vec![
                            LineSeries::new("signal", data),
                        ]);
                        let state = PlotState::default();
                        let plot = LinePlotPanelBinding::new_with_state(app, model, state);
                        DragDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut DragDemoWindowState) {
                    let output = state.plot.output_untracked(app);
                    if let Some(drag) = output.snapshot.drag {
                        let _ = state.plot.update_state(app, |s| {
                            DragDemoDriver::apply_drag(s, drag);
                        });
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut DragDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("drag-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![line_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/drag_demo.rs",
                        "advanced_manual",
                        "fixture plot drag demo",
                        owner="examples-plot-drag-demo",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-plot-drag-declarative-binding-boundary"
                ]
            )

    def test_plot_inf_lines_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/inf_lines_demo.rs",
                """
                use fret_plot::LinePlotPanelBinding;
                use fret_plot::declarative::{LinePlotPanelProps, line_plot_panel_in};
                use fret_plot::models::{LinePlotModel, LineSeries, YAxis};
                use fret_plot::retained;
                use fret_plot::state::{InfLineX, InfLineY, PlotOverlays, PlotState};
                use fret_plot::style::{LinePlotStyle, SeriesTooltipMode};
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct InfLinesDemoWindowState {
                    ui: UiTree<App>,
                    plot: LinePlotPanelBinding,
                }

                impl InfLinesDemoDriver {
                    fn build_ui(app: &mut App) -> InfLinesDemoWindowState {
                        let model = LinePlotModel::from_series(vec![
                            LineSeries::new("signal", data).y_axis(YAxis::Right),
                        ]);
                        let mut state = PlotState::default();
                        state.overlays = PlotOverlays {
                            inf_lines_x: vec![InfLineX::new(25.0)],
                            inf_lines_y: vec![InfLineY::new(0.0, YAxis::Left)],
                        };
                        let plot = LinePlotPanelBinding::new_with_state(app, model, state);
                        InfLinesDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut InfLinesDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut InfLinesDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("inf-lines-demo", move |cx| {
                            let style = LinePlotStyle {
                                series_tooltip: SeriesTooltipMode::NearestAtCursor,
                            };
                            let props = plot
                                .panel_props()
                                .style(style)
                                .y_axis_labels(labels)
                                .y2_axis_labels(labels)
                                .y3_axis_labels(labels)
                                .y4_axis_labels(labels);
                            vec![line_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyInfLinesPlot {
                    plot: fret_runtime::Model<LinePlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<LinePlotModel>,
                    plot_state: Model<PlotState>,
                    plot_output: Model<PlotOutput>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = LinePlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = LinePlotPanelProps::new(plot.clone())
                        .state(plot_state.clone())
                        .output(plot_output.clone());
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/inf_lines_demo.rs",
                        "advanced_manual",
                        "fixture inf-lines plot demo",
                        owner="examples-plot-inf-lines",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "advanced-surface-plot-inf-lines-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 6)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("LinePlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("LinePlotPanelProps", messages)

    def test_plot_inf_lines_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/inf_lines_demo.rs",
                """
                use fret_plot::LinePlotPanelBinding;
                use fret_plot::declarative::line_plot_panel_in;
                use fret_plot::models::{LinePlotModel, LineSeries, YAxis};
                use fret_plot::state::{InfLineX, InfLineY, PlotOverlays, PlotState};
                use fret_plot::style::{LinePlotStyle, SeriesTooltipMode};
                use fret_ui::{UiTree, declarative};

                struct InfLinesDemoWindowState {
                    ui: UiTree<App>,
                    plot: LinePlotPanelBinding,
                }

                impl InfLinesDemoDriver {
                    fn build_ui(app: &mut App) -> InfLinesDemoWindowState {
                        let model = LinePlotModel::from_series(vec![
                            LineSeries::new("signal", data).y_axis(YAxis::Right),
                        ]);
                        let mut state = PlotState::default();
                        state.overlays = PlotOverlays {
                            inf_lines_x: vec![InfLineX::new(25.0)],
                            inf_lines_y: vec![InfLineY::new(0.0, YAxis::Left)],
                        };
                        let plot = LinePlotPanelBinding::new_with_state(app, model, state);
                        InfLinesDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut InfLinesDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut InfLinesDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("inf-lines-demo", move |cx| {
                            let style = LinePlotStyle {
                                series_tooltip: SeriesTooltipMode::NearestAtCursor,
                            };
                            let props = plot
                                .panel_props()
                                .style(style)
                                .y_axis_labels(labels)
                                .y2_axis_labels(labels)
                                .y3_axis_labels(labels)
                                .y4_axis_labels(labels);
                            vec![line_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/inf_lines_demo.rs",
                        "advanced_manual",
                        "fixture inf-lines plot demo",
                        owner="examples-plot-inf-lines",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-plot-inf-lines-declarative-binding-boundary"
                ]
            )

    def test_plot_linked_cursor_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/linked_cursor_demo.rs",
                """
                use fret_plot::declarative::{area_plot_panel_in, line_plot_panel_in};
                use fret_plot::declarative::{AreaPlotPanelProps, LinePlotPanelProps};
                use fret_plot::linking::{LinkedPlotGroup, LinkedPlotMember, PlotLinkPolicy};
                use fret_plot::models::{AreaPlotModel, AreaSeries, LinePlotModel, LineSeries};
                use fret_plot::retained;
                use fret_plot::{AreaPlotPanelBinding, LinePlotPanelBinding};
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct LinkedCursorDemoWindowState {
                    ui: UiTree<App>,
                    top_plot: LinePlotPanelBinding,
                    bottom_plot: AreaPlotPanelBinding,
                    linked: LinkedPlotGroup,
                }

                impl LinkedCursorDemoDriver {
                    fn build_ui(app: &mut App) -> LinkedCursorDemoWindowState {
                        let top_plot = LinePlotPanelBinding::new(
                            app,
                            LinePlotModel::from_series(vec![
                                LineSeries::new("top", data),
                            ]),
                        );
                        let bottom_plot = AreaPlotPanelBinding::new(
                            app,
                            AreaPlotModel::from_series(vec![
                                AreaSeries::new("bottom", data),
                            ]),
                        );
                        let mut linked = LinkedPlotGroup::new(PlotLinkPolicy::default());
                        linked.push_binding(&top_plot).push_binding(&bottom_plot);
                        LinkedCursorDemoWindowState {
                            ui: UiTree::new(),
                            top_plot,
                            bottom_plot,
                            linked,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut LinkedCursorDemoWindowState) {
                    state.linked.tick(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut LinkedCursorDemoWindowState,
                ) {
                    let top_node = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("linked-cursor-demo-top", {
                            let top_plot = state.top_plot.clone();
                            move |cx| {
                                let top_style = LinePlotStyle::default();
                                let props = top_plot.panel_props().style(top_style);
                                vec![line_plot_panel_in(cx, props)]
                            }
                        });
                    let bottom_node = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("linked-cursor-demo-bottom", {
                            let bottom_plot = state.bottom_plot.clone();
                            move |cx| {
                                let bottom_style = LinePlotStyle::default();
                                let props = bottom_plot.panel_props().style(bottom_style);
                                vec![area_plot_panel_in(cx, props)]
                            }
                        });
                    state.ui.set_focus(Some(top_node));
                    let _ = bottom_node;
                }

                struct LegacyLinkedCursorPlot {
                    top_plot: fret_runtime::Model<LinePlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    state: &mut LinkedCursorDemoWindowState,
                    top_plot: Model<LinePlotModel>,
                    top_state: Model<PlotState>,
                    top_output: Model<PlotOutput>,
                    bottom_plot: AreaPlotPanelBinding,
                    bottom_state: Model<PlotState>,
                    bottom_output: Model<PlotOutput>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = LinePlotCanvas::new();
                    LinePlotCanvas::create_node(&mut state.ui, top_canvas);
                    let _ = AreaPlotCanvas::new();
                    AreaPlotCanvas::create_node(&mut state.ui, bottom_canvas);
                    let _member = LinkedPlotMember;
                    let _top_props = LinePlotPanelProps::new(top_plot)
                        .state(top_state)
                        .output(top_output);
                    let _bottom_props = AreaPlotPanelProps::new(bottom_plot.clone())
                        .state(bottom_state.clone())
                        .output(bottom_output.clone());
                    let _ = PlotState::default();
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/linked_cursor_demo.rs",
                        "advanced_manual",
                        "fixture linked cursor plot demo",
                        owner="examples-plot-linked-cursor",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "advanced-surface-plot-linked-cursor-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 10)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("LinePlotCanvas", messages)
            self.assertIn("AreaPlotCanvas", messages)
            self.assertIn("LinkedPlotMember", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("LinePlotPanelProps", messages)
            self.assertIn("AreaPlotPanelProps", messages)

    def test_plot_linked_cursor_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/linked_cursor_demo.rs",
                """
                use fret_plot::declarative::{area_plot_panel_in, line_plot_panel_in};
                use fret_plot::linking::{LinkedPlotGroup, PlotLinkPolicy};
                use fret_plot::models::{AreaPlotModel, AreaSeries, LinePlotModel, LineSeries};
                use fret_plot::{AreaPlotPanelBinding, LinePlotPanelBinding};
                use fret_ui::{UiTree, declarative};

                struct LinkedCursorDemoWindowState {
                    ui: UiTree<App>,
                    top_plot: LinePlotPanelBinding,
                    bottom_plot: AreaPlotPanelBinding,
                    linked: LinkedPlotGroup,
                }

                impl LinkedCursorDemoDriver {
                    fn build_ui(app: &mut App) -> LinkedCursorDemoWindowState {
                        let top_plot = LinePlotPanelBinding::new(
                            app,
                            LinePlotModel::from_series(vec![
                                LineSeries::new("top", data),
                            ]),
                        );
                        let bottom_plot = AreaPlotPanelBinding::new(
                            app,
                            AreaPlotModel::from_series(vec![
                                AreaSeries::new("bottom", data),
                            ]),
                        );
                        let mut linked = LinkedPlotGroup::new(PlotLinkPolicy::default());
                        linked.push_binding(&top_plot).push_binding(&bottom_plot);
                        LinkedCursorDemoWindowState {
                            ui: UiTree::new(),
                            top_plot,
                            bottom_plot,
                            linked,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut LinkedCursorDemoWindowState) {
                    state.linked.tick(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut LinkedCursorDemoWindowState,
                ) {
                    let top_node = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("linked-cursor-demo-top", {
                            let top_plot = state.top_plot.clone();
                            move |cx| {
                                let top_style = LinePlotStyle::default();
                                let props = top_plot.panel_props().style(top_style);
                                vec![line_plot_panel_in(cx, props)]
                            }
                        });
                    let bottom_node = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("linked-cursor-demo-bottom", {
                            let bottom_plot = state.bottom_plot.clone();
                            move |cx| {
                                let bottom_style = LinePlotStyle::default();
                                let props = bottom_plot.panel_props().style(bottom_style);
                                vec![area_plot_panel_in(cx, props)]
                            }
                        });
                    state.ui.set_focus(Some(top_node));
                    let _ = bottom_node;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/linked_cursor_demo.rs",
                        "advanced_manual",
                        "fixture linked cursor plot demo",
                        owner="examples-plot-linked-cursor",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-plot-linked-cursor-declarative-binding-boundary"
                ]
            )

    def test_plot_area_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/area_demo.rs",
                """
                use fret_plot::AreaPlotPanelBinding;
                use fret_plot::declarative::{AreaPlotPanelProps, area_plot_panel_in};
                use fret_plot::models::{AreaPlotModel, AreaSeries};
                use fret_plot::retained;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct AreaDemoWindowState {
                    ui: UiTree<App>,
                    plot: AreaPlotPanelBinding,
                }

                impl AreaDemoDriver {
                    fn build_ui(app: &mut App) -> AreaDemoWindowState {
                        let plot = AreaPlotPanelBinding::new(
                            app,
                            AreaPlotModel::from_series(vec![
                                AreaSeries::new("area", data),
                            ]),
                        );
                        AreaDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut AreaDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut AreaDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("area-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![area_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyAreaPlot {
                    plot: fret_runtime::Model<AreaPlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<AreaPlotModel>,
                    plot_state: Model<PlotState>,
                    plot_output: Model<PlotOutput>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = AreaPlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = AreaPlotPanelProps::new(plot.clone())
                        .state(plot_state.clone())
                        .output(plot_output.clone());
                    let _ = app.models_mut().insert(PlotState::default());
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/area_demo.rs",
                        "advanced_manual",
                        "fixture area plot demo",
                        owner="examples-plot-area",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-plot-area-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 7)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("AreaPlotCanvas", messages)
            self.assertIn("PlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("AreaPlotPanelProps", messages)

    def test_plot_area_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/area_demo.rs",
                """
                use fret_plot::AreaPlotPanelBinding;
                use fret_plot::declarative::area_plot_panel_in;
                use fret_plot::models::{AreaPlotModel, AreaSeries};
                use fret_ui::{UiTree, declarative};

                struct AreaDemoWindowState {
                    ui: UiTree<App>,
                    plot: AreaPlotPanelBinding,
                }

                impl AreaDemoDriver {
                    fn build_ui(app: &mut App) -> AreaDemoWindowState {
                        let plot = AreaPlotPanelBinding::new(
                            app,
                            AreaPlotModel::from_series(vec![
                                AreaSeries::new("area", data),
                            ]),
                        );
                        AreaDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut AreaDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut AreaDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("area-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![area_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/area_demo.rs",
                        "advanced_manual",
                        "fixture area plot demo",
                        owner="examples-plot-area",
                        allowed_raw_seams=("fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "advanced-surface-plot-area-declarative-binding-boundary"
                ]
            )

    def test_plot_stems_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/stems_demo.rs",
                """
                use fret_plot::StemsPlotPanelBinding;
                use fret_plot::declarative::{StemsPlotPanelProps, stems_plot_panel_in};
                use fret_plot::models::{StemsPlotModel, StemsSeries};
                use fret_plot::retained;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct StemsDemoWindowState {
                    ui: UiTree<App>,
                    plot: StemsPlotPanelBinding,
                }

                impl StemsDemoDriver {
                    fn build_ui(app: &mut App) -> StemsDemoWindowState {
                        let series = vec![
                            StemsSeries::new("stems", data),
                        ];
                        let plot = StemsPlotPanelBinding::new(app, StemsPlotModel::from_series(series));
                        StemsDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut StemsDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut StemsDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("stems-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![stems_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyStemsPlot {
                    plot: fret_runtime::Model<StemsPlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<StemsPlotModel>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = StemsPlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = StemsPlotPanelProps::new(plot.clone());
                    let _ = app.models_mut().insert(PlotState::default());
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/stems_demo.rs",
                        "advanced_manual",
                        "fixture stems plot demo",
                        owner="examples-plot-stems",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-plot-stems-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 7)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("StemsPlotCanvas", messages)
            self.assertIn("PlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("StemsPlotPanelProps", messages)

    def test_plot_stems_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/stems_demo.rs",
                """
                use fret_plot::StemsPlotPanelBinding;
                use fret_plot::declarative::stems_plot_panel_in;
                use fret_plot::models::{StemsPlotModel, StemsSeries};
                use fret_ui::{UiTree, declarative};

                struct StemsDemoWindowState {
                    ui: UiTree<App>,
                    plot: StemsPlotPanelBinding,
                }

                impl StemsDemoDriver {
                    fn build_ui(app: &mut App) -> StemsDemoWindowState {
                        let series = vec![
                            StemsSeries::new("stems", data),
                        ];
                        let plot = StemsPlotPanelBinding::new(app, StemsPlotModel::from_series(series));
                        StemsDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut StemsDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut StemsDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("stems-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![stems_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/stems_demo.rs",
                        "advanced_manual",
                        "fixture stems plot demo",
                        owner="examples-plot-stems",
                        allowed_raw_seams=("fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "advanced-surface-plot-stems-declarative-binding-boundary"
                ]
            )

    def test_plot_stairs_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/stairs_demo.rs",
                """
                use fret_plot::LinePlotPanelBinding;
                use fret_plot::declarative::{LinePlotPanelProps, line_plot_panel_in};
                use fret_plot::models::{LinePlotModel, LineSeries, StepMode};
                use fret_plot::retained;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct StairsDemoWindowState {
                    ui: UiTree<App>,
                    plot: LinePlotPanelBinding,
                }

                impl StairsDemoDriver {
                    fn build_ui(app: &mut App) -> StairsDemoWindowState {
                        let plot = LinePlotPanelBinding::new(
                            app,
                            LinePlotModel::from_series(vec![
                                LineSeries::new("stairs", data),
                            ]),
                        );
                        StairsDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut StairsDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut StairsDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("stairs-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().step_mode(StepMode::Post).style(style);
                            vec![line_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyStairsPlot {
                    plot: fret_runtime::Model<LinePlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<LinePlotModel>,
                    plot_state: Model<PlotState>,
                    plot_output: Model<PlotOutput>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = StairsPlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = LinePlotPanelProps::new(plot.clone())
                        .state(plot_state.clone())
                        .output(plot_output.clone());
                    let _ = app.models_mut().insert(PlotState::default());
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/stairs_demo.rs",
                        "advanced_manual",
                        "fixture stairs plot demo",
                        owner="examples-plot-stairs",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-plot-stairs-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("StairsPlotCanvas", messages)
            self.assertIn("PlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("LinePlotPanelProps", messages)

    def test_plot_stairs_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/stairs_demo.rs",
                """
                use fret_plot::LinePlotPanelBinding;
                use fret_plot::declarative::line_plot_panel_in;
                use fret_plot::models::{LinePlotModel, LineSeries, StepMode};
                use fret_ui::{UiTree, declarative};

                struct StairsDemoWindowState {
                    ui: UiTree<App>,
                    plot: LinePlotPanelBinding,
                }

                impl StairsDemoDriver {
                    fn build_ui(app: &mut App) -> StairsDemoWindowState {
                        let plot = LinePlotPanelBinding::new(
                            app,
                            LinePlotModel::from_series(vec![
                                LineSeries::new("stairs", data),
                            ]),
                        );
                        StairsDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut StairsDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut StairsDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("stairs-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().step_mode(StepMode::Post).style(style);
                            vec![line_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/stairs_demo.rs",
                        "advanced_manual",
                        "fixture stairs plot demo",
                        owner="examples-plot-stairs",
                        allowed_raw_seams=("fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "advanced-surface-plot-stairs-declarative-binding-boundary"
                ]
            )

    def test_plot_shaded_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/shaded_demo.rs",
                """
                use fret_plot::ShadedPlotPanelBinding;
                use fret_plot::declarative::{ShadedPlotPanelProps, shaded_plot_panel_in};
                use fret_plot::models::{ShadedPlotModel, ShadedSeries};
                use fret_plot::plot::axis::{AxisLabelFormatter, TimeAxisFormat};
                use fret_plot::retained;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct ShadedDemoWindowState {
                    ui: UiTree<App>,
                    plot: ShadedPlotPanelBinding,
                }

                impl ShadedDemoDriver {
                    fn build_ui(app: &mut App) -> ShadedDemoWindowState {
                        let plot = ShadedPlotPanelBinding::new(
                            app,
                            ShadedPlotModel::from_series(vec![
                                ShadedSeries::new("band", upper, lower),
                            ]),
                        );
                        ShadedDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut ShadedDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut ShadedDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("shaded-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot
                                .panel_props()
                                .style(style)
                                .x_axis_labels(AxisLabelFormatter::time_seconds(TimeAxisFormat {
                                    base_seconds: 1_700_000_000.0,
                                }));
                            vec![shaded_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyShadedPlot {
                    plot: fret_runtime::Model<ShadedPlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<ShadedPlotModel>,
                    plot_state: Model<PlotState>,
                    plot_output: Model<PlotOutput>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = ShadedPlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = ShadedPlotPanelProps::new(plot.clone())
                        .state(plot_state.clone())
                        .output(plot_output.clone());
                    let _ = app.models_mut().insert(PlotState::default());
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/shaded_demo.rs",
                        "advanced_manual",
                        "fixture shaded plot demo",
                        owner="examples-plot-shaded",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-plot-shaded-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("ShadedPlotCanvas", messages)
            self.assertIn("PlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("ShadedPlotPanelProps", messages)

    def test_plot_shaded_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/shaded_demo.rs",
                """
                use fret_plot::ShadedPlotPanelBinding;
                use fret_plot::declarative::shaded_plot_panel_in;
                use fret_plot::models::{ShadedPlotModel, ShadedSeries};
                use fret_plot::plot::axis::{AxisLabelFormatter, TimeAxisFormat};
                use fret_ui::{UiTree, declarative};

                struct ShadedDemoWindowState {
                    ui: UiTree<App>,
                    plot: ShadedPlotPanelBinding,
                }

                impl ShadedDemoDriver {
                    fn build_ui(app: &mut App) -> ShadedDemoWindowState {
                        let plot = ShadedPlotPanelBinding::new(
                            app,
                            ShadedPlotModel::from_series(vec![
                                ShadedSeries::new("band", upper, lower),
                            ]),
                        );
                        ShadedDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut ShadedDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut ShadedDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("shaded-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot
                                .panel_props()
                                .style(style)
                                .x_axis_labels(AxisLabelFormatter::time_seconds(TimeAxisFormat {
                                    base_seconds: 1_700_000_000.0,
                                }));
                            vec![shaded_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/shaded_demo.rs",
                        "advanced_manual",
                        "fixture shaded plot demo",
                        owner="examples-plot-shaded",
                        allowed_raw_seams=("fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "advanced-surface-plot-shaded-declarative-binding-boundary"
                ]
            )

    def test_plot_error_bars_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/error_bars_demo.rs",
                """
                use fret_plot::ErrorBarsPlotPanelBinding;
                use fret_plot::declarative::{ErrorBarsPlotPanelProps, error_bars_plot_panel_in};
                use fret_plot::models::{ErrorBar, ErrorBarsPlotModel, ErrorBarsSeries, YAxis};
                use fret_plot::retained;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};
                use std::sync::Arc;

                struct ErrorBarsDemoWindowState {
                    ui: UiTree<App>,
                    plot: ErrorBarsPlotPanelBinding,
                }

                impl ErrorBarsDemoDriver {
                    fn build_ui(app: &mut App) -> ErrorBarsDemoWindowState {
                        let plot = ErrorBarsPlotPanelBinding::new(
                            app,
                            ErrorBarsPlotModel::from_series(vec![
                                ErrorBarsSeries::new("measurement", points)
                                    .y_axis(YAxis::Right)
                                    .y_errors(Arc::from(left_y_errors))
                                    .x_errors(Arc::from(left_x_errors)),
                            ]),
                        );
                        ErrorBarsDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut ErrorBarsDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut ErrorBarsDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("error-bars-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![error_bars_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyErrorBarsPlot {
                    plot: fret_runtime::Model<ErrorBarsPlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<ErrorBarsPlotModel>,
                    plot_state: Model<PlotState>,
                    plot_output: Model<PlotOutput>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = ErrorBarsPlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = ErrorBarsPlotPanelProps::new(plot.clone())
                        .state(plot_state.clone())
                        .output(plot_output.clone());
                    let _ = app.models_mut().insert(PlotState::default());
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/error_bars_demo.rs",
                        "advanced_manual",
                        "fixture error-bars plot demo",
                        owner="examples-plot-error-bars",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "advanced-surface-plot-error-bars-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("ErrorBarsPlotCanvas", messages)
            self.assertIn("PlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("ErrorBarsPlotPanelProps", messages)

    def test_plot_error_bars_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/error_bars_demo.rs",
                """
                use fret_plot::ErrorBarsPlotPanelBinding;
                use fret_plot::declarative::error_bars_plot_panel_in;
                use fret_plot::models::{ErrorBar, ErrorBarsPlotModel, ErrorBarsSeries, YAxis};
                use fret_ui::{UiTree, declarative};
                use std::sync::Arc;

                struct ErrorBarsDemoWindowState {
                    ui: UiTree<App>,
                    plot: ErrorBarsPlotPanelBinding,
                }

                impl ErrorBarsDemoDriver {
                    fn build_ui(app: &mut App) -> ErrorBarsDemoWindowState {
                        let plot = ErrorBarsPlotPanelBinding::new(
                            app,
                            ErrorBarsPlotModel::from_series(vec![
                                ErrorBarsSeries::new("measurement", points)
                                    .y_axis(YAxis::Right)
                                    .y_errors(Arc::from(left_y_errors))
                                    .x_errors(Arc::from(left_x_errors)),
                            ]),
                        );
                        ErrorBarsDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut ErrorBarsDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut ErrorBarsDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("error-bars-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![error_bars_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/error_bars_demo.rs",
                        "advanced_manual",
                        "fixture error-bars plot demo",
                        owner="examples-plot-error-bars",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-plot-error-bars-declarative-binding-boundary"
                ]
            )

    def test_plot_histogram_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/histogram_demo.rs",
                """
                use fret_plot::HistogramPlotPanelBinding;
                use fret_plot::declarative::{HistogramPlotPanelProps, histogram_plot_panel_in};
                use fret_plot::models::{HistogramPlotModel, HistogramSeries};
                use fret_plot::retained;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct HistogramDemoWindowState {
                    ui: UiTree<App>,
                    plot: HistogramPlotPanelBinding,
                }

                impl HistogramDemoDriver {
                    fn build_ui(app: &mut App) -> HistogramDemoWindowState {
                        let series = vec![
                            HistogramSeries::new("histogram", samples)
                                .bins(80)
                                .bar_gap_fraction(0.12),
                        ];
                        let plot = HistogramPlotPanelBinding::new(app, HistogramPlotModel::from_series(series));
                        HistogramDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut HistogramDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut HistogramDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("histogram-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![histogram_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyHistogramPlot {
                    plot: fret_runtime::Model<HistogramPlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<HistogramPlotModel>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = HistogramPlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = HistogramPlotPanelProps::new(plot.clone());
                    let _ = app.models_mut().insert(PlotState::default());
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/histogram_demo.rs",
                        "advanced_manual",
                        "fixture histogram plot demo",
                        owner="examples-plot-histogram",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-plot-histogram-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 7)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("HistogramPlotCanvas", messages)
            self.assertIn("PlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("HistogramPlotPanelProps", messages)

    def test_plot_histogram_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/histogram_demo.rs",
                """
                use fret_plot::HistogramPlotPanelBinding;
                use fret_plot::declarative::histogram_plot_panel_in;
                use fret_plot::models::{HistogramPlotModel, HistogramSeries};
                use fret_ui::{UiTree, declarative};

                struct HistogramDemoWindowState {
                    ui: UiTree<App>,
                    plot: HistogramPlotPanelBinding,
                }

                impl HistogramDemoDriver {
                    fn build_ui(app: &mut App) -> HistogramDemoWindowState {
                        let series = vec![
                            HistogramSeries::new("histogram", samples)
                                .bins(80)
                                .bar_gap_fraction(0.12),
                        ];
                        let plot = HistogramPlotPanelBinding::new(app, HistogramPlotModel::from_series(series));
                        HistogramDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut HistogramDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut HistogramDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("histogram-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![histogram_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/histogram_demo.rs",
                        "advanced_manual",
                        "fixture histogram plot demo",
                        owner="examples-plot-histogram",
                        allowed_raw_seams=("fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "advanced-surface-plot-histogram-declarative-binding-boundary"
                ]
            )

    def test_plot_grouped_bars_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/grouped_bars_demo.rs",
                """
                use fret_plot::BarsPlotPanelBinding;
                use fret_plot::declarative::{BarsPlotPanelProps, bars_plot_panel_in};
                use fret_plot::models::{BarsPlotModel, CategoryBarSeries};
                use fret_plot::retained;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct GroupedBarsDemoWindowState {
                    ui: UiTree<App>,
                    plot: BarsPlotPanelBinding,
                }

                impl GroupedBarsDemoDriver {
                    fn build_ui(app: &mut App) -> GroupedBarsDemoWindowState {
                        let series = vec![
                            CategoryBarSeries::new("A", a),
                            CategoryBarSeries::new("B", b),
                        ];
                        let plot = BarsPlotPanelBinding::new(
                            app,
                            BarsPlotModel::grouped_categories(categories, series, 0.75, 0.18, 0.0),
                        );
                        GroupedBarsDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut GroupedBarsDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut GroupedBarsDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("grouped-bars-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![bars_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyGroupedBarsPlot {
                    plot: fret_runtime::Model<BarsPlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<BarsPlotModel>,
                    plot_state: Model<PlotState>,
                    plot_output: Model<PlotOutput>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = BarsPlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = BarsPlotPanelProps::new(plot.clone())
                        .state(plot_state.clone())
                        .output(plot_output.clone());
                    let _ = app.models_mut().insert(PlotState::default());
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/grouped_bars_demo.rs",
                        "advanced_manual",
                        "fixture grouped bars plot demo",
                        owner="examples-plot-grouped-bars",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "advanced-surface-plot-grouped-bars-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("BarsPlotCanvas", messages)
            self.assertIn("PlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("BarsPlotPanelProps", messages)

    def test_plot_grouped_bars_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/grouped_bars_demo.rs",
                """
                use fret_plot::BarsPlotPanelBinding;
                use fret_plot::declarative::bars_plot_panel_in;
                use fret_plot::models::{BarsPlotModel, CategoryBarSeries};
                use fret_ui::{UiTree, declarative};

                struct GroupedBarsDemoWindowState {
                    ui: UiTree<App>,
                    plot: BarsPlotPanelBinding,
                }

                impl GroupedBarsDemoDriver {
                    fn build_ui(app: &mut App) -> GroupedBarsDemoWindowState {
                        let series = vec![
                            CategoryBarSeries::new("A", a),
                            CategoryBarSeries::new("B", b),
                        ];
                        let plot = BarsPlotPanelBinding::new(
                            app,
                            BarsPlotModel::grouped_categories(categories, series, 0.75, 0.18, 0.0),
                        );
                        GroupedBarsDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut GroupedBarsDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut GroupedBarsDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("grouped-bars-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![bars_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/grouped_bars_demo.rs",
                        "advanced_manual",
                        "fixture grouped bars plot demo",
                        owner="examples-plot-grouped-bars",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-plot-grouped-bars-declarative-binding-boundary"
                ]
            )

    def test_plot_stacked_bars_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/stacked_bars_demo.rs",
                """
                use fret_plot::BarsPlotPanelBinding;
                use fret_plot::declarative::{BarsPlotPanelProps, bars_plot_panel_in};
                use fret_plot::models::{BarsPlotModel, CategoryBarSeries};
                use fret_plot::retained;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct StackedBarsDemoWindowState {
                    ui: UiTree<App>,
                    plot: BarsPlotPanelBinding,
                }

                impl StackedBarsDemoDriver {
                    fn build_ui(app: &mut App) -> StackedBarsDemoWindowState {
                        let series = vec![
                            CategoryBarSeries::new("A", a),
                            CategoryBarSeries::new("B", b),
                        ];
                        let plot = BarsPlotPanelBinding::new(
                            app,
                            BarsPlotModel::stacked_categories(categories, series, 0.8),
                        );
                        StackedBarsDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut StackedBarsDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut StackedBarsDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("stacked-bars-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![bars_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyStackedBarsPlot {
                    plot: fret_runtime::Model<BarsPlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<BarsPlotModel>,
                    plot_state: Model<PlotState>,
                    plot_output: Model<PlotOutput>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = BarsPlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = BarsPlotPanelProps::new(plot.clone())
                        .state(plot_state.clone())
                        .output(plot_output.clone());
                    let _ = app.models_mut().insert(PlotState::default());
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/stacked_bars_demo.rs",
                        "advanced_manual",
                        "fixture stacked bars plot demo",
                        owner="examples-plot-stacked-bars",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "advanced-surface-plot-stacked-bars-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("BarsPlotCanvas", messages)
            self.assertIn("PlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("BarsPlotPanelProps", messages)

    def test_plot_stacked_bars_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/stacked_bars_demo.rs",
                """
                use fret_plot::BarsPlotPanelBinding;
                use fret_plot::declarative::bars_plot_panel_in;
                use fret_plot::models::{BarsPlotModel, CategoryBarSeries};
                use fret_ui::{UiTree, declarative};

                struct StackedBarsDemoWindowState {
                    ui: UiTree<App>,
                    plot: BarsPlotPanelBinding,
                }

                impl StackedBarsDemoDriver {
                    fn build_ui(app: &mut App) -> StackedBarsDemoWindowState {
                        let series = vec![
                            CategoryBarSeries::new("A", a),
                            CategoryBarSeries::new("B", b),
                        ];
                        let plot = BarsPlotPanelBinding::new(
                            app,
                            BarsPlotModel::stacked_categories(categories, series, 0.8),
                        );
                        StackedBarsDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut StackedBarsDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut StackedBarsDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("stacked-bars-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![bars_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/stacked_bars_demo.rs",
                        "advanced_manual",
                        "fixture stacked bars plot demo",
                        owner="examples-plot-stacked-bars",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-plot-stacked-bars-declarative-binding-boundary"
                ]
            )

    def test_plot_candlestick_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/candlestick_demo.rs",
                """
                use fret_plot::CandlestickPlotPanelBinding;
                use fret_plot::declarative::{CandlestickPlotPanelProps, candlestick_plot_panel_in};
                use fret_plot::models::{CandlestickPlotModel, CandlestickSeries, OhlcPoint};
                use fret_plot::retained;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct CandlestickDemoWindowState {
                    ui: UiTree<App>,
                    plot: CandlestickPlotPanelBinding,
                }

                impl CandlestickDemoDriver {
                    fn build_ui(app: &mut App) -> CandlestickDemoWindowState {
                        let out: Vec<OhlcPoint> = vec![OhlcPoint {
                            x: 0.0,
                            open: 1.0,
                            high: 2.0,
                            low: 0.5,
                            close: 1.5,
                        }];
                        let plot = CandlestickPlotPanelBinding::new(
                            app,
                            CandlestickPlotModel::from_series(vec![
                                CandlestickSeries::new_sorted("ohlc", Arc::from(out), true)
                                    .width(0.9),
                            ]),
                        );
                        CandlestickDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut CandlestickDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut CandlestickDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("candlestick-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![candlestick_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyCandlestickPlot {
                    plot: fret_runtime::Model<CandlestickPlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<CandlestickPlotModel>,
                    plot_state: Model<PlotState>,
                    plot_output: Model<PlotOutput>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = CandlestickPlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = CandlestickPlotPanelProps::new(plot.clone())
                        .state(plot_state.clone())
                        .output(plot_output.clone());
                    let _ = app.models_mut().insert(PlotState::default());
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/candlestick_demo.rs",
                        "advanced_manual",
                        "fixture candlestick plot demo",
                        owner="examples-plot-candlestick",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "advanced-surface-plot-candlestick-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("CandlestickPlotCanvas", messages)
            self.assertIn("PlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("CandlestickPlotPanelProps", messages)

    def test_plot_candlestick_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/candlestick_demo.rs",
                """
                use fret_plot::CandlestickPlotPanelBinding;
                use fret_plot::declarative::candlestick_plot_panel_in;
                use fret_plot::models::{CandlestickPlotModel, CandlestickSeries, OhlcPoint};
                use fret_ui::{UiTree, declarative};

                struct CandlestickDemoWindowState {
                    ui: UiTree<App>,
                    plot: CandlestickPlotPanelBinding,
                }

                impl CandlestickDemoDriver {
                    fn build_ui(app: &mut App) -> CandlestickDemoWindowState {
                        let out: Vec<OhlcPoint> = vec![OhlcPoint {
                            x: 0.0,
                            open: 1.0,
                            high: 2.0,
                            low: 0.5,
                            close: 1.5,
                        }];
                        let plot = CandlestickPlotPanelBinding::new(
                            app,
                            CandlestickPlotModel::from_series(vec![
                                CandlestickSeries::new_sorted("ohlc", Arc::from(out), true)
                                    .width(0.9),
                            ]),
                        );
                        CandlestickDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut CandlestickDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut CandlestickDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("candlestick-demo", move |cx| {
                            let style = LinePlotStyle::default();
                            let props = plot.panel_props().style(style);
                            vec![candlestick_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/candlestick_demo.rs",
                        "advanced_manual",
                        "fixture candlestick plot demo",
                        owner="examples-plot-candlestick",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-plot-candlestick-declarative-binding-boundary"
                ]
            )

    def test_plot_heatmap_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/heatmap_demo.rs",
                """
                use fret_plot::HeatmapPlotPanelBinding;
                use fret_plot::declarative::{HeatmapPlotPanelProps, heatmap_plot_panel_in};
                use fret_plot::models::HeatmapPlotModel;
                use fret_plot::retained;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct HeatmapDemoWindowState {
                    ui: UiTree<App>,
                    plot: HeatmapPlotPanelBinding,
                }

                impl HeatmapDemoDriver {
                    fn build_ui(app: &mut App) -> HeatmapDemoWindowState {
                        let cols = 2usize;
                        let rows = 2usize;
                        let data_bounds = DataRect {
                            x_min: 0.0,
                            x_max: 1.0,
                            y_min: 0.0,
                            y_max: 1.0,
                        };
                        let values = vec![0.0, 0.2, 0.4, 0.6];
                        let plot = HeatmapPlotPanelBinding::new(
                            app,
                            HeatmapPlotModel::new(data_bounds, cols, rows, values),
                        );
                        HeatmapDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut HeatmapDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut HeatmapDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("heatmap-demo", move |cx| {
                            let mut style = LinePlotStyle::default();
                            style.heatmap_show_colorbar = true;
                            let props = plot.panel_props().style(style);
                            vec![heatmap_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyHeatmapPlot {
                    plot: fret_runtime::Model<HeatmapPlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<HeatmapPlotModel>,
                    plot_state: Model<PlotState>,
                    plot_output: Model<PlotOutput>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = HeatmapPlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = HeatmapPlotPanelProps::new(plot.clone())
                        .state(plot_state.clone())
                        .output(plot_output.clone());
                    let _ = app.models_mut().insert(PlotState::default());
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/heatmap_demo.rs",
                        "advanced_manual",
                        "fixture heatmap plot demo",
                        owner="examples-plot-heatmap",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "advanced-surface-plot-heatmap-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("HeatmapPlotCanvas", messages)
            self.assertIn("PlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("HeatmapPlotPanelProps", messages)

    def test_plot_heatmap_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/heatmap_demo.rs",
                """
                use fret_plot::HeatmapPlotPanelBinding;
                use fret_plot::declarative::heatmap_plot_panel_in;
                use fret_plot::models::HeatmapPlotModel;
                use fret_ui::{UiTree, declarative};

                struct HeatmapDemoWindowState {
                    ui: UiTree<App>,
                    plot: HeatmapPlotPanelBinding,
                }

                impl HeatmapDemoDriver {
                    fn build_ui(app: &mut App) -> HeatmapDemoWindowState {
                        let cols = 2usize;
                        let rows = 2usize;
                        let data_bounds = DataRect {
                            x_min: 0.0,
                            x_max: 1.0,
                            y_min: 0.0,
                            y_max: 1.0,
                        };
                        let values = vec![0.0, 0.2, 0.4, 0.6];
                        let plot = HeatmapPlotPanelBinding::new(
                            app,
                            HeatmapPlotModel::new(data_bounds, cols, rows, values),
                        );
                        HeatmapDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn handle_event(app: &mut App, state: &mut HeatmapDemoWindowState) {
                    let _ = state.plot.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut HeatmapDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("heatmap-demo", move |cx| {
                            let mut style = LinePlotStyle::default();
                            style.heatmap_show_colorbar = true;
                            let props = plot.panel_props().style(style);
                            vec![heatmap_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/heatmap_demo.rs",
                        "advanced_manual",
                        "fixture heatmap plot demo",
                        owner="examples-plot-heatmap",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-plot-heatmap-declarative-binding-boundary"
                ]
            )

    def test_plot_histogram2d_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/histogram2d_demo.rs",
                """
                use fret_plot::Histogram2DPlotPanelBinding;
                use fret_plot::declarative::{Histogram2DPlotPanelProps, histogram2d_plot_panel_in};
                use fret_plot::models::Histogram2DPlotModel;
                use fret_plot::plot::axis::{AxisLabelFormatter, AxisNumberFormat};
                use fret_plot::plot::histogram2d::{Histogram2DConfig, histogram2d_counts};
                use fret_plot::retained;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct Histogram2DDemoWindowState {
                    ui: UiTree<App>,
                    plot: Histogram2DPlotPanelBinding,
                }

                impl Histogram2DDemoDriver {
                    fn build_ui(app: &mut App) -> Histogram2DDemoWindowState {
                        let bounds = DataRect::default();
                        let points = Vec::new();
                        let grid = histogram2d_counts(Histogram2DConfig::new(bounds, 256, 192), points);
                        let model = Histogram2DPlotModel::new(grid.data_bounds, grid.cols, grid.rows, grid.values);
                        let plot = Histogram2DPlotPanelBinding::new(app, model);
                        Histogram2DDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut Histogram2DDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("histogram2d-demo", move |cx| {
                            let props = plot
                                .panel_props()
                                .x_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)))
                                .y_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)));
                            vec![histogram2d_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }

                struct LegacyHistogram2DPlot {
                    plot: fret_runtime::Model<Histogram2DPlotModel>,
                    output: Model<PlotOutput>,
                }

                fn bad(
                    app: &mut App,
                    plot: Model<Histogram2DPlotModel>,
                    plot_state: Model<PlotState>,
                    plot_output: Model<PlotOutput>,
                ) {
                    let _ = retained::legacy();
                    let _ = fret_plot::retained::legacy();
                    let _ = Histogram2DPlotCanvas;
                    let _ = PlotCanvas;
                    create_node_retained();
                    let _props = Histogram2DPlotPanelProps::new(plot.clone())
                        .state(plot_state.clone())
                        .output(plot_output.clone());
                    let _ = app.models_mut().insert(PlotState::default());
                    let _ = app.models_mut().insert(PlotOutput::default());
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/histogram2d_demo.rs",
                        "advanced_manual",
                        "fixture histogram2d plot demo",
                        owner="examples-plot-histogram2d",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "advanced-surface-plot-histogram2d-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("Histogram2DPlotCanvas", messages)
            self.assertIn("PlotCanvas", messages)
            self.assertIn("PlotOutput", messages)
            self.assertIn("Histogram2DPlotPanelProps", messages)

    def test_plot_histogram2d_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/histogram2d_demo.rs",
                """
                use fret_plot::Histogram2DPlotPanelBinding;
                use fret_plot::declarative::histogram2d_plot_panel_in;
                use fret_plot::models::Histogram2DPlotModel;
                use fret_plot::plot::axis::{AxisLabelFormatter, AxisNumberFormat};
                use fret_plot::plot::histogram2d::{Histogram2DConfig, histogram2d_counts};
                use fret_ui::{UiTree, declarative};

                struct Histogram2DDemoWindowState {
                    ui: UiTree<App>,
                    plot: Histogram2DPlotPanelBinding,
                }

                impl Histogram2DDemoDriver {
                    fn build_ui(app: &mut App) -> Histogram2DDemoWindowState {
                        let bounds = DataRect::default();
                        let points = Vec::new();
                        let grid = histogram2d_counts(Histogram2DConfig::new(bounds, 256, 192), points);
                        let model = Histogram2DPlotModel::new(grid.data_bounds, grid.cols, grid.rows, grid.values);
                        let plot = Histogram2DPlotPanelBinding::new(app, model);
                        Histogram2DDemoWindowState {
                            ui: UiTree::new(),
                            plot,
                        }
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut Histogram2DDemoWindowState,
                ) {
                    let plot = state.plot.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("histogram2d-demo", move |cx| {
                            let props = plot
                                .panel_props()
                                .x_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)))
                                .y_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)));
                            vec![histogram2d_plot_panel_in(cx, props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/histogram2d_demo.rs",
                        "advanced_manual",
                        "fixture histogram2d plot demo",
                        owner="examples-plot-histogram2d",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-plot-histogram2d-declarative-binding-boundary"
                ]
            )

    def test_plot_stress_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/plot_stress_demo.rs",
                """
                use fret_plot::LinePlotPanelBinding;
                use fret_plot::declarative::line_plot_panel_in;
                use fret_plot::declarative::{LinePlotPanelProps, line_plot_panel_in};
                use fret_plot::models::{LinePlotModel, LineSeries};
                use fret_plot::retained;
                use fret_ui::{UiTree, declarative};
                use std::sync::Arc;

                struct PlotStressWindowState {
                    ui: UiTree<App>,
                    models: PlotStressModelOwner,
                }

                struct PlotStressModelOwner {
                    plot: LinePlotPanelBinding,
                    animate: Model<bool>,
                }

                impl PlotStressModelOwner {
                    fn new(app: &mut App, points: usize, series: usize) -> Self {
                        Self {
                            plot: LinePlotPanelBinding::new(
                                app,
                                PlotStressDriver::build_plot_model(points, series),
                            ),
                            animate: app.models_mut().insert(true),
                        }
                    }

                    fn plot_binding(&self) -> LinePlotPanelBinding {
                        self.plot.clone()
                    }

                    fn animate_enabled(&self, app: &App) -> bool {
                        self.animate.read_ref(app, |value| *value).unwrap_or(false)
                    }

                    fn toggle_animate(&self, app: &mut App) {
                        let _ = self.animate.update(app, |value, _cx| *value = !*value);
                    }

                    fn shift_plot_bounds_for_animation(&self, app: &mut App, frame: u64) {
                        let _ = self.plot.update_model(app, |model, _cx| {
                            let _ = model;
                            let _ = frame;
                        });
                    }
                }

                impl PlotStressDriver {
                    fn build_plot_model(points: usize, series: usize) -> LinePlotModel {
                        let label: Arc<str> = Arc::from("signal");
                        let data = Self::build_series(points, series);
                        LinePlotModel::from_series_with_bounds(
                            vec![LineSeries::new(label, data)],
                            bounds,
                        )
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut PlotStressWindowState,
                ) {
                    let plot = state.models.plot_binding();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                    .render_root("plot-stress-demo", move |cx| {
                        let style = LinePlotStyle::default();
                        let props = plot.panel_props().style(style);
                        vec![line_plot_panel_in(cx, props)]
                    });
                    let _ = root;
                }

                struct LegacyPlotModels {
                    plot: Model<LinePlotModel>,
                }

                fn bad(app: &mut App, state: &PlotStressWindowState, points: usize, series: usize) {
                    let plot = app.models_mut().insert(PlotStressDriver::build_plot_model(points, series));
                    let _ = LinePlotPanelProps::new(plot.clone());
                    let _ = fret_plot::retained::something();
                    let _ = create_node_retained();
                    let _ = app.models().read(&state.animate, |_| true);
                    let _ = app.models_mut().update(&state.animate, |_| true);
                    let _ = app.models_mut().update(&state.plot, |_| true);
                }

                struct LinePlotCanvas;
                struct PlotCanvas;
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/plot_stress_demo.rs",
                        "internal_harness",
                        "fixture plot stress harness",
                        owner="examples-plot-stress",
                        allowed_raw_seams=("UiTree",),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "internal-harness-plot-stress-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 6)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("LinePlotPanelProps", messages)
            self.assertIn("fret_plot::retained", messages)
            self.assertIn("app.models_mut().update(&state.plot", messages)
            self.assertIn("LinePlotCanvas", messages)

    def test_plot_stress_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/plot_stress_demo.rs",
                """
                use fret_plot::LinePlotPanelBinding;
                use fret_plot::declarative::line_plot_panel_in;
                use fret_plot::models::{LinePlotModel, LineSeries};
                use fret_ui::{UiTree, declarative};
                use std::sync::Arc;

                struct PlotStressWindowState {
                    ui: UiTree<App>,
                    models: PlotStressModelOwner,
                }

                struct PlotStressModelOwner {
                    plot: LinePlotPanelBinding,
                    animate: Model<bool>,
                }

                impl PlotStressModelOwner {
                    fn new(app: &mut App, points: usize, series: usize) -> Self {
                        Self {
                            plot: LinePlotPanelBinding::new(
                                app,
                                PlotStressDriver::build_plot_model(points, series),
                            ),
                            animate: app.models_mut().insert(true),
                        }
                    }

                    fn plot_binding(&self) -> LinePlotPanelBinding {
                        self.plot.clone()
                    }

                    fn animate_enabled(&self, app: &App) -> bool {
                        self.animate.read_ref(app, |value| *value).unwrap_or(false)
                    }

                    fn toggle_animate(&self, app: &mut App) {
                        let _ = self.animate.update(app, |value, _cx| *value = !*value);
                    }

                    fn shift_plot_bounds_for_animation(&self, app: &mut App, frame: u64) {
                        let _ = self.plot.update_model(app, |model, _cx| {
                            let _ = model;
                            let _ = frame;
                        });
                    }
                }

                impl PlotStressDriver {
                    fn build_plot_model(points: usize, series: usize) -> LinePlotModel {
                        let label: Arc<str> = Arc::from("signal");
                        let data = Self::build_series(points, series);
                        LinePlotModel::from_series_with_bounds(
                            vec![LineSeries::new(label, data)],
                            bounds,
                        )
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut PlotStressWindowState,
                ) {
                    let plot = state.models.plot_binding();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                    .render_root("plot-stress-demo", move |cx| {
                        let style = LinePlotStyle::default();
                        let props = plot.panel_props().style(style);
                        vec![line_plot_panel_in(cx, props)]
                    });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/plot_stress_demo.rs",
                        "internal_harness",
                        "fixture plot stress harness",
                        owner="examples-plot-stress",
                        allowed_raw_seams=("UiTree",),
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
                    == "internal-harness-plot-stress-declarative-binding-boundary"
                ]
            )

    def test_chart_stress_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/chart_stress_demo.rs",
                """
                use fret_chart::{ChartCanvasPanelBinding, chart_canvas_panel};
                use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel};
                use fret_chart::retained::ChartCanvas;
                use fret_runtime::{Model, PlatformCapabilities};
                use fret_ui::UiTree;
                use fret_ui::retained_bridge::Bridge;

                pub struct ChartStressWindowState {
                    ui: UiTree<App>,
                    chart: ChartCanvasPanelBinding,
                }

                impl ChartStressDriver {
                    fn build_chart(
                        points: usize,
                        scatter_lod: Option<SeriesLodSpecV1>,
                    ) -> (ChartEngine, ChartSpec) {
                        let _ = points;
                        let _ = scatter_lod;
                        (ChartEngine::default(), ChartSpec::default())
                    }
                }

                fn create_window_state(driver: &mut ChartStressDriver, app: &mut App) -> ChartStressWindowState {
                    let (engine, spec) = ChartStressDriver::build_chart(driver.points, driver.scatter_lod);
                    let chart = ChartCanvasPanelBinding::new(app, spec, engine);
                    ChartStressWindowState { ui: UiTree::new(), chart }
                }

                fn render(
                    driver: &mut ChartStressDriver,
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut ChartStressWindowState,
                ) {
                    let chart = state.chart.clone();
                    let root = fret_ui::declarative::render_root(
                        &mut state.ui,
                        app,
                        services,
                        window,
                        bounds,
                        "chart-stress-demo-root",
                        move |cx| {
                            chart.observe_engine_paint(cx);
                            let props = chart.panel_props();
                            vec![chart_canvas_panel(cx, props)]
                        },
                    );
                    let _ = root;
                    let stats = state
                        .chart
                        .read_engine(app, |_app, engine| engine.stats().clone())
                        .unwrap_or_default();
                    println!(
                        "chart_stress_demo: points={} avg_declarative_render={:.1}us stage_runs(data/layout/visual/marks)={}/{}/{}/{} emitted(points/marks)={}/{}",
                        driver.points,
                        1.0,
                        stats.stage_data_runs,
                        stats.stage_layout_runs,
                        stats.stage_visual_runs,
                        stats.stage_marks_runs,
                        stats.points_emitted,
                        stats.marks_emitted
                    );
                }

                struct LegacyChartModels {
                    engine: Model<ChartEngine>,
                    spec: ChartSpec,
                }

                fn bad(app: &mut App, cx: &mut Cx, engine: ChartEngine, spec: ChartSpec) {
                    let engine = app.models_mut().insert(engine);
                    let mut props = ChartCanvasPanelProps::new(spec);
                    props.engine = Some(engine);
                    cx.observe_model(&engine);
                    let _ = ChartCanvas::new();
                    let _ = ChartCanvas::create_node();
                    let _ = create_node_retained();
                    let _ = "avg_canvas_paint";
                }

                struct ChartStressCanvas;
                impl<H: fret_ui::UiHost> Widget<H> for ChartStressCanvas {}
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/chart_stress_demo.rs",
                        "internal_harness",
                        "fixture chart stress harness",
                        owner="examples-chart-stress",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "internal-harness-chart-stress-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("ChartCanvasPanelProps", messages)
            self.assertIn("ChartCanvas::new", messages)
            self.assertIn("ChartStressCanvas", messages)
            self.assertIn("avg_canvas_paint", messages)

    def test_chart_stress_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/chart_stress_demo.rs",
                """
                use fret_chart::{ChartCanvasPanelBinding, chart_canvas_panel};
                use fret_runtime::PlatformCapabilities;
                use fret_ui::UiTree;

                pub struct ChartStressWindowState {
                    ui: UiTree<App>,
                    chart: ChartCanvasPanelBinding,
                }

                impl ChartStressDriver {
                    fn build_chart(
                        points: usize,
                        scatter_lod: Option<SeriesLodSpecV1>,
                    ) -> (ChartEngine, ChartSpec) {
                        let _ = points;
                        let _ = scatter_lod;
                        (ChartEngine::default(), ChartSpec::default())
                    }
                }

                fn create_window_state(driver: &mut ChartStressDriver, app: &mut App) -> ChartStressWindowState {
                    let (engine, spec) = ChartStressDriver::build_chart(driver.points, driver.scatter_lod);
                    let chart = ChartCanvasPanelBinding::new(app, spec, engine);
                    ChartStressWindowState { ui: UiTree::new(), chart }
                }

                fn render(
                    driver: &mut ChartStressDriver,
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut ChartStressWindowState,
                ) {
                    let chart = state.chart.clone();
                    let root = fret_ui::declarative::render_root(
                        &mut state.ui,
                        app,
                        services,
                        window,
                        bounds,
                        "chart-stress-demo-root",
                        move |cx| {
                            chart.observe_engine_paint(cx);
                            let props = chart.panel_props();
                            vec![chart_canvas_panel(cx, props)]
                        },
                    );
                    let _ = root;
                    let stats = state
                        .chart
                        .read_engine(app, |_app, engine| engine.stats().clone())
                        .unwrap_or_default();
                    println!(
                        "chart_stress_demo: points={} avg_declarative_render={:.1}us stage_runs(data/layout/visual/marks)={}/{}/{}/{} emitted(points/marks)={}/{}",
                        driver.points,
                        1.0,
                        stats.stage_data_runs,
                        stats.stage_layout_runs,
                        stats.stage_visual_runs,
                        stats.stage_marks_runs,
                        stats.points_emitted,
                        stats.marks_emitted
                    );
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                internal_harness_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/chart_stress_demo.rs",
                        "internal_harness",
                        "fixture chart stress harness",
                        owner="examples-chart-stress",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
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
                    == "internal-harness-chart-stress-declarative-binding-boundary"
                ]
            )

    def test_chart_demo_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/chart_demo.rs",
                """
                use fret_chart::{ChartCanvasPanelBinding, chart_canvas_panel};
                use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel};
                use fret_chart::retained::ChartCanvas;
                use fret_runtime::Model;
                use fret_ui::UiTree;

                struct ChartDemoWindowState {
                    ui: UiTree<App>,
                    chart: ChartCanvasPanelBinding,
                }

                impl ChartDemoDriver {
                    fn build_chart() -> (ChartEngine, ChartSpec) {
                        let dataset_id = delinea::ids::DatasetId::new(1);
                        let x_axis = AxisId::new(1);
                        let y_left_axis = AxisId::new(2);
                        let y_right_axis = AxisId::new(3);
                        let stack_id = StackId::new(1);
                        let spec = ChartSpec {
                            axes: vec![
                                delinea::AxisSpec {
                                    name: Some("Left".to_string()),
                                    kind: AxisKind::Y,
                                    scale: Default::default(),
                                    ..Default::default()
                                },
                                delinea::AxisSpec {
                                    name: Some("Right".to_string()),
                                    kind: AxisKind::Y,
                                    position: Some(AxisPosition::Right),
                                    scale: Default::default(),
                                    ..Default::default()
                                },
                                delinea::AxisSpec {
                                    id: x_axis,
                                    kind: AxisKind::X,
                                    scale: AxisScale::Time(TimeAxisScale),
                                    ..Default::default()
                                },
                            ],
                            axis_pointer: Some(delinea::AxisPointerSpec {
                                enabled: true,
                                ..Default::default()
                            }),
                            series: vec![
                                SeriesSpec {
                                    name: Some("Stack A (area)".to_string()),
                                    kind: SeriesKind::Area,
                                    y_axis: y_left_axis,
                                    stack: Some(stack_id),
                                    area_baseline: Some(AreaBaseline::Zero),
                                    ..Default::default()
                                },
                                SeriesSpec {
                                    name: Some("Stack B (area)".to_string()),
                                    kind: SeriesKind::Area,
                                    y_axis: y_left_axis,
                                    stack: Some(stack_id),
                                    area_baseline: Some(AreaBaseline::Zero),
                                    ..Default::default()
                                },
                                SeriesSpec {
                                    name: Some("Right axis (line)".to_string()),
                                    kind: SeriesKind::Line,
                                    y_axis: y_right_axis,
                                    ..Default::default()
                                },
                            ],
                            ..Default::default()
                        };
                        let mut engine = ChartEngine::new(spec.clone()).expect("chart spec should be valid");
                        let mut table = DataTable::default();
                        engine.datasets_mut().insert(dataset_id, table);
                        (engine, spec)
                    }

                    fn build_ui(app: &mut App) -> ChartDemoWindowState {
                        let (engine, spec) = Self::build_chart();
                        let chart = ChartCanvasPanelBinding::new(app, spec, engine);
                        ChartDemoWindowState { ui: UiTree::new(), chart }
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut ChartDemoWindowState,
                ) {
                    let chart = state.chart.clone();
                    let root = fret_ui::declarative::render_root(
                        &mut state.ui,
                        app,
                        services,
                        window,
                        bounds,
                        "chart-demo-root",
                        move |cx| {
                            chart.observe_engine_paint(cx);
                            let props = chart.panel_props();
                            vec![chart_canvas_panel(cx, props)]
                        },
                    );
                    let _ = root;
                }

                struct LegacyChartDemoCanvas {
                    engine: Model<ChartEngine>,
                    output: Model<ChartCanvasOutput>,
                }

                fn bad(app: &mut App, cx: &mut Cx, engine: ChartEngine, spec: ChartSpec) {
                    let engine = app.models_mut().insert(engine);
                    let output = app.models_mut().insert(ChartCanvasOutput::default());
                    let _other = app.models_mut().insert(ChartEngine::default());
                    let mut props = ChartCanvasPanelProps::new(spec).output_model(output);
                    props.engine = Some(engine);
                    cx.observe_model(&engine);
                    let _ = ChartCanvas::new();
                    let _ = ChartCanvas::create_node();
                    create_node_retained();
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/chart_demo.rs",
                        "advanced_manual",
                        "fixture chart demo",
                        owner="examples-chart-demo",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-chart-demo-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("ChartCanvasPanelProps", messages)
            self.assertIn("ChartCanvas::new", messages)
            self.assertIn("ChartCanvas::create_node", messages)
            self.assertIn("ChartCanvasOutput", messages)

    def test_chart_demo_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/chart_demo.rs",
                """
                use fret_chart::{ChartCanvasPanelBinding, chart_canvas_panel};
                use fret_ui::UiTree;

                struct ChartDemoWindowState {
                    ui: UiTree<App>,
                    chart: ChartCanvasPanelBinding,
                }

                impl ChartDemoDriver {
                    fn build_chart() -> (ChartEngine, ChartSpec) {
                        let dataset_id = delinea::ids::DatasetId::new(1);
                        let x_axis = AxisId::new(1);
                        let y_left_axis = AxisId::new(2);
                        let y_right_axis = AxisId::new(3);
                        let stack_id = StackId::new(1);
                        let spec = ChartSpec {
                            axes: vec![
                                delinea::AxisSpec {
                                    name: Some("Left".to_string()),
                                    kind: AxisKind::Y,
                                    scale: Default::default(),
                                    ..Default::default()
                                },
                                delinea::AxisSpec {
                                    name: Some("Right".to_string()),
                                    kind: AxisKind::Y,
                                    position: Some(AxisPosition::Right),
                                    scale: Default::default(),
                                    ..Default::default()
                                },
                                delinea::AxisSpec {
                                    id: x_axis,
                                    kind: AxisKind::X,
                                    scale: AxisScale::Time(TimeAxisScale),
                                    ..Default::default()
                                },
                            ],
                            axis_pointer: Some(delinea::AxisPointerSpec {
                                enabled: true,
                                ..Default::default()
                            }),
                            series: vec![
                                SeriesSpec {
                                    name: Some("Stack A (area)".to_string()),
                                    kind: SeriesKind::Area,
                                    y_axis: y_left_axis,
                                    stack: Some(stack_id),
                                    area_baseline: Some(AreaBaseline::Zero),
                                    ..Default::default()
                                },
                                SeriesSpec {
                                    name: Some("Stack B (area)".to_string()),
                                    kind: SeriesKind::Area,
                                    y_axis: y_left_axis,
                                    stack: Some(stack_id),
                                    area_baseline: Some(AreaBaseline::Zero),
                                    ..Default::default()
                                },
                                SeriesSpec {
                                    name: Some("Right axis (line)".to_string()),
                                    kind: SeriesKind::Line,
                                    y_axis: y_right_axis,
                                    ..Default::default()
                                },
                            ],
                            ..Default::default()
                        };
                        let mut engine = ChartEngine::new(spec.clone()).expect("chart spec should be valid");
                        let mut table = DataTable::default();
                        engine.datasets_mut().insert(dataset_id, table);
                        (engine, spec)
                    }

                    fn build_ui(app: &mut App) -> ChartDemoWindowState {
                        let (engine, spec) = Self::build_chart();
                        let chart = ChartCanvasPanelBinding::new(app, spec, engine);
                        ChartDemoWindowState { ui: UiTree::new(), chart }
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut ChartDemoWindowState,
                ) {
                    let chart = state.chart.clone();
                    let root = fret_ui::declarative::render_root(
                        &mut state.ui,
                        app,
                        services,
                        window,
                        bounds,
                        "chart-demo-root",
                        move |cx| {
                            chart.observe_engine_paint(cx);
                            let props = chart.panel_props();
                            vec![chart_canvas_panel(cx, props)]
                        },
                    );
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/chart_demo.rs",
                        "advanced_manual",
                        "fixture chart demo",
                        owner="examples-chart-demo",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-chart-demo-declarative-binding-boundary"
                ]
            )

    def test_chart_bars_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/bars_demo.rs",
                """
                use fret_chart::{ChartCanvasPanelBinding, chart_canvas_panel};
                use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel};
                use fret_chart::retained::ChartCanvas;
                use fret_runtime::Model;
                use fret_ui::{UiTree, declarative};

                struct BarsDemoWindowState {
                    ui: UiTree<App>,
                    chart: ChartCanvasPanelBinding,
                }

                impl BarsDemoDriver {
                    fn build_chart() -> (ChartEngine, ChartSpec) {
                        (ChartEngine::default(), ChartSpec::default())
                    }

                    fn build_ui(app: &mut App) -> BarsDemoWindowState {
                        let (engine, spec) = Self::build_chart();
                        let chart = ChartCanvasPanelBinding::new(app, spec, engine);
                        BarsDemoWindowState { ui: UiTree::new(), chart }
                    }
                }

                fn handle_event(app: &mut App, state: &mut BarsDemoWindowState) {
                    let _ = state.chart.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut BarsDemoWindowState,
                ) {
                    let chart = state.chart.clone();
                    let root = declarative::render_root(
                        &mut state.ui,
                        app,
                        services,
                        window,
                        bounds,
                        "bars-demo-root",
                        move |cx| {
                            chart.observe_engine_paint(cx);
                            let props = chart.panel_props();
                            vec![chart_canvas_panel(cx, props)]
                        },
                    );
                    let _ = root;
                }

                struct LegacyBarsChart {
                    engine: Model<ChartEngine>,
                    output: Model<ChartCanvasOutput>,
                }

                fn bad(app: &mut App, cx: &mut Cx, engine: ChartEngine, spec: ChartSpec) {
                    let engine = app.models_mut().insert(engine);
                    let output = app.models_mut().insert(ChartCanvasOutput::default());
                    let _other = app.models_mut().insert(ChartEngine::default());
                    let mut props = ChartCanvasPanelProps::new(spec).output_model(output);
                    props.engine = Some(engine);
                    cx.observe_model(&engine);
                    let _ = ChartCanvas::new();
                    let _ = ChartCanvas::create_node();
                    create_node_retained();
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/bars_demo.rs",
                        "advanced_manual",
                        "fixture bars chart demo",
                        owner="examples-chart-bars",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "advanced-surface-chart-bars-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("ChartCanvasPanelProps", messages)
            self.assertIn("ChartCanvas::new", messages)
            self.assertIn("ChartCanvas::create_node", messages)
            self.assertIn("ChartCanvasOutput", messages)

    def test_chart_bars_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/bars_demo.rs",
                """
                use fret_chart::{ChartCanvasPanelBinding, chart_canvas_panel};
                use fret_ui::{UiTree, declarative};

                struct BarsDemoWindowState {
                    ui: UiTree<App>,
                    chart: ChartCanvasPanelBinding,
                }

                impl BarsDemoDriver {
                    fn build_chart() -> (ChartEngine, ChartSpec) {
                        (ChartEngine::default(), ChartSpec::default())
                    }

                    fn build_ui(app: &mut App) -> BarsDemoWindowState {
                        let (engine, spec) = Self::build_chart();
                        let chart = ChartCanvasPanelBinding::new(app, spec, engine);
                        BarsDemoWindowState { ui: UiTree::new(), chart }
                    }
                }

                fn handle_event(app: &mut App, state: &mut BarsDemoWindowState) {
                    let _ = state.chart.output_untracked(app);
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut BarsDemoWindowState,
                ) {
                    let chart = state.chart.clone();
                    let root = declarative::render_root(
                        &mut state.ui,
                        app,
                        services,
                        window,
                        bounds,
                        "bars-demo-root",
                        move |cx| {
                            chart.observe_engine_paint(cx);
                            let props = chart.panel_props();
                            vec![chart_canvas_panel(cx, props)]
                        },
                    );
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/bars_demo.rs",
                        "advanced_manual",
                        "fixture bars chart demo",
                        owner="examples-chart-bars",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-chart-bars-declarative-binding-boundary"
                ]
            )

    def test_chart_category_line_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/category_line_demo.rs",
                """
                use fret_chart::{ChartCanvasPanelBinding, chart_canvas_panel};
                use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel};
                use fret_chart::retained::ChartCanvas;
                use fret_runtime::Model;
                use fret_ui::UiTree;

                struct CategoryLineDemoWindowState {
                    ui: UiTree<App>,
                    chart: ChartCanvasPanelBinding,
                }

                impl CategoryLineDemoDriver {
                    fn build_chart() -> (ChartEngine, ChartSpec) {
                        let categories = vec!["A".to_string(), "B".to_string()];
                        let spec = ChartSpec {
                            axes: vec![delinea::AxisSpec {
                                scale: AxisScale::Category(delinea::CategoryAxisScale { categories }),
                                ..Default::default()
                            }],
                            data_zoom_x: vec![DataZoomXSpec {
                                id: zoom_id,
                                axis: x_axis,
                                filter_mode: FilterMode::Filter,
                                min_value_span: Some(6.0),
                                max_value_span: Some(80.0),
                            }],
                            ..Default::default()
                        };
                        let mut engine = ChartEngine::new(spec.clone()).expect("chart spec should be valid");
                        engine.apply_action(Action::SetDataWindowX {
                            axis: x_axis,
                            window: Some(DataWindow {
                                min: 16.0,
                                max: 64.0,
                            }),
                        });
                        (engine, spec)
                    }

                    fn build_ui(app: &mut App) -> CategoryLineDemoWindowState {
                        let (engine, spec) = Self::build_chart();
                        let chart = ChartCanvasPanelBinding::new(app, spec, engine);
                        CategoryLineDemoWindowState { ui: UiTree::new(), chart }
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut CategoryLineDemoWindowState,
                ) {
                    let chart = state.chart.clone();
                    let root = fret_ui::declarative::render_root(
                        &mut state.ui,
                        app,
                        services,
                        window,
                        bounds,
                        "category-line-demo-root",
                        move |cx| {
                            chart.observe_engine_paint(cx);
                            let props = chart.panel_props();
                            vec![chart_canvas_panel(cx, props)]
                        },
                    );
                    let _ = root;
                }

                struct LegacyCategoryLineChart {
                    engine: Model<ChartEngine>,
                    output: Model<ChartCanvasOutput>,
                }

                fn bad(app: &mut App, cx: &mut Cx, engine: ChartEngine, spec: ChartSpec) {
                    let engine = app.models_mut().insert(engine);
                    let output = app.models_mut().insert(ChartCanvasOutput::default());
                    let _other = app.models_mut().insert(ChartEngine::default());
                    let mut props = ChartCanvasPanelProps::new(spec).output_model(output);
                    props.engine = Some(engine);
                    cx.observe_model(&engine);
                    let _ = ChartCanvas::new();
                    let _ = ChartCanvas::create_node();
                    create_node_retained();
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/category_line_demo.rs",
                        "advanced_manual",
                        "fixture category-line chart demo",
                        owner="examples-chart-category-line",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "advanced-surface-chart-category-line-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("ChartCanvasPanelProps", messages)
            self.assertIn("ChartCanvas::new", messages)
            self.assertIn("ChartCanvas::create_node", messages)
            self.assertIn("ChartCanvasOutput", messages)

    def test_chart_category_line_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/category_line_demo.rs",
                """
                use fret_chart::{ChartCanvasPanelBinding, chart_canvas_panel};
                use fret_ui::UiTree;

                struct CategoryLineDemoWindowState {
                    ui: UiTree<App>,
                    chart: ChartCanvasPanelBinding,
                }

                impl CategoryLineDemoDriver {
                    fn build_chart() -> (ChartEngine, ChartSpec) {
                        let categories = vec!["A".to_string(), "B".to_string()];
                        let spec = ChartSpec {
                            axes: vec![delinea::AxisSpec {
                                scale: AxisScale::Category(delinea::CategoryAxisScale { categories }),
                                ..Default::default()
                            }],
                            data_zoom_x: vec![DataZoomXSpec {
                                id: zoom_id,
                                axis: x_axis,
                                filter_mode: FilterMode::Filter,
                                min_value_span: Some(6.0),
                                max_value_span: Some(80.0),
                            }],
                            ..Default::default()
                        };
                        let mut engine = ChartEngine::new(spec.clone()).expect("chart spec should be valid");
                        engine.apply_action(Action::SetDataWindowX {
                            axis: x_axis,
                            window: Some(DataWindow {
                                min: 16.0,
                                max: 64.0,
                            }),
                        });
                        (engine, spec)
                    }

                    fn build_ui(app: &mut App) -> CategoryLineDemoWindowState {
                        let (engine, spec) = Self::build_chart();
                        let chart = ChartCanvasPanelBinding::new(app, spec, engine);
                        CategoryLineDemoWindowState { ui: UiTree::new(), chart }
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut CategoryLineDemoWindowState,
                ) {
                    let chart = state.chart.clone();
                    let root = fret_ui::declarative::render_root(
                        &mut state.ui,
                        app,
                        services,
                        window,
                        bounds,
                        "category-line-demo-root",
                        move |cx| {
                            chart.observe_engine_paint(cx);
                            let props = chart.panel_props();
                            vec![chart_canvas_panel(cx, props)]
                        },
                    );
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/category_line_demo.rs",
                        "advanced_manual",
                        "fixture category-line chart demo",
                        owner="examples-chart-category-line",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-chart-category-line-declarative-binding-boundary"
                ]
            )

    def test_chart_horizontal_bars_legacy_retained_authoring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/horizontal_bars_demo.rs",
                """
                use fret_chart::{ChartCanvasPanelBinding, chart_canvas_panel};
                use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel};
                use fret_chart::retained::ChartCanvas;
                use fret_runtime::Model;
                use fret_ui::UiTree;

                struct HorizontalBarsDemoWindowState {
                    ui: UiTree<App>,
                    chart: ChartCanvasPanelBinding,
                }

                impl HorizontalBarsDemoDriver {
                    fn build_chart() -> (ChartEngine, ChartSpec) {
                        let categories = vec!["A".to_string(), "B".to_string()];
                        let spec = ChartSpec {
                            axes: vec![delinea::AxisSpec {
                                scale: AxisScale::Category(delinea::CategoryAxisScale { categories }),
                                ..Default::default()
                            }],
                            visual_maps: vec![VisualMapSpec {
                                id: VisualMapId::new(1),
                                mode: VisualMapMode::Continuous,
                                dataset: None,
                                series: vec![series_c_id],
                                field: x_c_field,
                                domain: (-80.0, 80.0),
                                initial_range: Some((-20.0, 20.0)),
                                initial_piece_mask: None,
                                point_radius_mul_range: None,
                                stroke_width_range: None,
                                opacity_mul_range: Some((0.2, 1.0)),
                                buckets: 8,
                                out_of_range_opacity: 0.25,
                            }],
                            series: vec![SeriesSpec {
                                stack: Some(stack_id),
                                ..Default::default()
                            }],
                            ..Default::default()
                        };
                        let engine = ChartEngine::new(spec.clone()).expect("chart spec should be valid");
                        (engine, spec)
                    }

                    fn build_ui(app: &mut App) -> HorizontalBarsDemoWindowState {
                        let (engine, spec) = Self::build_chart();
                        let chart = ChartCanvasPanelBinding::new(app, spec, engine);
                        HorizontalBarsDemoWindowState { ui: UiTree::new(), chart }
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut HorizontalBarsDemoWindowState,
                ) {
                    let chart = state.chart.clone();
                    let root = fret_ui::declarative::render_root(
                        &mut state.ui,
                        app,
                        services,
                        window,
                        bounds,
                        "horizontal-bars-demo-root",
                        move |cx| {
                            chart.observe_engine_paint(cx);
                            let props = chart.panel_props();
                            vec![chart_canvas_panel(cx, props)]
                        },
                    );
                    let _ = root;
                }

                struct LegacyHorizontalBarsChart {
                    engine: Model<ChartEngine>,
                    output: Model<ChartCanvasOutput>,
                }

                fn bad(app: &mut App, cx: &mut Cx, engine: ChartEngine, spec: ChartSpec) {
                    let engine = app.models_mut().insert(engine);
                    let output = app.models_mut().insert(ChartCanvasOutput::default());
                    let _other = app.models_mut().insert(ChartEngine::default());
                    let mut props = ChartCanvasPanelProps::new(spec).output_model(output);
                    props.engine = Some(engine);
                    cx.observe_model(&engine);
                    let _ = ChartCanvas::new();
                    let _ = ChartCanvas::create_node();
                    create_node_retained();
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/horizontal_bars_demo.rs",
                        "advanced_manual",
                        "fixture horizontal-bars chart demo",
                        owner="examples-chart-horizontal-bars",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule
                == "advanced-surface-chart-horizontal-bars-declarative-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("ChartCanvasPanelProps", messages)
            self.assertIn("ChartCanvas::new", messages)
            self.assertIn("ChartCanvas::create_node", messages)
            self.assertIn("ChartCanvasOutput", messages)

    def test_chart_horizontal_bars_declarative_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/horizontal_bars_demo.rs",
                """
                use fret_chart::{ChartCanvasPanelBinding, chart_canvas_panel};
                use fret_ui::UiTree;

                struct HorizontalBarsDemoWindowState {
                    ui: UiTree<App>,
                    chart: ChartCanvasPanelBinding,
                }

                impl HorizontalBarsDemoDriver {
                    fn build_chart() -> (ChartEngine, ChartSpec) {
                        let categories = vec!["A".to_string(), "B".to_string()];
                        let spec = ChartSpec {
                            axes: vec![delinea::AxisSpec {
                                scale: AxisScale::Category(delinea::CategoryAxisScale { categories }),
                                ..Default::default()
                            }],
                            visual_maps: vec![VisualMapSpec {
                                id: VisualMapId::new(1),
                                mode: VisualMapMode::Continuous,
                                dataset: None,
                                series: vec![series_c_id],
                                field: x_c_field,
                                domain: (-80.0, 80.0),
                                initial_range: Some((-20.0, 20.0)),
                                initial_piece_mask: None,
                                point_radius_mul_range: None,
                                stroke_width_range: None,
                                opacity_mul_range: Some((0.2, 1.0)),
                                buckets: 8,
                                out_of_range_opacity: 0.25,
                            }],
                            series: vec![SeriesSpec {
                                stack: Some(stack_id),
                                ..Default::default()
                            }],
                            ..Default::default()
                        };
                        let engine = ChartEngine::new(spec.clone()).expect("chart spec should be valid");
                        (engine, spec)
                    }

                    fn build_ui(app: &mut App) -> HorizontalBarsDemoWindowState {
                        let (engine, spec) = Self::build_chart();
                        let chart = ChartCanvasPanelBinding::new(app, spec, engine);
                        HorizontalBarsDemoWindowState { ui: UiTree::new(), chart }
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut HorizontalBarsDemoWindowState,
                ) {
                    let chart = state.chart.clone();
                    let root = fret_ui::declarative::render_root(
                        &mut state.ui,
                        app,
                        services,
                        window,
                        bounds,
                        "horizontal-bars-demo-root",
                        move |cx| {
                            chart.observe_engine_paint(cx);
                            let props = chart.panel_props();
                            vec![chart_canvas_panel(cx, props)]
                        },
                    );
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/horizontal_bars_demo.rs",
                        "advanced_manual",
                        "fixture horizontal-bars chart demo",
                        owner="examples-chart-horizontal-bars",
                        allowed_raw_seams=("fret_ui", "UiTree"),
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
                    == "advanced-surface-chart-horizontal-bars-declarative-binding-boundary"
                ]
            )

    def test_echarts_adapter_raw_chart_and_text_wiring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/echarts_demo.rs",
                """
                use fret::advanced::text;
                use fret_chart::{ChartCanvasPanelBinding, chart_canvas_panel};
                use fret_runtime::Model;
                use fret_ui::{ElementContext, Invalidation};
                use fret_ui_kit::declarative::text as decl_text;

                struct EchartsDemoChart {
                    title: std::sync::Arc<str>,
                    chart: ChartCanvasPanelBinding,
                }

                struct EchartsDemoState {
                    charts: Vec<EchartsDemoChart>,
                }

                fn init_window(app: &mut KernelApp) -> EchartsDemoState {
                    let (engine_basic, spec_basic) = build_chart();
                    let (engine_percent, spec_percent) = build_chart();
                    EchartsDemoState {
                        charts: vec![
                            EchartsDemoChart {
                                title: "basic".into(),
                                chart: ChartCanvasPanelBinding::new(app, spec_basic, engine_basic),
                            },
                            EchartsDemoChart {
                                title: "percent".into(),
                                chart: ChartCanvasPanelBinding::new(app, spec_percent, engine_percent),
                            },
                        ],
                    }
                }

                fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut EchartsDemoState) -> ViewElements {
                    for chart in &st.charts {
                        chart.chart.observe_engine_paint(cx);
                    }

                    let mut out = Vec::new();
                    for chart in &st.charts {
                        out.push(text::section_chrome_label(
                            cx,
                            std::sync::Arc::clone(&chart.title),
                        ));
                        let props = chart.chart.panel_props();
                        out.push(chart_canvas_panel(cx, props));
                    }
                    out.into()
                }

                struct LegacyChart {
                    engine: Model<ChartEngine>,
                    spec: ChartSpec,
                }

                fn bad(
                    app: &mut KernelApp,
                    cx: &mut ElementContext<'_, KernelApp>,
                    chart: &LegacyChart,
                    engine_basic: ChartEngine,
                    engine_percent: ChartEngine,
                ) {
                    let _ = cx.text(std::sync::Arc::clone(&chart.title));
                    let _ = decl_text::body(cx, "bad");
                    let _ = app.models_mut().insert(engine_basic);
                    let _ = app.models_mut().insert(engine_percent);
                    cx.observe_model(&chart.engine);
                    let mut props = ChartCanvasPanelProps::new(chart.spec.clone());
                    props.engine = Some(chart.engine.clone());
                    let _ = Invalidation;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                comparison_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/echarts_demo.rs",
                        "comparison",
                        "fixture ECharts adapter surface",
                        owner="examples-echarts-adapter",
                        allowed_raw_seams=(
                            "fret::advanced",
                            "fret_ui",
                            "AnyElement",
                            "ElementContext",
                        ),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "comparison-surface-echarts-adapter-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 6)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("cx.text", messages)
            self.assertIn("decl_text", messages)
            self.assertIn("Model<ChartEngine>", messages)
            self.assertIn("ChartCanvasPanelProps", messages)

    def test_echarts_adapter_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/echarts_demo.rs",
                """
                use fret::advanced::text;
                use fret_chart::{ChartCanvasPanelBinding, chart_canvas_panel};
                use fret_ui::ElementContext;

                struct EchartsDemoChart {
                    title: std::sync::Arc<str>,
                    chart: ChartCanvasPanelBinding,
                }

                struct EchartsDemoState {
                    charts: Vec<EchartsDemoChart>,
                }

                fn init_window(app: &mut KernelApp) -> EchartsDemoState {
                    let (engine_basic, spec_basic) = build_chart();
                    let (engine_percent, spec_percent) = build_chart();
                    EchartsDemoState {
                        charts: vec![
                            EchartsDemoChart {
                                title: "basic".into(),
                                chart: ChartCanvasPanelBinding::new(app, spec_basic, engine_basic),
                            },
                            EchartsDemoChart {
                                title: "percent".into(),
                                chart: ChartCanvasPanelBinding::new(app, spec_percent, engine_percent),
                            },
                        ],
                    }
                }

                fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut EchartsDemoState) -> ViewElements {
                    for chart in &st.charts {
                        chart.chart.observe_engine_paint(cx);
                    }

                    let mut out = Vec::new();
                    for chart in &st.charts {
                        out.push(text::section_chrome_label(
                            cx,
                            std::sync::Arc::clone(&chart.title),
                        ));
                        let props = chart.chart.panel_props();
                        out.push(chart_canvas_panel(cx, props));
                    }
                    out.into()
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                comparison_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/echarts_demo.rs",
                        "comparison",
                        "fixture ECharts adapter surface",
                        owner="examples-echarts-adapter",
                        allowed_raw_seams=(
                            "fret::advanced",
                            "fret_ui",
                            "AnyElement",
                            "ElementContext",
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
                    if violation.rule == "comparison-surface-echarts-adapter-binding-boundary"
                ]
            )

    def test_chart_multi_axis_retained_linked_wiring_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/chart_multi_axis_demo.rs",
                """
                use fret_chart::{
                    ChartCanvasLinkedGroupBinding,
                    ChartCanvasLinkedPanelBinding,
                    ChartCanvasLinkedStateBinding,
                    ChartLinkPolicy,
                    ChartLinkRouter,
                    LinkAxisKey,
                    chart_canvas_panel,
                };
                use fret_chart::{
                    AxisPointerLinkAnchor,
                    BrushSelectionLink2D,
                    ChartCanvasOutput,
                    ChartCanvasPanelProps,
                    ChartLinkPolicy,
                    ChartLinkRouter,
                    LinkAxisKey,
                    LinkedChartGroup,
                    LinkedChartMember,
                    chart_canvas_panel,
                };
                use fret_chart::retained::ChartCanvas;
                use fret_runtime::Model;
                use fret_ui::{AnyElement, ElementContext, UiTree};

                struct ChartMultiAxisDemoDiagnosticsHandles {
                    shared_state: ChartCanvasLinkedStateBinding,
                    top_chart: ChartCanvasLinkedPanelBinding,
                    bottom_chart: ChartCanvasLinkedPanelBinding,
                }

                pub struct ChartMultiAxisDemoWindowState {
                    ui: UiTree<App>,
                    linked: ChartCanvasLinkedGroupBinding,
                    top_chart: ChartCanvasLinkedPanelBinding,
                    bottom_chart: ChartCanvasLinkedPanelBinding,
                }

                impl ChartMultiAxisDemoDriver {
                    fn build_ui(app: &mut App) -> ChartMultiAxisDemoWindowState {
                        let (top_engine, top_spec, top_router) =
                            ChartMultiAxisDemoDriver::build_chart(delinea::ids::ChartId::new(1));
                        let (bottom_engine, bottom_spec, bottom_router) =
                            ChartMultiAxisDemoDriver::build_chart(delinea::ids::ChartId::new(2));
                        let mut linked = ChartCanvasLinkedGroupBinding::new(
                            app,
                            ChartLinkPolicy {
                                brush: true,
                                axis_pointer: true,
                                domain_windows: true,
                            },
                        );
                        let top_chart = linked.push_panel(app, top_spec, top_engine, top_router);
                        let bottom_chart =
                            linked.push_panel(app, bottom_spec, bottom_engine, bottom_router);

                        ChartMultiAxisDemoWindowState {
                            ui: UiTree::new(),
                            linked,
                            top_chart,
                            bottom_chart,
                        }
                    }

                    fn build_chart(
                        chart_id: delinea::ids::ChartId
                    ) -> (ChartEngine, ChartSpec, ChartLinkRouter) {
                        todo!()
                    }

                    fn chart_panel(
                        cx: &mut ElementContext<'_, App>,
                        chart: ChartCanvasLinkedPanelBinding,
                        test_id: &'static str,
                    ) -> AnyElement {
                        chart_canvas_panel(cx, chart.panel_props_with_test_id(test_id))
                    }

                    fn snapshot(app: &App, handles: &ChartMultiAxisDemoDiagnosticsHandles) {
                        let _ = handles.shared_state.domain_windows_untracked(app);
                        let _ = handles.top_chart.output_untracked(app);
                    }

                    fn diagnostics(app: &mut App, state: &mut ChartMultiAxisDemoWindowState) {
                        let _ = state.top_chart.output_untracked(app);
                        let _ = state.linked.domain_windows_untracked(app);
                        let _ = state.top_chart.read_engine(app, |_app, engine| engine.stats());
                        let _ = state.top_chart.update_engine(app, |engine, _cx| {
                            engine.apply_action(Action::Noop);
                        });
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut ChartMultiAxisDemoWindowState,
                ) {
                    let top_chart = state.top_chart.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("chart-multi-axis-demo", move |cx| {
                            vec![ChartMultiAxisDemoDriver::chart_panel(
                                cx,
                                top_chart.clone(),
                                "chart-multi-axis-top",
                            )]
                        });
                    let _ = root;
                }

                struct LegacyLinkedCharts {
                    top_engine: Model<ChartEngine>,
                    bottom_engine: Model<ChartEngine>,
                    top_spec: ChartSpec,
                    bottom_spec: ChartSpec,
                    linked: LinkedChartGroup,
                    top_output: Model<ChartCanvasOutput>,
                    bottom_output: Model<ChartCanvasOutput>,
                }

                fn bad(
                    spec: ChartSpec,
                    engine: Model<ChartEngine>,
                    output: Model<ChartCanvasOutput>,
                    shared_brush: BrushSelectionLink2D,
                    shared_axis_pointer: AxisPointerLinkAnchor,
                    shared_domain_windows: LinkAxisKey,
                ) {
                    let _ = ChartCanvasOutput::default();
                    let mut props = ChartCanvasPanelProps::new(spec)
                        .output_model(output)
                        .linked_brush(shared_brush)
                        .linked_axis_pointer(shared_axis_pointer)
                        .linked_domain_windows(shared_domain_windows);
                    props.engine = Some(engine);
                    let _ = ChartCanvas::new();
                    let _ = ChartCanvas::new_shared();
                    let _ = ChartCanvas::create_node();
                    let _ = FixedSplit::create_node_with_children();
                    let _engine_cell: Rc<RefCell<ChartEngine>> = todo!();
                    let _other: std::rc::Rc<std::cell::RefCell<ChartEngine>> = todo!();
                    create_node_retained();
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/chart_multi_axis_demo.rs",
                        "advanced_manual",
                        "fixture chart multi-axis surface",
                        owner="examples-chart-multi-axis",
                        allowed_raw_seams=(
                            "fret_runtime",
                            "fret_ui",
                            "AnyElement",
                            "ElementContext",
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
                if violation.rule
                == "advanced-surface-chart-multi-axis-linked-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 10)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("LinkedChartGroup", messages)
            self.assertIn("Model<ChartEngine>", messages)
            self.assertIn("ChartCanvasPanelProps", messages)
            self.assertIn("ChartCanvas::new_shared", messages)
            self.assertIn("FixedSplit::create_node_with_children", messages)

    def test_chart_multi_axis_linked_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/chart_multi_axis_demo.rs",
                """
                use fret_chart::{
                    ChartCanvasLinkedGroupBinding,
                    ChartCanvasLinkedPanelBinding,
                    ChartCanvasLinkedStateBinding,
                    ChartLinkPolicy,
                    ChartLinkRouter,
                    LinkAxisKey,
                    chart_canvas_panel,
                };
                use fret_ui::{AnyElement, ElementContext, UiTree};

                struct ChartMultiAxisDemoDiagnosticsHandles {
                    shared_state: ChartCanvasLinkedStateBinding,
                    top_chart: ChartCanvasLinkedPanelBinding,
                    bottom_chart: ChartCanvasLinkedPanelBinding,
                }

                pub struct ChartMultiAxisDemoWindowState {
                    ui: UiTree<App>,
                    linked: ChartCanvasLinkedGroupBinding,
                    top_chart: ChartCanvasLinkedPanelBinding,
                    bottom_chart: ChartCanvasLinkedPanelBinding,
                }

                impl ChartMultiAxisDemoDriver {
                    fn build_ui(app: &mut App) -> ChartMultiAxisDemoWindowState {
                        let (top_engine, top_spec, top_router) =
                            ChartMultiAxisDemoDriver::build_chart(delinea::ids::ChartId::new(1));
                        let (bottom_engine, bottom_spec, bottom_router) =
                            ChartMultiAxisDemoDriver::build_chart(delinea::ids::ChartId::new(2));
                        let mut linked = ChartCanvasLinkedGroupBinding::new(
                            app,
                            ChartLinkPolicy {
                                brush: true,
                                axis_pointer: true,
                                domain_windows: true,
                            },
                        );
                        let top_chart = linked.push_panel(app, top_spec, top_engine, top_router);
                        let bottom_chart =
                            linked.push_panel(app, bottom_spec, bottom_engine, bottom_router);

                        ChartMultiAxisDemoWindowState {
                            ui: UiTree::new(),
                            linked,
                            top_chart,
                            bottom_chart,
                        }
                    }

                    fn build_chart(
                        chart_id: delinea::ids::ChartId
                    ) -> (ChartEngine, ChartSpec, ChartLinkRouter) {
                        todo!()
                    }

                    fn chart_panel(
                        cx: &mut ElementContext<'_, App>,
                        chart: ChartCanvasLinkedPanelBinding,
                        test_id: &'static str,
                    ) -> AnyElement {
                        chart_canvas_panel(cx, chart.panel_props_with_test_id(test_id))
                    }

                    fn snapshot(app: &App, handles: &ChartMultiAxisDemoDiagnosticsHandles) {
                        let _ = handles.shared_state.domain_windows_untracked(app);
                        let _ = handles.top_chart.output_untracked(app);
                    }

                    fn diagnostics(app: &mut App, state: &mut ChartMultiAxisDemoWindowState) {
                        let _ = state.top_chart.output_untracked(app);
                        let _ = state.linked.domain_windows_untracked(app);
                        let _ = state.top_chart.read_engine(app, |_app, engine| engine.stats());
                        let _ = state.top_chart.update_engine(app, |engine, _cx| {
                            engine.apply_action(Action::Noop);
                        });
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut ChartMultiAxisDemoWindowState,
                ) {
                    let top_chart = state.top_chart.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("chart-multi-axis-demo", move |cx| {
                            vec![ChartMultiAxisDemoDriver::chart_panel(
                                cx,
                                top_chart.clone(),
                                "chart-multi-axis-top",
                            )]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/chart_multi_axis_demo.rs",
                        "advanced_manual",
                        "fixture chart multi-axis surface",
                        owner="examples-chart-multi-axis",
                        allowed_raw_seams=(
                            "fret_ui",
                            "AnyElement",
                            "ElementContext",
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
                    == "advanced-surface-chart-multi-axis-linked-binding-boundary"
                ]
            )

    def test_echarts_multi_grid_retained_helpers_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/echarts_multi_grid_demo.rs",
                """
                use fret_chart::{ChartCanvasMultiGridBinding, chart_canvas_panel};
                use fret_chart::retained::{UniformGrid, create_multi_grid_chart_canvas_nodes};
                use fret_runtime::Model;
                use fret_ui::UiTree;

                pub struct EchartsMultiGridDemoWindowState {
                    ui: UiTree<App>,
                    chart: ChartCanvasMultiGridBinding,
                }

                impl EchartsMultiGridDemoDriver {
                    fn build_ui(app: &mut App, window: AppWindowId) -> EchartsMultiGridDemoWindowState {
                        let (engine, spec, grids) = Self::build_chart();
                        EchartsMultiGridDemoWindowState {
                            ui: UiTree::new(),
                            chart: ChartCanvasMultiGridBinding::new(app, spec, engine, grids),
                        }
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut EchartsMultiGridDemoWindowState,
                ) {
                    let chart = state.chart.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("echarts-multi-grid-demo", move |cx| {
                            chart.observe_engine_paint(cx);
                            let mut overlay_props = chart.overlay_panel_props();
                            let grid_views = cx.container(container, move |cx| {
                                chart
                                    .grids()
                                    .iter()
                                    .copied()
                                    .map(|grid| {
                                        let props = chart.grid_panel_props(grid);
                                        chart_canvas_panel(cx, props)
                                    })
                                    .collect::<Vec<_>>()
                            });
                            vec![grid_views, chart_canvas_panel(cx, overlay_props)]
                        });
                    let _ = root;
                }

                struct LegacyMultiGridChart {
                    engine: Model<ChartEngine>,
                    spec: ChartSpec,
                }

                fn bad(app: &mut App, grid: GridId, spec: ChartSpec, engine: ChartEngine) {
                    let _ = create_multi_grid_chart_canvas_nodes();
                    let _ = UniformGrid;
                    let _ = ChartCanvas::new_grid_view();
                    let _ = ChartCanvas::new_overlay();
                    let _ = ChartCanvas::create_node();
                    let _ = create_node_retained();
                    let mut props = ChartCanvasPanelProps::new(spec).grid_view(grid);
                    props.engine = Some(engine);
                    let mut overlay_props = ChartCanvasPanelProps::new(spec.clone()).overlay_only();
                    overlay_props.engine = Some(engine.clone());
                    let _ = app.models_mut().insert(engine);
                    let _engine_cell: Rc<RefCell<ChartEngine>> = todo!();
                    let _other: std::rc::Rc<std::cell::RefCell<ChartEngine>> = todo!();
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/echarts_multi_grid_demo.rs",
                        "advanced_manual",
                        "fixture ECharts multi-grid surface",
                        owner="examples-echarts-multi-grid",
                        allowed_raw_seams=("fret_runtime", "fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "advanced-surface-echarts-multi-grid-binding-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 8)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("create_multi_grid_chart_canvas_nodes", messages)
            self.assertIn("UniformGrid", messages)
            self.assertIn("ChartCanvas::new_grid_view", messages)
            self.assertIn("ChartCanvasPanelProps", messages)
            self.assertIn("Rc<RefCell<ChartEngine>>", messages)

    def test_echarts_multi_grid_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/echarts_multi_grid_demo.rs",
                """
                use fret_chart::{ChartCanvasMultiGridBinding, chart_canvas_panel};
                use fret_ui::UiTree;

                pub struct EchartsMultiGridDemoWindowState {
                    ui: UiTree<App>,
                    chart: ChartCanvasMultiGridBinding,
                }

                impl EchartsMultiGridDemoDriver {
                    fn build_ui(app: &mut App, window: AppWindowId) -> EchartsMultiGridDemoWindowState {
                        let (engine, spec, grids) = Self::build_chart();
                        EchartsMultiGridDemoWindowState {
                            ui: UiTree::new(),
                            chart: ChartCanvasMultiGridBinding::new(app, spec, engine, grids),
                        }
                    }
                }

                fn render(
                    app: &mut App,
                    services: &mut Services,
                    window: AppWindowId,
                    bounds: Rect,
                    state: &mut EchartsMultiGridDemoWindowState,
                ) {
                    let chart = state.chart.clone();
                    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                        .render_root("echarts-multi-grid-demo", move |cx| {
                            chart.observe_engine_paint(cx);
                            let mut overlay_props = chart.overlay_panel_props();
                            let grid_views = cx.container(container, move |cx| {
                                chart
                                    .grids()
                                    .iter()
                                    .copied()
                                    .map(|grid| {
                                        let props = chart.grid_panel_props(grid);
                                        chart_canvas_panel(cx, props)
                                    })
                                    .collect::<Vec<_>>()
                            });
                            vec![grid_views, chart_canvas_panel(cx, overlay_props)]
                        });
                    let _ = root;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/echarts_multi_grid_demo.rs",
                        "advanced_manual",
                        "fixture ECharts multi-grid surface",
                        owner="examples-echarts-multi-grid",
                        allowed_raw_seams=("fret_ui", "UiTree"),
                    )
                ],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "advanced-surface-echarts-multi-grid-binding-boundary"
                ]
            )

    def test_workspace_shell_driver_direct_model_writes_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/workspace_shell_demo/driver.rs",
                """
                use fret_runtime::{ModelStore, PlatformCapabilities};

                struct WorkspaceShellModelBundle {}
                impl WorkspaceShellModelBundle {
                    fn new(
                        models: &mut ModelStore,
                        window_layout: WorkspaceWindowLayout,
                        file_tree_items: Vec<TreeItem>,
                        file_tree_state: TreeState,
                    ) -> Self {
                        Self {}
                    }
                }

                struct WorkspaceShellModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> WorkspaceShellModelOwner<'a> {
                    fn new(models: &'a mut ModelStore) -> Self {
                        Self { models }
                    }

                    fn update<T: Any, R>(&mut self, model: &Model<T>, f: impl FnOnce(&mut T) -> R) -> Option<R> {
                        None
                    }

                    fn set<T: Any>(&mut self, model: &Model<T>, value: T) -> bool {
                        true
                    }

                    fn update_window_layout<R>(
                        &mut self,
                        state: &WorkspaceShellWindowState,
                        f: impl FnOnce(&mut WorkspaceWindowLayout) -> R,
                    ) -> Option<R> {
                        None
                    }

                    fn open_dirty_close_prompt(&mut self, state: &WorkspaceShellWindowState, prompt: WorkspaceShellDirtyClosePrompt) {}
                    fn clear_dirty_close_prompt(&mut self, state: &WorkspaceShellWindowState) {}
                    fn toggle_tabstrip_two_row_pinned(&mut self, model: &Model<bool>) -> bool { true }
                }

                fn workspace_shell_update_window_layout<R>(
                    app: &mut App,
                    state: &WorkspaceShellWindowState,
                    f: impl FnOnce(&mut WorkspaceWindowLayout) -> R,
                ) -> Option<R> {
                    WorkspaceShellModelOwner::new(app.models_mut()).update_window_layout(state, f)
                }

                fn workspace_shell_open_dirty_close_prompt(app: &mut App, state: &WorkspaceShellWindowState, prompt: WorkspaceShellDirtyClosePrompt) {
                    WorkspaceShellModelOwner::new(app.models_mut()).open_dirty_close_prompt(state, prompt);
                }

                fn workspace_shell_clear_dirty_close_prompt(app: &mut App, state: &WorkspaceShellWindowState) {
                    WorkspaceShellModelOwner::new(app.models_mut()).clear_dirty_close_prompt(state);
                }

                fn workspace_shell_host_clear_dirty_close_prompt(
                    host: &mut dyn Host,
                    prompt_model: &Model<Option<WorkspaceShellDirtyClosePrompt>>,
                    open_model: &Model<bool>,
                ) {
                    let mut owner = WorkspaceShellModelOwner::new(host.models_mut());
                    let _ = owner.set(prompt_model, None);
                    let _ = owner.set(open_model, false);
                }

                fn build_ui(app: &mut App, window_layout: WorkspaceWindowLayout, items_value: Vec<TreeItem>, state_value: TreeState) {
                    let models = WorkspaceShellModelBundle::new(app.models_mut(), window_layout, items_value, state_value);
                    let _ = models;
                }

                fn on_close(host: &mut dyn Host, prompt_model: Model<Option<WorkspaceShellDirtyClosePrompt>>, open_model: Model<bool>) {
                    workspace_shell_host_clear_dirty_close_prompt(host, &prompt_model, &open_model);
                }

                fn open_prompt(app: &mut App, state: &WorkspaceShellWindowState, req: Request) {
                    workspace_shell_open_dirty_close_prompt(app, state, WorkspaceShellDirtyClosePrompt::window_close(req),);
                    workspace_shell_clear_dirty_close_prompt(app, state);
                }

                fn toggle(app: &mut App, state: &WorkspaceShellWindowState) {
                    WorkspaceShellModelOwner::new(app.models_mut()).toggle_tabstrip_two_row_pinned(&state.tabstrip_two_row_pinned);
                }

                fn bad(app: &mut App, state: &WorkspaceShellWindowState) {
                    let _ = app.models_mut().update(&state.tabstrip_two_row_pinned, |_| true);
                    let _ = ModelStore::update(app.models_mut(), &state.tabstrip_two_row_pinned, |_| true);
                    let mut store = app.models_mut();
                    let _ = store.update(&state.tabstrip_two_row_pinned, |_| true);
                    let _ = app.models_mut().insert(false);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/workspace_shell_demo",
                        "advanced_manual",
                        "fixture workspace shell surface",
                        owner="examples-workspace-shell",
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
                if violation.rule == "advanced-surface-workspace-shell-driver-owner-boundary"
            ]
            self.assertEqual(4, len(owner_violations))
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("models_mut().update", messages)
            self.assertIn("ModelStore::update", messages)
            self.assertIn("ModelStore alias", messages)
            self.assertIn("models_mut().insert", messages)

    def test_workspace_shell_driver_owner_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/workspace_shell_demo/driver.rs",
                """
                use fret_runtime::{ModelStore, PlatformCapabilities};

                struct WorkspaceShellModelBundle {}
                impl WorkspaceShellModelBundle {
                    fn new(
                        models: &mut ModelStore,
                        window_layout: WorkspaceWindowLayout,
                        file_tree_items: Vec<TreeItem>,
                        file_tree_state: TreeState,
                    ) -> Self {
                        Self {}
                    }
                }

                struct WorkspaceShellModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> WorkspaceShellModelOwner<'a> {
                    fn new(models: &'a mut ModelStore) -> Self {
                        Self { models }
                    }

                    fn update<T: Any, R>(&mut self, model: &Model<T>, f: impl FnOnce(&mut T) -> R) -> Option<R> {
                        None
                    }

                    fn set<T: Any>(&mut self, model: &Model<T>, value: T) -> bool {
                        true
                    }

                    fn update_window_layout<R>(
                        &mut self,
                        state: &WorkspaceShellWindowState,
                        f: impl FnOnce(&mut WorkspaceWindowLayout) -> R,
                    ) -> Option<R> {
                        None
                    }

                    fn open_dirty_close_prompt(&mut self, state: &WorkspaceShellWindowState, prompt: WorkspaceShellDirtyClosePrompt) {}
                    fn clear_dirty_close_prompt(&mut self, state: &WorkspaceShellWindowState) {}
                    fn toggle_tabstrip_two_row_pinned(&mut self, model: &Model<bool>) -> bool { true }
                }

                fn workspace_shell_update_window_layout<R>(
                    app: &mut App,
                    state: &WorkspaceShellWindowState,
                    f: impl FnOnce(&mut WorkspaceWindowLayout) -> R,
                ) -> Option<R> {
                    WorkspaceShellModelOwner::new(app.models_mut()).update_window_layout(state, f)
                }

                fn workspace_shell_open_dirty_close_prompt(app: &mut App, state: &WorkspaceShellWindowState, prompt: WorkspaceShellDirtyClosePrompt) {
                    WorkspaceShellModelOwner::new(app.models_mut()).open_dirty_close_prompt(state, prompt);
                }

                fn workspace_shell_clear_dirty_close_prompt(app: &mut App, state: &WorkspaceShellWindowState) {
                    WorkspaceShellModelOwner::new(app.models_mut()).clear_dirty_close_prompt(state);
                }

                fn workspace_shell_host_clear_dirty_close_prompt(
                    host: &mut dyn Host,
                    prompt_model: &Model<Option<WorkspaceShellDirtyClosePrompt>>,
                    open_model: &Model<bool>,
                ) {
                    let mut owner = WorkspaceShellModelOwner::new(host.models_mut());
                    let _ = owner.set(prompt_model, None);
                    let _ = owner.set(open_model, false);
                }

                fn build_ui(app: &mut App, window_layout: WorkspaceWindowLayout, items_value: Vec<TreeItem>, state_value: TreeState) {
                    let models = WorkspaceShellModelBundle::new(app.models_mut(), window_layout, items_value, state_value);
                    let _ = models;
                }

                fn on_close(host: &mut dyn Host, prompt_model: Model<Option<WorkspaceShellDirtyClosePrompt>>, open_model: Model<bool>) {
                    workspace_shell_host_clear_dirty_close_prompt(host, &prompt_model, &open_model);
                }

                fn open_prompt(app: &mut App, state: &WorkspaceShellWindowState, req: Request) {
                    workspace_shell_open_dirty_close_prompt(app, state, WorkspaceShellDirtyClosePrompt::window_close(req),);
                    workspace_shell_clear_dirty_close_prompt(app, state);
                }

                fn toggle(app: &mut App, state: &WorkspaceShellWindowState) {
                    WorkspaceShellModelOwner::new(app.models_mut()).toggle_tabstrip_two_row_pinned(&state.tabstrip_two_row_pinned);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/workspace_shell_demo",
                        "advanced_manual",
                        "fixture workspace shell surface",
                        owner="examples-workspace-shell",
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
                    == "advanced-surface-workspace-shell-driver-owner-boundary"
                ]
            )

    def test_api_workbench_direct_model_access_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/api_workbench_lite_demo.rs",
                """
                use std::sync::Arc;

                use fret::app::prelude::*;
                use fret::app::{LocalState, LocalStateTxn};
                use fret_ui_shadcn::facade as shadcn;

                struct WorkbenchLocals {
                    method: LocalState<Option<Arc<str>>>,
                }

                type ApiWorkbenchModelStore = fret_runtime::ModelStore;

                struct ApiWorkbenchModelOwner<'a> {
                    models: &'a mut ApiWorkbenchModelStore,
                }

                impl<'a> ApiWorkbenchModelOwner<'a> {
                    fn new(models: &'a mut ApiWorkbenchModelStore) -> Self {
                        Self { models }
                    }

                    fn request_snapshot(&mut self, locals: &WorkbenchLocals) -> Option<RequestSnapshot> {
                        LocalStateTxn::with_model_store(self.models, |_tx| None)
                    }

                    fn prepare_request_submission_ui(&mut self, locals: &WorkbenchLocals) -> bool {
                        LocalStateTxn::with_model_store(self.models, |_tx| true)
                    }

                    fn apply_response_snapshot(
                        &mut self,
                        locals: &WorkbenchLocals,
                        state: MutationState<RequestSnapshot, ResponsePayload>,
                    ) -> bool {
                        LocalStateTxn::with_model_store(self.models, |_tx| true)
                    }

                    fn apply_collection(&mut self, locals: &WorkbenchLocals, preset_id: u8) -> bool {
                        LocalStateTxn::with_model_store(self.models, |_tx| true)
                    }

                    fn submit_request(
                        &mut self,
                        window: WindowId,
                        locals: &WorkbenchLocals,
                        response_mutation: &MutationHandle<RequestSnapshot, ResponsePayload>,
                        history_save_mutation: &MutationHandle<RequestSnapshot, ()>,
                    ) -> bool {
                        let snapshot = RequestSnapshot;
                        let mut handled = self.prepare_request_submission_ui(locals);
                        handled = history_save_mutation.submit(self.models, window, snapshot.clone()) || handled;
                        handled = response_mutation.submit(self.models, window, snapshot) || handled;
                        handled
                    }

                    fn retry_last_request(
                        &mut self,
                        window: WindowId,
                        locals: &WorkbenchLocals,
                        response_mutation: &MutationHandle<RequestSnapshot, ResponsePayload>,
                        history_save_mutation: &MutationHandle<RequestSnapshot, ()>,
                    ) -> bool {
                        if !self.can_retry_last_request(response_mutation) {
                            return false;
                        }
                        let mut handled = self.prepare_request_submission_ui(locals);
                        handled = history_save_mutation.retry_last(self.models, window) || handled;
                        handled = response_mutation.retry_last(self.models, window) || handled;
                        handled
                    }

                    fn can_retry_last_request(
                        &mut self,
                        response_mutation: &MutationHandle<RequestSnapshot, ResponsePayload>,
                    ) -> bool {
                        self.models.read(response_mutation.model(), |_st| true).ok().unwrap_or(false)
                    }

                    fn load_history(
                        &mut self,
                        locals: &WorkbenchLocals,
                        history_query: &QueryHandle<Vec<PersistedHistoryEntry>>,
                        history_id: u64,
                    ) -> bool {
                        let _history = self.models.read(history_query.model(), Clone::clone).ok();
                        LocalStateTxn::with_model_store(self.models, |_tx| true)
                    }
                }

                impl WorkbenchLocals {
                    fn new(cx: &mut AppUi<'_, '_>) -> Self {
                        todo!()
                    }
                }

                fn render(
                    cx: &mut AppUi<'_, '_>,
                    locals: WorkbenchLocals,
                    response_mutation: MutationHandle<RequestSnapshot, ResponsePayload>,
                    history_save_mutation: MutationHandle<RequestSnapshot, ()>,
                    history_query: QueryHandle<Vec<PersistedHistoryEntry>>,
                    window: WindowId,
                ) {
                    let _ = shadcn::Button::new("Send Request");
                    let _ = cx.data().update_after_mutation_completion(
                        1,
                        &response_mutation,
                        {
                            let locals = locals.clone();
                            move |models, state| {
                                ApiWorkbenchModelOwner::new(models).apply_response_snapshot(&locals, state)
                            }
                        },
                    );
                    bind_actions(cx, &locals, &response_mutation, &history_save_mutation, &history_query, window);
                }

                fn bind_actions(
                    cx: &mut AppUi<'_, '_>,
                    locals: &WorkbenchLocals,
                    response_mutation: &MutationHandle<RequestSnapshot, ResponsePayload>,
                    history_save_mutation: &MutationHandle<RequestSnapshot, ()>,
                    history_query: &QueryHandle<Vec<PersistedHistoryEntry>>,
                    window: WindowId,
                ) {
                    cx.actions().models::<act::SendRequest>({
                        let locals = locals.clone();
                        let response_mutation = response_mutation.clone();
                        let history_save_mutation = history_save_mutation.clone();
                        move |models| {
                            ApiWorkbenchModelOwner::new(models).submit_request(
                                window,
                                &locals,
                                &response_mutation,
                                &history_save_mutation,
                            )
                        }
                    });
                    cx.actions().models::<act::RetryLastRequest>({
                        let locals = locals.clone();
                        let response_mutation = response_mutation.clone();
                        let history_save_mutation = history_save_mutation.clone();
                        move |models| {
                            ApiWorkbenchModelOwner::new(models).retry_last_request(
                                window,
                                &locals,
                                &response_mutation,
                                &history_save_mutation,
                            )
                        }
                    });
                    cx.actions().availability::<act::RetryLastRequest>({
                        let response_mutation = response_mutation.clone();
                        move |host, _acx| {
                            ApiWorkbenchModelOwner::new(host.models_mut())
                                .can_retry_last_request(&response_mutation)
                        }
                    });
                    cx.actions().payload_models::<act::LoadCollection>({
                        let locals = locals.clone();
                        move |models, preset_id| {
                            ApiWorkbenchModelOwner::new(models).apply_collection(&locals, preset_id)
                        }
                    });
                    cx.actions().payload_models::<act::LoadHistory>({
                        let locals = locals.clone();
                        let history_query = history_query.clone();
                        move |models, history_id| {
                            ApiWorkbenchModelOwner::new(models).load_history(&locals, &history_query, history_id)
                        }
                    });
                }

                fn bad(app: &mut KernelApp, host: &mut Host, state: &State) {
                    let _ = app.models_mut().update(&state.model, |_| true);
                    let _ = app.models_mut().read(&state.model, |_| true);
                    let _ = ModelStore::update(app.models_mut(), &state.model, |_| true);
                    let _ = ModelStore::read(app.models_mut(), &state.model, |_| true);
                    let _ = LocalStateTxn::with_model_store(app.models_mut(), |_tx| true);
                    let mut store = host.models_mut();
                    let _ = store.update(&state.model, |_| true);
                    let _ = store.read(&state.model, |_| true);
                }

                fn main() {
                    let _ = FretApp::new("api-workbench-lite");
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                comparison_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/api_workbench_lite_demo.rs",
                        "comparison",
                        "fixture API Workbench surface",
                        owner="examples-api-workbench",
                        allowed_raw_seams=(
                            "fret_app",
                            "fret_runtime",
                            "fret_ui",
                            "AnyElement",
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
                if violation.rule == "comparison-surface-api-workbench-model-owner-boundary"
            ]
            self.assertGreaterEqual(len(owner_violations), 7)
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("models_mut().update", messages)
            self.assertIn("models_mut().read", messages)
            self.assertIn("ModelStore::update", messages)
            self.assertIn("ModelStore::read", messages)
            self.assertIn("LocalStateTxn::with_model_store", messages)
            self.assertIn("ModelStore alias", messages)
            self.assertIn("host.models_mut()", messages)

    def test_api_workbench_owner_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/api_workbench_lite_demo.rs",
                """
                use std::sync::Arc;

                use fret::app::prelude::*;
                use fret::app::{LocalState, LocalStateTxn};
                use fret_ui_shadcn::facade as shadcn;

                struct WorkbenchLocals {
                    method: LocalState<Option<Arc<str>>>,
                }

                type ApiWorkbenchModelStore = fret_runtime::ModelStore;

                struct ApiWorkbenchModelOwner<'a> {
                    models: &'a mut ApiWorkbenchModelStore,
                }

                impl<'a> ApiWorkbenchModelOwner<'a> {
                    fn new(models: &'a mut ApiWorkbenchModelStore) -> Self {
                        Self { models }
                    }

                    fn request_snapshot(&mut self, locals: &WorkbenchLocals) -> Option<RequestSnapshot> {
                        LocalStateTxn::with_model_store(self.models, |_tx| None)
                    }

                    fn prepare_request_submission_ui(&mut self, locals: &WorkbenchLocals) -> bool {
                        LocalStateTxn::with_model_store(self.models, |_tx| true)
                    }

                    fn apply_response_snapshot(
                        &mut self,
                        locals: &WorkbenchLocals,
                        state: MutationState<RequestSnapshot, ResponsePayload>,
                    ) -> bool {
                        LocalStateTxn::with_model_store(self.models, |_tx| true)
                    }

                    fn apply_collection(&mut self, locals: &WorkbenchLocals, preset_id: u8) -> bool {
                        LocalStateTxn::with_model_store(self.models, |_tx| true)
                    }

                    fn submit_request(
                        &mut self,
                        window: WindowId,
                        locals: &WorkbenchLocals,
                        response_mutation: &MutationHandle<RequestSnapshot, ResponsePayload>,
                        history_save_mutation: &MutationHandle<RequestSnapshot, ()>,
                    ) -> bool {
                        let snapshot = RequestSnapshot;
                        let mut handled = self.prepare_request_submission_ui(locals);
                        handled = history_save_mutation.submit(self.models, window, snapshot.clone()) || handled;
                        handled = response_mutation.submit(self.models, window, snapshot) || handled;
                        handled
                    }

                    fn retry_last_request(
                        &mut self,
                        window: WindowId,
                        locals: &WorkbenchLocals,
                        response_mutation: &MutationHandle<RequestSnapshot, ResponsePayload>,
                        history_save_mutation: &MutationHandle<RequestSnapshot, ()>,
                    ) -> bool {
                        if !self.can_retry_last_request(response_mutation) {
                            return false;
                        }
                        let mut handled = self.prepare_request_submission_ui(locals);
                        handled = history_save_mutation.retry_last(self.models, window) || handled;
                        handled = response_mutation.retry_last(self.models, window) || handled;
                        handled
                    }

                    fn can_retry_last_request(
                        &mut self,
                        response_mutation: &MutationHandle<RequestSnapshot, ResponsePayload>,
                    ) -> bool {
                        self.models.read(response_mutation.model(), |_st| true).ok().unwrap_or(false)
                    }

                    fn load_history(
                        &mut self,
                        locals: &WorkbenchLocals,
                        history_query: &QueryHandle<Vec<PersistedHistoryEntry>>,
                        history_id: u64,
                    ) -> bool {
                        let _history = self.models.read(history_query.model(), Clone::clone).ok();
                        LocalStateTxn::with_model_store(self.models, |_tx| true)
                    }
                }

                impl WorkbenchLocals {
                    fn new(cx: &mut AppUi<'_, '_>) -> Self {
                        todo!()
                    }
                }

                fn render(
                    cx: &mut AppUi<'_, '_>,
                    locals: WorkbenchLocals,
                    response_mutation: MutationHandle<RequestSnapshot, ResponsePayload>,
                    history_save_mutation: MutationHandle<RequestSnapshot, ()>,
                    history_query: QueryHandle<Vec<PersistedHistoryEntry>>,
                    window: WindowId,
                ) {
                    let _ = shadcn::Button::new("Send Request");
                    let _ = cx.data().update_after_mutation_completion(
                        1,
                        &response_mutation,
                        {
                            let locals = locals.clone();
                            move |models, state| {
                                ApiWorkbenchModelOwner::new(models).apply_response_snapshot(&locals, state)
                            }
                        },
                    );
                    bind_actions(cx, &locals, &response_mutation, &history_save_mutation, &history_query, window);
                }

                fn bind_actions(
                    cx: &mut AppUi<'_, '_>,
                    locals: &WorkbenchLocals,
                    response_mutation: &MutationHandle<RequestSnapshot, ResponsePayload>,
                    history_save_mutation: &MutationHandle<RequestSnapshot, ()>,
                    history_query: &QueryHandle<Vec<PersistedHistoryEntry>>,
                    window: WindowId,
                ) {
                    cx.actions().models::<act::SendRequest>({
                        let locals = locals.clone();
                        let response_mutation = response_mutation.clone();
                        let history_save_mutation = history_save_mutation.clone();
                        move |models| {
                            ApiWorkbenchModelOwner::new(models).submit_request(
                                window,
                                &locals,
                                &response_mutation,
                                &history_save_mutation,
                            )
                        }
                    });
                    cx.actions().models::<act::RetryLastRequest>({
                        let locals = locals.clone();
                        let response_mutation = response_mutation.clone();
                        let history_save_mutation = history_save_mutation.clone();
                        move |models| {
                            ApiWorkbenchModelOwner::new(models).retry_last_request(
                                window,
                                &locals,
                                &response_mutation,
                                &history_save_mutation,
                            )
                        }
                    });
                    cx.actions().availability::<act::RetryLastRequest>({
                        let response_mutation = response_mutation.clone();
                        move |host, _acx| {
                            ApiWorkbenchModelOwner::new(host.models_mut())
                                .can_retry_last_request(&response_mutation)
                        }
                    });
                    cx.actions().payload_models::<act::LoadCollection>({
                        let locals = locals.clone();
                        move |models, preset_id| {
                            ApiWorkbenchModelOwner::new(models).apply_collection(&locals, preset_id)
                        }
                    });
                    cx.actions().payload_models::<act::LoadHistory>({
                        let locals = locals.clone();
                        let history_query = history_query.clone();
                        move |models, history_id| {
                            ApiWorkbenchModelOwner::new(models).load_history(&locals, &history_query, history_id)
                        }
                    });
                }

                fn main() {
                    let _ = FretApp::new("api-workbench-lite");
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[],
                comparison_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/api_workbench_lite_demo.rs",
                        "comparison",
                        "fixture API Workbench surface",
                        owner="examples-api-workbench",
                        allowed_raw_seams=(
                            "fret_app",
                            "fret_runtime",
                            "fret_ui",
                            "AnyElement",
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
                    if violation.rule == "comparison-surface-api-workbench-model-owner-boundary"
                ]
            )

    def test_genui_direct_model_access_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/genui_demo.rs",
                """
                use fret_runtime::{Model, ModelStore};

                struct GenUiModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> GenUiModelOwner<'a> {
                    fn new(models: &'a mut ModelStore) -> Self {
                        Self { models }
                    }

                    fn update<T: Any, R>(&mut self, model: &Model<T>, f: impl FnOnce(&mut T) -> R) -> Option<R> {
                        None
                    }

                    fn read<T: Any, R>(&mut self, model: &Model<T>, f: impl FnOnce(&T) -> R) -> Option<R> {
                        None
                    }
                }

                struct GenUiState;
                impl GenUiState {
                    fn reset_runtime_models(&self, app: &mut KernelApp, seed: Value) {
                        let mut owner = GenUiModelOwner::new(app.models_mut());
                        let _ = owner;
                    }
                }

                fn reset(state: &GenUiState, app: &mut KernelApp, seed: Value) {
                    state.reset_runtime_models(app, seed);
                }

                fn handler(
                    host: &mut Host,
                    state_model_for_confirm: Model<Value>,
                    validation_model: Model<Validation>,
                    state_model_for_submit: Model<Value>,
                    out: Validation,
                ) {
                    let mut owner = GenUiModelOwner::new(host.models_mut());
                    owner.read(&state_model_for_confirm, |v| v);
                    owner.update(&state_model_for_confirm, |v| {});
                    owner.update(&validation_model, |v| *v = out);
                    owner.update(&state_model_for_submit, |v| {});
                }

                fn bad(app: &mut KernelApp, host: &mut Host, state: &State) {
                    let _ = app.models_mut().update(&state.model, |_| true);
                    let _ = app.models_mut().read(&state.model, |_| true);
                    let _ = ModelStore::update(app.models_mut(), &state.model, |_| true);
                    let _ = ModelStore::read(app.models_mut(), &state.model, |_| true);
                    let mut store = host.models_mut();
                    let _ = store.update(&state.model, |_| true);
                    let _ = store.read(&state.model, |_| true);
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/genui_demo.rs",
                        "advanced_manual",
                        "fixture GenUI surface",
                        owner="examples-genui-demo",
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
                if violation.rule == "advanced-surface-genui-model-owner-boundary"
            ]
            self.assertEqual(6, len(owner_violations))
            messages = "\n".join(violation.message for violation in owner_violations)
            self.assertIn("models_mut().update", messages)
            self.assertIn("models_mut().read", messages)
            self.assertIn("ModelStore::update", messages)
            self.assertIn("ModelStore::read", messages)
            self.assertIn("ModelStore alias", messages)

    def test_genui_owner_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/genui_demo.rs",
                """
                use fret_runtime::{Model, ModelStore};

                struct GenUiModelOwner<'a> {
                    models: &'a mut ModelStore,
                }

                impl<'a> GenUiModelOwner<'a> {
                    fn new(models: &'a mut ModelStore) -> Self {
                        Self { models }
                    }

                    fn update<T: Any, R>(&mut self, model: &Model<T>, f: impl FnOnce(&mut T) -> R) -> Option<R> {
                        None
                    }

                    fn read<T: Any, R>(&mut self, model: &Model<T>, f: impl FnOnce(&T) -> R) -> Option<R> {
                        None
                    }
                }

                struct GenUiState;
                impl GenUiState {
                    fn reset_runtime_models(&self, app: &mut KernelApp, seed: Value) {
                        let mut owner = GenUiModelOwner::new(app.models_mut());
                        let _ = owner;
                    }
                }

                fn reset(state: &GenUiState, app: &mut KernelApp, seed: Value) {
                    state.reset_runtime_models(app, seed);
                }

                fn handler(
                    host: &mut Host,
                    state_model_for_confirm: Model<Value>,
                    validation_model: Model<Validation>,
                    state_model_for_submit: Model<Value>,
                    out: Validation,
                ) {
                    let mut owner = GenUiModelOwner::new(host.models_mut());
                    owner.read(&state_model_for_confirm, |v| v);
                    owner.update(&state_model_for_confirm, |v| {});
                    owner.update(&validation_model, |v| *v = out);
                    owner.update(&state_model_for_submit, |v| {});
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/genui_demo.rs",
                        "advanced_manual",
                        "fixture GenUI surface",
                        owner="examples-genui-demo",
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
                    if violation.rule == "advanced-surface-genui-model-owner-boundary"
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
                use fret::app::{self, App, AppLocalStateExt as _, LocalState, RenderContextAccess as _, text};
                use fret_app::{App, CommandId, Effect, Model, WindowRequest};

                struct DemoWindowState {
                    table_output: LocalState<shadcn::DataTableViewOutput>,
                    table_recipe: shadcn::DataTableRecipe<DemoRow>,
                    legacy_output: Model<shadcn::DataTableViewOutput>,
                }

                fn build_ui(app: &mut App) {
                    let table_output = app.models_mut().insert(shadcn::DataTableViewOutput::default());
                    let _ = table_output;
                }

                fn render(cx: &mut ElementContext<'_, App>, state: &DemoWindowState) {
                    let table_output = state.table_output.clone();
                    let table_recipe = state.table_recipe.clone();
                    cx.observe_model(&table_output,Invalidation::Layout);
                    let _ = table_output.layout_value(cx);
                    let table_output = app.local_state(shadcn::DataTableViewOutput::default());
                    let table_state = app.local_state(shadcn::TableState::default());
                    let table_recipe = shadcn::DataTableRecipe::new(&table_state, &table_output, columns, |row, _i, _parent| shadcn::RowKey(row.id));
                    let table_output = state.table_output.clone();
                    let table_recipe = state.table_recipe.clone();
                    let _ = table_output.layout_value(cx);
                    let table_parts = table_recipe.into_elements(cx, rows, 1, |cx, col, row| text::table_cell(cx, Arc::from("")));
                    shadcn::DataTablePagination::new(&table_state, table_output.clone());
                    shadcn::DataTable::new(rows, columns).output_model(table_output.clone());
                }

                impl fret::app::View for DemoWindowState {
                    fn init(app: &mut App, window: fret::WindowId) -> Self {
                        create_window_state(app, window)
                    }

                    fn render(&mut self, cx: &mut fret::AppUi<'_, '_>) -> fret::Ui {
                        render_datatable_demo(cx, self)
                    }
                }

                fn run() {
                    fret::FretApp::new("datatable-demo").view::<DemoWindowState>()?;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/datatable_demo.rs",
                        "default_app_clean",
                        "fixture datatable demo surface",
                        owner="examples-datatable",
                    )
                ],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "surface-datatable-output-boundary"
            ]
            self.assertEqual(4, len(owner_violations))
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
                use fret::app::{self, App, AppLocalStateExt as _, LocalState, RenderContextAccess as _, text};

                struct DemoWindowState {
                    table_output: LocalState<shadcn::DataTableViewOutput>,
                    table_recipe: shadcn::DataTableRecipe<DemoRow>,
                }

                fn build_ui(app: &mut App) {
                    let table_output = app.local_state(shadcn::DataTableViewOutput::default());
                    let table_state = app.local_state(shadcn::TableState::default());
                    let columns = datatable_columns();
                    let table_recipe = shadcn::DataTableRecipe::new(&table_state, &table_output, columns, |row, _i, _parent| shadcn::RowKey(row.id));
                    let _ = table_output;
                    let _ = table_recipe;
                }

                fn render(cx: &mut ElementContext<'_, App>, state: &DemoWindowState) {
                    let table_output = state.table_output.clone();
                    let table_recipe = state.table_recipe.clone();
                    let _ = table_output.layout_value(cx);
                    let table_parts = table_recipe.into_elements(cx, rows, 1, |cx, col, row| text::table_cell(cx, Arc::from("")));
                }

                impl fret::app::View for DemoWindowState {
                    fn init(app: &mut App, window: fret::WindowId) -> Self {
                        create_window_state(app, window)
                    }

                    fn render(&mut self, cx: &mut fret::AppUi<'_, '_>) -> fret::Ui {
                        render_datatable_demo(cx, self)
                    }
                }

                fn run() {
                    fret::FretApp::new("datatable-demo").view::<DemoWindowState>()?;
                }
                """,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/datatable_demo.rs",
                        "default_app_clean",
                        "fixture datatable demo surface",
                        owner="examples-datatable",
                    )
                ],
                advanced_manual_surfaces=[],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )

            self.assertFalse(
                [
                    violation
                    for violation in violations
                    if violation.rule == "surface-datatable-output-boundary"
                ]
            )

    def test_editor_notes_direct_model_writes_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            surface = POLICY.SurfacePath(
                "apps/fret-examples/src/editor_notes_demo.rs",
                "advanced_manual",
                "fixture editor notes surface",
                owner=POLICY.EDITOR_NOTES_OWNER,
                allowed_raw_seams=("fret_core", "fret_ui", "AnyElement"),
                retirement=POLICY.FRET_EXAMPLES_ADVANCED_RETIREMENT,
            )
            injections = (
                ("raw-runtime-import", "use fret_runtime::RuntimeProbe;"),
                ("raw-model-store", "struct RawStore(ModelStore);"),
                ("raw-model-handle", "struct RawModel(Model<String>);"),
                ("legacy-asset-model-owner", "struct EditorAssetModels;"),
                ("legacy-text-model-owner", "struct EditorNotesModelOwner;"),
                ("raw-model-store-access", "fn raw() { host.models_mut(); }"),
                ("manual-draft-controller", "fn raw() { TextFieldDraftController::new(); }"),
                ("direct-text-field-model-construction", "fn raw() { TextField::new(model); }"),
                (
                    "direct-theme-model-construction",
                    "fn raw() { EditorThemePresetPicker::new(model); }",
                ),
                ("raw-selector-model-read", "fn raw() { selector_model_paint(); }"),
                ("raw-local-state-construction", "fn raw() { LocalState::new_in(); }"),
                ("raw-local-state-bridge", "fn raw() { LocalState::from_model(); }"),
            )

            write(
                root / "apps/fret-examples/src/editor_notes_demo.rs",
                "\n".join(
                    [EDITOR_NOTES_APP_FACING_FIXTURE]
                    + [injection for _seam, injection in injections]
                ),
            )
            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[surface],
                policy_recipe_surfaces=[],
                mechanism_root_surfaces=[],
            )
            owner_violations = [
                violation
                for violation in violations
                if violation.rule == "app-facing-editor-notes-binding-boundary"
            ]
            self.assertEqual(len(injections), len(owner_violations))
            for seam, injection in injections:
                with self.subTest(seam=seam):
                    seam_violations = [
                        violation
                        for violation in owner_violations
                        if f"`{seam}`" in violation.message
                    ]
                    self.assertEqual(1, len(seam_violations), injection)

    def test_editor_notes_binding_surface_is_allowed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(
                root / "apps/fret-examples/src/editor_notes_demo.rs",
                EDITOR_NOTES_APP_FACING_FIXTURE,
            )

            violations = check_fixture_policy(
                root,
                default_surfaces=[],
                advanced_manual_surfaces=[
                    POLICY.SurfacePath(
                        "apps/fret-examples/src/editor_notes_demo.rs",
                        "advanced_manual",
                        "fixture editor notes surface",
                        owner=POLICY.EDITOR_NOTES_OWNER,
                        allowed_raw_seams=(
                            "fret_core",
                            "fret_ui",
                            "AnyElement",
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
                    if violation.rule == "app-facing-editor-notes-binding-boundary"
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

    def test_cookbook_classified_surfaces_require_explicit_owners(self) -> None:
        for helper in (
            POLICY._cookbook_advanced_surface,
            POLICY._cookbook_renderer_lab_surface,
        ):
            owner_parameter = inspect.signature(helper).parameters["owner"]
            self.assertEqual(inspect.Parameter.KEYWORD_ONLY, owner_parameter.kind)
            self.assertIs(inspect.Parameter.empty, owner_parameter.default)

        specs_by_path = {
            spec.path: spec
            for spec in (
                *POLICY.ADVANCED_MANUAL_SURFACES,
                *POLICY.RENDERER_LAB_SURFACES,
            )
        }
        expected_owners = {
            "apps/fret-cookbook/examples/compositing_alpha_basics.rs": "cookbook-compositing-alpha",
            "apps/fret-cookbook/examples/customv1_basics.rs": "cookbook-customv1",
            "apps/fret-cookbook/examples/docking_basics.rs": "cookbook-docking",
            "apps/fret-cookbook/examples/embedded_viewport_basics.rs": "cookbook-embedded-viewport",
            "apps/fret-cookbook/examples/external_texture_import_basics.rs": "cookbook-external-texture-import",
            "apps/fret-cookbook/examples/image_asset_cache_basics.rs": "cookbook-image-asset-cache",
            "apps/fret-cookbook/examples/utility_window_materials_windows.rs": "cookbook-utility-window-materials-windows",
        }
        for path, owner in expected_owners.items():
            self.assertEqual(owner, specs_by_path[path].owner)

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
