#!/usr/bin/env python3
"""Responsibility source-policy checks for Fret UI architecture surfaces.

This gate complements dependency layering. It catches source-level drift that cargo metadata cannot
see: default authoring paths importing raw runtime seams, advanced/manual seams being used without
classification, `fret-ui` root exports growing policy-coded names, and policy-coded vocabulary
returning to selected public mechanism APIs.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence


WORKSPACE_ROOT = Path(__file__).resolve().parent.parent
GATE_NAME = "surface responsibility policy"


@dataclass(frozen=True)
class SurfacePath:
    path: str
    category: str
    reason: str
    owner: str = ""
    allowed_raw_seams: tuple[str, ...] = ()
    retirement: str = ""


@dataclass(frozen=True)
class SurfaceViolation:
    rule: str
    path: Path
    line_no: int
    message: str
    source: str = ""


DEFAULT_AUTHORING_SURFACES: tuple[SurfacePath, ...] = (
    SurfacePath(
        "README.md",
        "default_app_clean",
        "repository first-contact snippets should teach the app facade",
    ),
    SurfacePath(
        "docs/first-hour.md",
        "default_app_clean",
        "first-hour docs are copied by app authors",
    ),
    SurfacePath(
        "docs/examples/README.md",
        "default_app_clean",
        "example taxonomy should keep default snippets on the app facade",
    ),
    SurfacePath(
        "docs/examples/todo-app-golden-path.md",
        "default_app_clean",
        "todo golden path is a default app tutorial",
    ),
    SurfacePath(
        "crates/fretboard/src/scaffold/contracts.rs",
        "default_app_clean",
        "public scaffold contracts should not expose runtime seams",
    ),
    SurfacePath(
        "crates/fretboard/src/scaffold/templates.rs",
        "default_app_clean",
        "generated app templates are copied verbatim",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/hello.rs",
        "default_app_clean",
        "default cookbook basics should teach the app facade",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/hello_counter.rs",
        "default_app_clean",
        "default cookbook basics should teach the app facade",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/toggle_basics.rs",
        "default_app_clean",
        "default cookbook basics should teach the app facade",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/date_picker_basics.rs",
        "default_app_clean",
        "default cookbook controls should stay on app-facing state helpers",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/drag_basics.rs",
        "default_app_clean",
        "default drag cookbook should stay on app-facing pointer and local-state helpers",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/async_inbox_basics.rs",
        "default_app_clean",
        "default async inbox cookbook should stay on app-facing async-work and local-state helpers",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs",
        "default_app_clean",
        "default canvas cookbook should stay on app-facing canvas, pointer, and local-state helpers",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/gizmo_basics.rs",
        "default_app_clean",
        "default gizmo cookbook should stay on app-facing canvas, pointer, and local-state helpers",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/chart_interactions_basics.rs",
        "default_app_clean",
        "default chart cookbook should stay on app-facing chart, command, and local-state helpers",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/markdown_and_code_basics.rs",
        "default_app_clean",
        "default cookbook controls should stay on app-facing state helpers",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/mutation_toast_feedback_basics.rs",
        "default_app_clean",
        "default mutation feedback cookbook should stay on app-facing data/effect helpers",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/payload_actions_basics.rs",
        "default_app_clean",
        "default action cookbook should stay on app-facing typed action helpers",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/imui_action_basics.rs",
        "default_app_clean",
        "default IMUI action cookbook should stay on app-facing local-state and imui facades",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/imui_editor_controls_basics.rs",
        "default_app_clean",
        "default IMUI editor controls cookbook should stay on app-facing local-state and editor facades",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/imui_plot_basics.rs",
        "default_app_clean",
        "default IMUI plot cookbook should stay on the plot binding facade instead of raw models",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/theme_switching_basics.rs",
        "default_app_clean",
        "default cookbook theme controls should stay on app-facing state helpers",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/data_table_basics.rs",
        "default_app_clean",
        "default data-table cookbook should not teach raw local-state construction",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/commands_keymap_basics.rs",
        "default_app_clean",
        "default command/keymap cookbook should stay on the explicit fret command facade",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/form_basics.rs",
        "default_app_clean",
        "default form cookbook should stay on app-facing action and semantics helpers",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/text_input_basics.rs",
        "default_app_clean",
        "default text-input cookbook should stay on app-facing command and semantics helpers",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/router_basics.rs",
        "default_app_clean",
        "default router cookbook should stay on the explicit fret router facade",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/virtual_list_basics.rs",
        "default_app_clean",
        "default virtual-list cookbook should stay on explicit fret style and virtual-list facades",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/undo_basics.rs",
        "default_app_clean",
        "default undo cookbook should stay on app-facing local-state and command facades",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/toast_basics.rs",
        "default_app_clean",
        "default toast cookbook should stay on app-facing effect helpers",
    ),
    SurfacePath(
        "apps/fret-examples/src/simple_todo_demo.rs",
        "default_app_clean",
        "copyable simple-todo example view should stay on the app facade",
    ),
)

POLICY_RECIPE_SURFACES: tuple[SurfacePath, ...] = (
    SurfacePath(
        "ecosystem/fret-ui-kit/src",
        "policy_recipe",
        "policy and headless infrastructure may consume generic fret-ui mechanisms",
    ),
    SurfacePath(
        "ecosystem/fret-ui-shadcn/src",
        "policy_recipe",
        "recipe crates may compose mechanism types into shadcn policy surfaces",
    ),
)


COOKBOOK_ADVANCED_RETIREMENT = (
    "Remove this quarantine record after the example either moves to default app facade wrappers "
    "or is reclassified under explicit advanced driver/view/interop/raw docs."
)

FRET_EXAMPLES_ADVANCED_RETIREMENT = (
    "Remove this quarantine record after the demo splits copyable app-view code from manual "
    "runner/test harness glue or moves the remaining raw seams behind explicit public wrappers."
)

CLASSIFIED_RAW_SURFACE_CATEGORIES = frozenset(
    {"advanced_manual", "comparison_surface", "internal_harness", "renderer_lab"}
)


def _cookbook_advanced_surface(
    filename: str,
    reason: str,
    allowed_raw_seams: tuple[str, ...],
) -> SurfacePath:
    owner = f"cookbook-{filename.removesuffix('.rs').removesuffix('_basics').replace('_', '-')}"
    return SurfacePath(
        f"apps/fret-cookbook/examples/{filename}",
        "advanced_manual",
        f"{filename} remains classified as an advanced cookbook surface because {reason}",
        owner=owner,
        allowed_raw_seams=allowed_raw_seams,
        retirement=COOKBOOK_ADVANCED_RETIREMENT,
    )


def _cookbook_renderer_lab_surface(
    filename: str,
    reason: str,
    allowed_raw_seams: tuple[str, ...],
) -> SurfacePath:
    owner = f"cookbook-{filename.removesuffix('.rs').removesuffix('_basics').replace('_', '-')}"
    return SurfacePath(
        f"apps/fret-cookbook/examples/{filename}",
        "renderer_lab",
        f"{filename} remains classified as a renderer lab because {reason}",
        owner=owner,
        allowed_raw_seams=allowed_raw_seams,
    )


def _fret_examples_advanced_surface(
    filename: str,
    reason: str,
    allowed_raw_seams: tuple[str, ...],
    owner: str | None = None,
) -> SurfacePath:
    stem = filename.removesuffix(".rs").replace("_", "-")
    return SurfacePath(
        f"apps/fret-examples/src/{filename}",
        "advanced_manual",
        f"{filename} remains classified as an advanced examples surface because {reason}",
        owner=owner or f"examples-{stem}",
        allowed_raw_seams=allowed_raw_seams,
        retirement=FRET_EXAMPLES_ADVANCED_RETIREMENT,
    )


CUSTOM_EFFECT_V2_WEB_ALLOWED_RAW_SEAMS = (
    "fret::advanced",
    "fret_app",
    "fret_core",
    "fret_launch",
    "fret_runtime",
    "fret_ui",
    "AnyElement",
    "ElementContext",
    "FnDriver",
    "ModelStore",
    "UiTree",
)

CUSTOM_EFFECT_V2_WEB_RETIREMENT = (
    "Remove this quarantine record after a public custom-effect parameter/control binding owns "
    "parameter models, reset/toggle actions, and effect-layer composition without exposing the "
    "raw ModelStore owner seam."
)


def _fret_examples_custom_effect_v2_web_surface(filename: str, variant: str) -> SurfacePath:
    return SurfacePath(
        f"apps/fret-examples/src/{filename}",
        "advanced_manual",
        (
            f"{filename} remains classified as an advanced examples surface because the {variant} "
            "custom-effect v2 web proof owns manual runner/bootstrap, raw parameter models, a "
            "local ModelStore owner, and low-level effect-layer composition"
        ),
        owner="examples-custom-effect-v2-web",
        allowed_raw_seams=CUSTOM_EFFECT_V2_WEB_ALLOWED_RAW_SEAMS,
        retirement=CUSTOM_EFFECT_V2_WEB_RETIREMENT,
    )


def _fret_examples_comparison_surface(
    path: str,
    reason: str,
    allowed_raw_seams: tuple[str, ...],
    owner: str | None = None,
) -> SurfacePath:
    stem = path.removesuffix(".rs").replace("/", "-").replace("_", "-")
    return SurfacePath(
        f"apps/fret-examples/src/{path}",
        "comparison_surface",
        f"{path} remains classified as a comparison surface because {reason}",
        owner=owner or f"examples-{stem}",
        allowed_raw_seams=allowed_raw_seams,
    )


def _fret_examples_internal_harness(
    path: str,
    reason: str,
    allowed_raw_seams: tuple[str, ...],
    owner: str | None = None,
) -> SurfacePath:
    stem = path.removesuffix(".rs").replace("/", "-").replace("_", "-")
    return SurfacePath(
        f"apps/fret-examples/src/{path}",
        "internal_harness",
        f"{path} remains classified as an internal harness because {reason}",
        owner=owner or f"examples-{stem}",
        allowed_raw_seams=allowed_raw_seams,
    )


COMPARISON_SURFACES: tuple[SurfacePath, ...] = (
    _fret_examples_comparison_surface(
        "api_workbench_lite_demo.rs",
        "it is an API ergonomics and migration reference while public workbench-lite and "
        "mutation-workbench starters cover default onboarding flows",
        (
            "fret_app",
            "fret_core",
            "fret_runtime",
            "fret_ui",
            "AnyElement",
            "ModelStore",
        ),
        owner="examples-api-workbench",
    ),
    _fret_examples_comparison_surface(
        "hello_world_compare_demo.rs",
        "it is a GPUI/Fret comparison and runtime diagnostics proof rather than an app-authoring "
        "tutorial",
        (
            "fret::advanced",
            "fret_core",
            "fret_runtime",
            "fret_ui",
            "AnyElement",
        ),
        owner="examples-hello-world-compare",
    ),
    _fret_examples_comparison_surface(
        "echarts_demo.rs",
        "it is an ECharts adapter smoke/comparison surface, while first-contact chart examples use "
        "ChartCanvasPanelBinding",
        (
            "fret::advanced",
            "fret_core",
            "fret_ui",
            "AnyElement",
            "ElementContext",
        ),
        owner="examples-echarts-adapter",
    ),
    _fret_examples_comparison_surface(
        "imui_editor_proof_demo/authoring_parity",
        "it compares declarative and immediate-mode authoring surfaces inside the editor proof",
        (
            "fret::advanced",
            "fret_core",
            "fret_runtime",
            "fret_ui",
            "ElementContext",
        ),
        owner="examples-imui-authoring-parity",
    ),
)


INTERNAL_HARNESS_SURFACES: tuple[SurfacePath, ...] = (
    _fret_examples_internal_harness(
        "lib.rs",
        "the examples crate root owns shared native/web harness helpers, launch glue, and theme "
        "interop helpers for demo shells",
        ("fret::advanced", "fret_app", "fret_core", "fret_launch"),
        owner="examples-harness-root",
    ),
    _fret_examples_internal_harness(
        "docking_arbitration_demo.rs",
        "the arbitration harness is ADR/conformance infrastructure for docking, viewports, "
        "overlays, and launch hooks",
        (
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
        owner="examples-docking-arbitration",
    ),
    _fret_examples_internal_harness(
        "plot_stress_demo.rs",
        "the plot stress harness owns manual driver state and retained stress-model plumbing",
        (
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "FnDriver",
            "UiTree",
        ),
        owner="examples-plot-stress",
    ),
    _fret_examples_internal_harness(
        "chart_stress_demo.rs",
        "the chart stress harness owns manual driver state, env-driven perf controls, and chart "
        "engine statistics directly",
        (
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "FnDriver",
            "UiTree",
        ),
        owner="examples-chart-stress",
    ),
    _fret_examples_internal_harness(
        "simple_todo_demo/driver.rs",
        "the simple-todo driver module owns native/web compatibility launch glue for demo shells",
        (
            "fret::advanced",
            "fret_launch",
            "fret_runtime",
        ),
        owner="examples-simple-todo-driver",
    ),
)


RENDERER_LAB_SURFACES: tuple[SurfacePath, ...] = (
    _cookbook_renderer_lab_surface(
        "compositing_alpha_basics.rs",
        "it is a deterministic screenshot baseline for straight-vs-premultiplied alpha renderer "
        "semantics rather than an app-authoring lesson",
        ("fret::advanced", "fret_app", "fret_core", "fret_launch", "FnDriver"),
    ),
    _cookbook_renderer_lab_surface(
        "image_asset_cache_basics.rs",
        "it is a deterministic screenshot baseline for keyed ImageAssetCache upload, eviction, "
        "and reload behavior rather than an app-authoring lesson",
        ("fret::advanced", "fret_app", "fret_core", "fret_launch", "FnDriver"),
    ),
    _cookbook_renderer_lab_surface(
        "customv1_basics.rs",
        "it is a capability-gated CustomV1 renderer/effect smoke surface rather than an "
        "app-authoring lesson",
        ("fret::advanced", "fret_core", "fret_ui", "AnyElement"),
    ),
)


ADVANCED_MANUAL_SURFACES: tuple[SurfacePath, ...] = (
    SurfacePath(
        "apps/fret-examples/src/workspace_shell_demo",
        "advanced_manual",
        (
            "workspace shell proof owns manual launch, UiTree, frame lifecycle, command "
            "dispatch, overlay, virtual-list, and diagnostics seams directly"
        ),
        owner="examples-workspace-shell",
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
        retirement=(
            "Replace with a public workspace-shell starter once AppUi wrappers own command, "
            "overlay, virtual-list, diagnostics, and window lifecycle flows"
        ),
    ),
    _fret_examples_advanced_surface(
        "todo_demo.rs",
        "the app-facing view is also the semantics/runtime test harness for the golden-path demo",
        (
            "fret::advanced",
            "fret_core",
            "fret_runtime",
            "fret_ui",
            "AnyElement",
            "UiTree",
        ),
        owner="examples-todo",
    ),
    _fret_examples_advanced_surface(
        "components_gallery.rs",
        "the gallery owns manual window lifecycle, component state matrices, file-dialog hooks, "
        "and diagnostics integration",
        (
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
        owner="examples-components-gallery",
    ),
    _fret_examples_advanced_surface(
        "docking_demo.rs",
        "the docking proof owns manual driver state, retained tree integration, and docking model "
        "wiring directly",
        (
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
        owner="examples-docking",
    ),
    _fret_examples_advanced_surface(
        "plot_demo.rs",
        "the plot proof owns manual driver state and retained plot model integration",
        (
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "FnDriver",
            "UiTree",
        ),
        owner="examples-plot",
    ),
    _fret_examples_advanced_surface(
        "gizmo3d_demo.rs",
        "the 3D gizmo proof owns manual runner, retained viewport state, and low-level rendering "
        "integration",
        (
            "fret::advanced",
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "FnDriver",
            "UiTree",
        ),
        owner="examples-gizmo3d",
    ),
    _fret_examples_custom_effect_v2_web_surface(
        "custom_effect_v2_web_demo.rs",
        "glass/distortion",
    ),
    _fret_examples_custom_effect_v2_web_surface(
        "custom_effect_v2_identity_web_demo.rs",
        "identity",
    ),
    _fret_examples_custom_effect_v2_web_surface(
        "custom_effect_v2_lut_web_demo.rs",
        "LUT",
    ),
    _fret_examples_custom_effect_v2_web_surface(
        "custom_effect_v2_glass_chrome_web_demo.rs",
        "glass chrome",
    ),
    _fret_examples_advanced_surface(
        "echarts_multi_grid_demo.rs",
        "the multi-grid ECharts proof still owns manual runner/bootstrap seams, while its shared "
        "chart engine, per-grid panels, and overlay-only panel are routed through "
        "ChartCanvasMultiGridBinding",
        (
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "FnDriver",
            "UiTree",
        ),
        owner="examples-echarts-multi-grid",
    ),
    _fret_examples_advanced_surface(
        "chart_multi_axis_demo.rs",
        "the linked multi-axis proof owns shared output, brush, axis-pointer, and domain-window "
        "models for explicit chart coordination",
        (
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
        owner="examples-chart-multi-axis",
    ),
    _cookbook_advanced_surface(
        "docking_basics.rs",
        "docking still demonstrates retained tree and low-level element interop",
        (
            "fret::advanced",
            "fret_app",
            "fret_core",
            "fret_runtime",
            "fret_ui",
            "AnyElement",
            "ElementContext",
            "UiTree",
        ),
    ),
    _cookbook_advanced_surface(
        "embedded_viewport_basics.rs",
        "embedded viewport interop still needs manual kernel, model, and element context seams",
        ("fret::advanced", "fret_core", "fret_runtime", "fret_ui", "ElementContext"),
    ),
    _cookbook_advanced_surface(
        "external_texture_import_basics.rs",
        "external texture import still owns manual launch, tree, and interop seams",
        (
            "fret::advanced",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "ElementContext",
            "UiTree",
        ),
    ),
    _cookbook_advanced_surface(
        "utility_window_materials_windows.rs",
        "utility window materials still owns retained tree and manual window/material seams",
        (
            "fret::advanced",
            "fret_app",
            "fret_core",
            "fret_runtime",
            "ElementContext",
            "UiTree",
        ),
    ),
    SurfacePath(
        "apps/fret-examples/src/node_graph_demo.rs",
        "advanced_manual",
        "node graph demo is an advanced proof only for app-view prelude plus low-level paint override types",
        owner="examples-node-graph",
        allowed_raw_seams=("fret_core",),
        retirement=(
            "Reclassify after node/canvas public starter covers graph creation, selection, "
            "diagnostics, and edge paint overrides without direct fret_core paint types"
        ),
    ),
    SurfacePath(
        "apps/fret-examples-imui/src",
        "advanced_manual",
        (
            "IMUI example app is an explicit immediate-mode interop lane that still demonstrates "
            "raw local-state model binding and low-level element/context hooks"
        ),
        owner="examples-imui",
        allowed_raw_seams=(
            "fret::advanced",
            "fret_core",
            "fret_ui",
            "AnyElement",
            "ElementContext",
        ),
        retirement=(
            "Remove raw local-state model hooks and low-level element/context imports after IMUI "
            "controls expose app-facing bindings for model, update, and context-owned reads"
        ),
    ),
    SurfacePath(
        "crates/fret-framework/src/lib.rs",
        "advanced_manual",
        "fret-framework is the manual assembly facade, not the default app crate",
        owner="kernel-facade",
        allowed_raw_seams=(
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "ElementContext",
            "FnDriver",
            "UiTree",
        ),
        retirement=(
            "Keep only while ADR 0109 preserves an opt-in manual assembly facade; shrink "
            "exports when default app and launch facades cover the same entry points"
        ),
    ),
)

MECHANISM_ROOT_SURFACES: tuple[SurfacePath, ...] = (
    SurfacePath(
        "crates/fret-ui/src/lib.rs",
        "mechanism_crate_root",
        "fret-ui root exports are the public mechanism substrate",
    ),
)

DEFAULT_FORBIDDEN_PATTERNS: tuple[tuple[str, str], ...] = (
    (
        r"\buse\s+fret_ui::",
        "default app/tutorial surfaces must not import `fret_ui`; use `fret::app::prelude::*` or an explicit advanced/component lane",
    ),
    (
        r"\buse\s+fret_core::",
        "default app/tutorial surfaces must not import `fret_core`; use the curated `fret` facade exports",
    ),
    (
        r"\bfret_canvas::",
        "default app/tutorial surfaces must not import `fret_canvas`; use the explicit `fret::canvas` app facade",
    ),
    (
        r"\bfret_chart::",
        "default app/tutorial surfaces must not import `fret_chart`; use the explicit `fret::chart` app facade",
    ),
    (
        r"\bfret_app::",
        "default app/tutorial surfaces must not import `fret_app`; use `fret::app` or `fret::commands` facade exports",
    ),
    (
        r"\bfret_runtime::",
        "default app/tutorial surfaces must not import `fret_runtime`; use app-facing `LocalState`, actions, and data helpers",
    ),
    (
        r"\bFnDriver\b",
        "`FnDriver` is an advanced/manual assembly seam, not a default app authoring noun",
    ),
    (
        r"\bUiTree\b",
        "`UiTree` is a retained runtime mechanism, not a default app authoring noun",
    ),
    (
        r"\bElementContext\b",
        "`ElementContext` is a runtime/component seam; default app helpers should prefer `AppUi`, `AppRenderContext`, or typed children",
    ),
    (
        r"\bfret::advanced\b",
        "`fret::advanced` must stay off default app/tutorial surfaces",
    ),
    (
        r"\bfret::advanced::prelude::\*",
        "`fret::advanced::prelude::*` must stay off default app/tutorial surfaces",
    ),
    (
        r"\badvanced::prelude::\*",
        "`advanced::prelude::*` must stay off default app/tutorial surfaces",
    ),
    (
        r"\bAppUiRawActionNotifyExt\b",
        "`AppUiRawActionNotifyExt` is an advanced/raw action hook, not a default app authoring helper",
    ),
    (
        r"\bcx\.on_(payload_)?action_notify::<",
        "default app surfaces should use `cx.actions()` helpers instead of raw `on_action_notify` hooks",
    ),
    (
        r"\bLocalState::new_in\b",
        "default app surfaces should use `app.local_state(...)` in init or `cx.state().local*` in render",
    ),
    (
        r"\bModelStore\b",
        "`ModelStore` is a raw runtime seam; default app surfaces should use app-facing state/action/data helpers",
    ),
    (
        r"\bUiPointerActionHost\b",
        "`UiPointerActionHost` is a raw pointer mechanism; default app surfaces should use `fret::pointer` helpers",
    ),
    (
        r"\bPointerRegionProps\b",
        "`PointerRegionProps` is a mechanism prop bag; default app surfaces should use `fret::pointer::PointerRegion`",
    ),
    (
        r"\bDefaultAction::FocusOnPointerDown\b",
        "`DefaultAction::FocusOnPointerDown` is a raw runtime default; default app surfaces should use `PointerActionCx::prevent_focus_on_pointer_down`",
    ),
)

RAW_SEAM_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("fret::advanced", re.compile(r"\bfret::advanced\b|\badvanced::prelude::\*")),
    ("fret_app", re.compile(r"\bfret_app::")),
    ("fret_core", re.compile(r"\bfret_core::")),
    ("fret_launch", re.compile(r"\bfret_launch::")),
    ("fret_runtime", re.compile(r"\bfret_runtime::")),
    ("fret_ui", re.compile(r"\bfret_ui::")),
    ("AnyElement", re.compile(r"\bAnyElement\b")),
    ("ElementContext", re.compile(r"\bElementContext\b")),
    ("FnDriver", re.compile(r"\bFnDriver\b")),
    ("ModelStore", re.compile(r"\bModelStore\b")),
    ("UiActionHostAdapter", re.compile(r"\bUiActionHostAdapter\b")),
    ("UiTree", re.compile(r"\bUiTree\b")),
)

PUBLIC_EXAMPLE_SCAN_ROOTS: tuple[str, ...] = (
    "apps/fret-cookbook/examples",
    "apps/fret-examples/src/lib.rs",
    "apps/fret-examples/src/api_workbench_lite_demo.rs",
    "apps/fret-examples/src/hello_world_compare_demo.rs",
    "apps/fret-examples/src/echarts_demo.rs",
    "apps/fret-examples/src/echarts_multi_grid_demo.rs",
    "apps/fret-examples/src/chart_multi_axis_demo.rs",
    "apps/fret-examples/src/chart_stress_demo.rs",
    "apps/fret-examples/src/custom_effect_v2_web_demo.rs",
    "apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs",
    "apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs",
    "apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs",
    "apps/fret-examples/src/imui_editor_proof_demo/authoring_parity",
    "apps/fret-examples/src/simple_todo_demo.rs",
    "apps/fret-examples/src/todo_demo.rs",
    "apps/fret-examples/src/components_gallery.rs",
    "apps/fret-examples/src/docking_demo.rs",
    "apps/fret-examples/src/docking_arbitration_demo.rs",
    "apps/fret-examples/src/plot_demo.rs",
    "apps/fret-examples/src/plot_stress_demo.rs",
    "apps/fret-examples/src/gizmo3d_demo.rs",
)

PUBLIC_EXAMPLE_CLASSIFICATION_PATTERNS: tuple[tuple[str, re.Pattern[str], str], ...] = (
    (
        "advanced-facade",
        re.compile(r"\bfret::advanced\b|\badvanced::prelude::\*|\badvanced::raw\b"),
        "`fret::advanced` usage in public examples must be classified as default-clean or advanced/manual",
    ),
    (
        "manual-kernel-app",
        re.compile(r"\bKernelApp\b|\bAppWindowId\b"),
        "`KernelApp` and `AppWindowId` are manual runtime nouns and require an advanced/manual classification",
    ),
    (
        "manual-driver",
        re.compile(r"\bFnDriver\b|\bfret_launch::"),
        "`FnDriver` and direct launch imports are manual driver seams and require classification",
    ),
    (
        "raw-model-store",
        re.compile(r"\bModelStore\b|\bLocalState::new_in\b|\bModel<"),
        "raw model-store/model usage in public examples must be classified or replaced with app-facing state helpers",
    ),
    (
        "retained-ui-tree",
        re.compile(r"\bUiTree\b"),
        "`UiTree` is a retained runtime mechanism and public examples must classify direct usage",
    ),
)

POLICY_CODED_EXPORT_TERMS: tuple[str, ...] = (
    "Dialog",
    "Popover",
    "Menu",
    "Tooltip",
    "Dismiss",
    "Dismissable",
    "AutoFocus",
    "FocusTrap",
    "RovingFocus",
    "Typeahead",
    "ScrollDismiss",
    "HoverIntent",
    "ResizablePanelGroup",
)

# Terms in this list are denied from root/default authoring surfaces. Some of them, such as
# `RovingFocus` and `Typeahead`, may still appear inside `crates/fret-ui` mechanism members when
# the runtime only forwards interaction facts and component crates own the policy.
MECHANISM_ROOT_EXPORT_CLASSIFICATIONS: dict[str, str] = {}

MECHANISM_PUBLIC_MEMBER_FORBIDDEN_PATTERNS: tuple[tuple[str, re.Pattern[str], str], ...] = (
    (
        "scroll-dismiss",
        re.compile(r"\bpub\s+fn\s+\w*scroll_dismiss\w*\b"),
        "`fret-ui` public layer APIs must use mechanism vocabulary such as `scroll_observer`, not scroll-dismiss policy names",
    ),
    (
        "dismiss-action-hook",
        re.compile(
            r"\bpub\s+(enum|struct|type)\s+(DismissReason|DismissRequestCx|OnDismissRequest|OnDismissiblePointerMove)\b"
        ),
        "`fret-ui` action hook APIs must use mechanism vocabulary such as `LayerInteraction*`; Radix dismiss naming belongs in `fret-ui-kit`",
    ),
    (
        "dismissible-public-member",
        re.compile(r"(?i)\bpub\s+(fn|struct|enum|type|mod)\s+\w*dismissible\w*\b"),
        "`fret-ui` public mechanism APIs must use mechanism vocabulary such as `LayerInteraction*`; Dismissible naming belongs in `fret-ui-kit`",
    ),
    (
        "auto-focus-action-hook",
        re.compile(
            r"\bpub\s+(struct|type)\s+(AutoFocusRequestCx|OnOpenAutoFocus|OnCloseAutoFocus)\b"
        ),
        "`fret-ui` focus handoff APIs must use mechanism vocabulary such as `FocusHandoff*`; Radix auto-focus naming belongs in `fret-ui-kit`",
    ),
)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _iter_source_files(path: Path) -> list[Path]:
    if path.is_file():
        return [path] if path.suffix in {".md", ".rs"} else []
    if not path.is_dir():
        return []
    return sorted(
        p
        for p in path.rglob("*")
        if p.is_file() and p.suffix in {".md", ".rs"} and ".git" not in p.parts
    )


def _code_lines_for_scan(path: Path, text: str) -> list[tuple[int, str]]:
    if path.suffix != ".md":
        return list(enumerate(text.splitlines(), start=1))

    lines: list[tuple[int, str]] = []
    in_fence = False
    include_fence = False
    fence_re = re.compile(r"^\s*```([A-Za-z0-9_-]*)")
    for line_no, line in enumerate(text.splitlines(), start=1):
        match = fence_re.match(line)
        if match:
            if in_fence:
                in_fence = False
                include_fence = False
            else:
                lang = match.group(1).lower()
                in_fence = True
                include_fence = lang in {"", "rust", "rs"}
            continue
        if in_fence and include_fence:
            lines.append((line_no, line))
    return lines


def _is_rust_source_line_ignorable(line: str) -> bool:
    stripped = line.strip()
    return stripped.startswith("//") or stripped.startswith("///") or stripped.startswith("//!")


def _scan_default_authoring_surface(root: Path, spec: SurfacePath) -> list[SurfaceViolation]:
    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        for line_no, line in _code_lines_for_scan(path, text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for pattern, message in DEFAULT_FORBIDDEN_PATTERNS:
                if re.search(pattern, line):
                    violations.append(
                        SurfaceViolation(
                            rule="default-app-clean",
                            path=path,
                            line_no=line_no,
                            message=message,
                            source=line.strip(),
                        )
                    )
                    break
    return violations


def _classified_raw_rule_prefix(spec: SurfacePath) -> str:
    return "advanced-surface" if spec.category == "advanced_manual" else spec.category


def _scan_classified_raw_surface(root: Path, spec: SurfacePath) -> list[SurfaceViolation]:
    violations: list[SurfaceViolation] = []
    allowed_raw_seams = set(spec.allowed_raw_seams)
    used_raw_seams: set[str] = set()
    rule_prefix = _classified_raw_rule_prefix(spec)
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        for line_no, line in _code_lines_for_scan(path, text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in RAW_SEAM_PATTERNS:
                if not pattern.search(line):
                    continue
                used_raw_seams.add(seam)
                if seam in allowed_raw_seams:
                    continue
                violations.append(
                    SurfaceViolation(
                        rule=f"{rule_prefix}-unlisted-raw-seam",
                        path=path,
                        line_no=line_no,
                        message=(
                            f"{spec.category} surface uses raw seam `{seam}` without listing it "
                            "in the classification record's allowed_raw_seams"
                        ),
                        source=line.strip(),
                    )
                )
    for seam in sorted(allowed_raw_seams - used_raw_seams):
        violations.append(
            SurfaceViolation(
                rule=f"{rule_prefix}-unused-allowed-raw-seam",
                path=root / spec.path,
                line_no=1,
                message=(
                    f"{spec.category} surface lists raw seam `{seam}` in allowed_raw_seams, "
                    "but the seam is no longer used; shrink the classification record"
                ),
            )
        )
    return violations


def _collect_root_public_statements(text: str) -> list[tuple[int, str]]:
    public_text = text.split("#[cfg(test)]\nmod ", 1)[0]
    statements: list[tuple[int, str]] = []
    collecting = False
    start_line = 0
    parts: list[str] = []

    for line_no, line in enumerate(public_text.splitlines(), start=1):
        stripped = line.strip()
        if not collecting:
            if not re.match(r"^(#\[[^\]]+\]\s*)?pub\s+(use|type|struct|enum|trait|fn|mod)\b", stripped):
                continue
            collecting = True
            start_line = line_no
            parts = [stripped]
        else:
            parts.append(stripped)

        if ";" in stripped or stripped.endswith("{"):
            statements.append((start_line, " ".join(parts)))
            collecting = False
            start_line = 0
            parts = []

    if collecting and parts:
        statements.append((start_line, " ".join(parts)))
    return statements


def _exported_symbols(statement: str) -> set[str]:
    words = set(re.findall(r"\b[A-Z][A-Za-z0-9_]*\b", statement))
    module_match = re.search(r"\bpub\s+mod\s+([a-zA-Z0-9_]+)\b", statement)
    if module_match:
        words.add(module_match.group(1))
    return words


def _is_policy_coded_statement(statement: str) -> bool:
    return any(term in statement for term in POLICY_CODED_EXPORT_TERMS)


def _classification_reason(symbols: Iterable[str]) -> str | None:
    for symbol in symbols:
        reason = MECHANISM_ROOT_EXPORT_CLASSIFICATIONS.get(symbol)
        if reason:
            return reason
    return None


def _scan_mechanism_root(root: Path, spec: SurfacePath) -> list[SurfaceViolation]:
    path = root / spec.path
    if not path.exists():
        return []
    violations: list[SurfaceViolation] = []
    for line_no, statement in _collect_root_public_statements(_read_text(path)):
        if not _is_policy_coded_statement(statement):
            continue
        symbols = _exported_symbols(statement)
        if _classification_reason(symbols):
            continue
        term = next(term for term in POLICY_CODED_EXPORT_TERMS if term in statement)
        violations.append(
            SurfaceViolation(
                rule="mechanism-root-policy-vocabulary",
                path=path,
                line_no=line_no,
                message=(
                    f"`crates/fret-ui` root export contains policy-coded term `{term}` without "
                    "an explicit mechanism/compat classification"
                ),
                source=statement,
            )
        )
    return violations


def _scan_mechanism_public_members(root: Path) -> list[SurfaceViolation]:
    path = root / "crates/fret-ui/src"
    if not path.exists():
        return []
    violations: list[SurfaceViolation] = []
    for source_path in _iter_source_files(path):
        for line_no, line in _code_lines_for_scan(source_path, _read_text(source_path)):
            for name, pattern, message in MECHANISM_PUBLIC_MEMBER_FORBIDDEN_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule=f"mechanism-public-member-policy-vocabulary:{name}",
                        path=source_path,
                        line_no=line_no,
                        message=message,
                        source=line.strip(),
                    )
                )
    return violations


def _validate_surface_specs(specs: Sequence[SurfacePath]) -> list[SurfaceViolation]:
    violations: list[SurfaceViolation] = []
    for spec in specs:
        if not spec.reason.strip():
            violations.append(
                SurfaceViolation(
                    rule="surface-classification-reason",
                    path=Path(spec.path),
                    line_no=1,
                    message=f"{spec.category} surface classification must include a reason",
                )
            )
        if spec.category not in CLASSIFIED_RAW_SURFACE_CATEGORIES:
            continue
        if not spec.owner.strip():
            violations.append(
                SurfaceViolation(
                    rule=f"{_classified_raw_rule_prefix(spec)}-classification-owner",
                    path=Path(spec.path),
                    line_no=1,
                    message=f"{spec.category} surface classification must include an owner",
                )
            )
        if spec.category == "advanced_manual" and not spec.retirement.strip():
            violations.append(
                SurfaceViolation(
                    rule="advanced-surface-classification-retirement",
                    path=Path(spec.path),
                    line_no=1,
                    message="advanced/manual surface classification must include a retirement condition",
                )
            )
        if not spec.allowed_raw_seams or any(not seam.strip() for seam in spec.allowed_raw_seams):
            violations.append(
                SurfaceViolation(
                    rule=f"{_classified_raw_rule_prefix(spec)}-classification-raw-seams",
                    path=Path(spec.path),
                    line_no=1,
                    message=f"{spec.category} surface classification must list allowed raw seams",
                )
            )
    for symbol, reason in MECHANISM_ROOT_EXPORT_CLASSIFICATIONS.items():
        if not reason.strip():
            violations.append(
                SurfaceViolation(
                    rule="surface-classification-reason",
                    path=Path("crates/fret-ui/src/lib.rs"),
                    line_no=1,
                    message=f"compat export `{symbol}` must include a reason",
                )
            )
    return violations


def _validate_existing_classified_surfaces(
    root: Path,
    specs: Sequence[SurfacePath],
) -> list[SurfaceViolation]:
    violations: list[SurfaceViolation] = []
    for spec in specs:
        path = root / spec.path
        if not path.exists():
            violations.append(
                SurfaceViolation(
                    rule="classified-surface-exists",
                    path=path,
                    line_no=1,
                    message=(
                        f"{spec.category} surface classification points to a missing path; "
                        "remove stale classifications or update the path"
                    ),
                )
            )
    return violations


def _path_is_covered_by_specs(root: Path, path: Path, specs: Sequence[SurfacePath]) -> bool:
    resolved_path = path.resolve()
    for spec in specs:
        resolved_spec_path = (root / spec.path).resolve()
        try:
            resolved_path.relative_to(resolved_spec_path)
            return True
        except ValueError:
            continue
    return False


def _scan_unclassified_public_examples(
    root: Path,
    scan_roots: Sequence[str],
    classified_specs: Sequence[SurfacePath],
) -> list[SurfaceViolation]:
    violations: list[SurfaceViolation] = []
    for scan_root in scan_roots:
        for path in _iter_source_files(root / scan_root):
            if _path_is_covered_by_specs(root, path, classified_specs):
                continue
            for line_no, line in _code_lines_for_scan(path, _read_text(path)):
                if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                    continue
                for seam, pattern, message in PUBLIC_EXAMPLE_CLASSIFICATION_PATTERNS:
                    if not pattern.search(line):
                        continue
                    violations.append(
                        SurfaceViolation(
                            rule="public-example-unclassified-raw-seam",
                            path=path,
                            line_no=line_no,
                            message=(
                                f"{message}. Public example uses high-risk seam `{seam}` without an "
                                "explicit surface classification. Add an app/default wrapper, "
                                "move the example to an explicit advanced lane, or add a "
                                "temporary quarantine record with owner and retirement."
                            ),
                            source=line.strip(),
                        )
                    )
                    break
    return violations


def check_surface_policy(
    root: Path,
    *,
    default_surfaces: Sequence[SurfacePath] = DEFAULT_AUTHORING_SURFACES,
    advanced_manual_surfaces: Sequence[SurfacePath] = ADVANCED_MANUAL_SURFACES,
    comparison_surfaces: Sequence[SurfacePath] = COMPARISON_SURFACES,
    internal_harness_surfaces: Sequence[SurfacePath] = INTERNAL_HARNESS_SURFACES,
    renderer_lab_surfaces: Sequence[SurfacePath] = RENDERER_LAB_SURFACES,
    policy_recipe_surfaces: Sequence[SurfacePath] = POLICY_RECIPE_SURFACES,
    mechanism_root_surfaces: Sequence[SurfacePath] = MECHANISM_ROOT_SURFACES,
    public_example_scan_roots: Sequence[str] = PUBLIC_EXAMPLE_SCAN_ROOTS,
) -> list[SurfaceViolation]:
    specs = [
        *default_surfaces,
        *advanced_manual_surfaces,
        *comparison_surfaces,
        *internal_harness_surfaces,
        *renderer_lab_surfaces,
        *policy_recipe_surfaces,
        *mechanism_root_surfaces,
    ]
    violations = _validate_surface_specs(specs)
    violations.extend(_validate_existing_classified_surfaces(root, specs))

    for spec in default_surfaces:
        violations.extend(_scan_default_authoring_surface(root, spec))

    for spec in advanced_manual_surfaces:
        violations.extend(_scan_classified_raw_surface(root, spec))

    for spec in comparison_surfaces:
        violations.extend(_scan_classified_raw_surface(root, spec))

    for spec in internal_harness_surfaces:
        violations.extend(_scan_classified_raw_surface(root, spec))

    for spec in renderer_lab_surfaces:
        violations.extend(_scan_classified_raw_surface(root, spec))

    violations.extend(
        _scan_unclassified_public_examples(
            root,
            public_example_scan_roots,
            [
                *default_surfaces,
                *advanced_manual_surfaces,
                *comparison_surfaces,
                *internal_harness_surfaces,
                *renderer_lab_surfaces,
            ],
        )
    )

    # Policy/recipe surfaces are intentionally classified here. Dependency direction and backend
    # leakage remain owned by `tools/check_layering.py`; this checker only prevents treating their
    # mechanism consumption as a default-app violation.
    _ = policy_recipe_surfaces

    for spec in mechanism_root_surfaces:
        violations.extend(_scan_mechanism_root(root, spec))
    violations.extend(_scan_mechanism_public_members(root))

    return violations


def _print_violations(root: Path, violations: Sequence[SurfaceViolation]) -> None:
    print(f"[gate] {GATE_NAME}")
    if not violations:
        print("[gate] ok")
        return

    print(f"[gate] FAIL: {len(violations)} violation(s)")
    for violation in violations[:80]:
        try:
            rel = violation.path.resolve().relative_to(root.resolve())
        except ValueError:
            rel = violation.path
        print(f"  - {rel}:{violation.line_no}: {violation.rule}")
        print(f"      {violation.message}")
        if violation.source:
            print(f"      {violation.source}")
    if len(violations) > 80:
        print(f"  ... and {len(violations) - 80} more")


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--root",
        type=Path,
        default=WORKSPACE_ROOT,
        help="repository root to scan (default: current Fret workspace root)",
    )
    args = parser.parse_args(argv)

    root = args.root.resolve()
    violations = check_surface_policy(root)
    _print_violations(root, violations)
    return 1 if violations else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
