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
    SurfacePath(
        "apps/fret-examples/src/todo_demo.rs",
        "default_app_clean",
        "todo demo should stay on app-facing style, text, env, and typed-action facades",
    ),
    SurfacePath(
        "apps/fret-examples/src/query_demo.rs",
        "default_app_clean",
        "default query/read-cache example should stay on app-facing data, time, and text facades",
    ),
    SurfacePath(
        "apps/fret-examples/src/query_async_tokio_demo.rs",
        "default_app_clean",
        "default async query example should stay on app-facing data, time, and text facades",
    ),
    SurfacePath(
        "apps/fret-examples/src/async_playground_demo.rs",
        "default_app_clean",
        "async query playground should stay on app-facing data, text, scroll, pressable, and element aliases",
    ),
    SurfacePath(
        "apps/fret-examples/src/plot_image_demo.rs",
        "default_app_clean",
        "plot image overlay example should stay on the default declarative plot app surface",
    ),
    SurfacePath(
        "apps/fret-examples/src/tags_demo.rs",
        "default_app_clean",
        "plot tags/text overlay example should stay on the default declarative plot app surface",
    ),
    SurfacePath(
        "apps/fret-examples/src/node_graph_demo.rs",
        "default_app_clean",
        "node graph example should stay on app-facing view, style, and node facades",
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


MANUAL_CHART_DEMO_FILENAMES = (
    "area_demo.rs",
    "bars_demo.rs",
    "candlestick_demo.rs",
    "category_line_demo.rs",
    "chart_demo.rs",
    "error_bars_demo.rs",
    "grouped_bars_demo.rs",
    "heatmap_demo.rs",
    "histogram2d_demo.rs",
    "histogram_demo.rs",
    "horizontal_bars_demo.rs",
    "inf_lines_demo.rs",
    "linked_cursor_demo.rs",
    "plot3d_demo.rs",
    "shaded_demo.rs",
    "stacked_bars_demo.rs",
    "stairs_demo.rs",
    "stems_demo.rs",
)

MANUAL_CHART_ALLOWED_RAW_SEAMS = (
    "fret_app",
    "fret_core",
    "fret_launch",
    "fret_runtime",
    "fret_ui",
    "FnDriver",
    "UiTree",
)


def _fret_examples_manual_chart_surface(filename: str) -> SurfacePath:
    stem = filename.removesuffix(".rs").replace("_", "-")
    return _fret_examples_advanced_surface(
        filename,
        (
            "it owns a manual retained-chart demo runner with FnDriver/UiTree lifecycle and "
            "direct chart viewport wiring"
        ),
        MANUAL_CHART_ALLOWED_RAW_SEAMS,
        owner=f"examples-chart-{stem}",
    )


CUSTOM_EFFECT_REFERENCE_DEMO_FILENAMES = (
    "custom_effect_v1_demo.rs",
    "custom_effect_v2_demo.rs",
    "custom_effect_v3_demo.rs",
    "custom_effect_v3_web_demo.rs",
)

CUSTOM_EFFECT_NATIVE_REFERENCE_ALLOWED_RAW_SEAMS = (
    "fret::advanced",
    "fret_core",
    "fret_ui",
    "AnyElement",
    "ElementContext",
)

CUSTOM_EFFECT_WEB_REFERENCE_ALLOWED_RAW_SEAMS = (
    "fret_app",
    "fret_core",
    "fret_launch",
    "fret_runtime",
    "FnDriver",
)


def _fret_examples_custom_effect_reference_surface(filename: str) -> SurfacePath:
    stem = filename.removesuffix(".rs").replace("_", "-")
    if filename == "custom_effect_v3_web_demo.rs":
        return _fret_examples_advanced_surface(
            filename,
            (
                "the custom-effect V3 web reference owns manual web runner/bootstrap, direct "
                "scene/effect composition, and validation for the bounded custom-effect contract"
            ),
            CUSTOM_EFFECT_WEB_REFERENCE_ALLOWED_RAW_SEAMS,
            owner=f"examples-{stem}",
        )

    variant = filename.removesuffix("_demo.rs").replace("_", " ")
    return _fret_examples_advanced_surface(
        filename,
        (
            f"the {variant} reference keeps explicit effect/runtime ownership to validate "
            "renderer/effect ABI wiring for the bounded custom-effect contract instead of "
            "teaching default app authoring"
        ),
        CUSTOM_EFFECT_NATIVE_REFERENCE_ALLOWED_RAW_SEAMS,
        owner=f"examples-{stem}",
    )


STREAMING_IMPORT_DEMO_FILENAMES = (
    "streaming_i420_demo.rs",
    "streaming_image_demo.rs",
    "streaming_nv12_demo.rs",
)

STREAMING_IMPORT_ALLOWED_RAW_SEAMS = (
    "fret_app",
    "fret_core",
    "fret_launch",
    "fret_runtime",
    "FnDriver",
)


def _fret_examples_streaming_import_surface(filename: str) -> SurfacePath:
    stem = filename.removesuffix(".rs").replace("_", "-")
    return _fret_examples_advanced_surface(
        filename,
        (
            "the streaming image upload proof owns manual FnDriver event/render hooks, direct "
            "scene image ops, and runtime image update effects"
        ),
        STREAMING_IMPORT_ALLOWED_RAW_SEAMS,
        owner=f"examples-{stem}",
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
    "UiTree",
)

CUSTOM_EFFECT_V2_WEB_OWNER_HELPER_ALLOWED_RAW_SEAMS = (
    "fret_core",
    "fret_runtime",
    "ModelStore",
)

CUSTOM_EFFECT_V2_WEB_OWNER = "examples-custom-effect-v2-web"

CUSTOM_EFFECT_V2_WEB_RETIREMENT = (
    "Remove this quarantine record after a public custom-effect parameter/control binding owns "
    "parameter models, reset/toggle actions, and effect-layer composition without exposing the "
    "raw ModelStore owner seam."
)

CUSTOM_EFFECT_V2_WEB_DEMO_REQUIRED_MARKERS = (
    "CustomEffectV2WebControlBinding",
    "CustomEffectV2ScalarControl",
    "CustomEffectV2ScalarSpec",
    "CustomEffectV2WebVariantControls",
    "impl CustomEffectV2WebVariantControls for DemoControls",
    "fn reset_variant_controls(",
    "CustomEffectV2ScalarSpec::new(",
    "binding: CustomEffectV2WebControlBinding",
    ".toggle_surface_in(",
    ".reset_controls_in(",
)

CUSTOM_EFFECT_V2_WEB_OWNER_HELPER_REQUIRED_MARKERS = (
    "struct CustomEffectV2ParamSlot",
    "struct CustomEffectV2ParamPack",
    "struct CustomEffectV2ScalarSpec",
    "struct CustomEffectV2ScalarControl",
    "struct CustomEffectV2WebControlBinding",
    "struct CustomEffectV2WebCommonControls",
    "struct CustomEffectV2WebModelOwner",
    "struct CustomEffectV2WebVariantReset",
    "trait CustomEffectV2WebVariantControls",
    "fn set_model",
    "fn toggle_surface_in",
    "fn reset_controls_in<C: CustomEffectV2WebVariantControls>",
    "self.models",
    ".update",
)

CUSTOM_EFFECT_V2_WEB_DIRECT_RAW_WRITE_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("models_mut().update", re.compile(r"\bmodels_mut\s*\(\s*\)\s*\.\s*update\s*\(")),
    ("ModelStore::update", re.compile(r"\bModelStore\s*::\s*update\s*\(")),
)

CUSTOM_EFFECT_V2_WEB_DEMO_FORBIDDEN_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    (
        "legacy-modelstore-alias",
        re.compile(r"\btype\s+CustomEffectV2\w*WebModelStore\s*="),
    ),
    (
        "legacy-local-owner",
        re.compile(r"\bstruct\s+CustomEffectV2\w*WebModelOwner\b"),
    ),
    (
        "shared-owner-constructor",
        re.compile(r"\bCustomEffectV2WebModelOwner\s*::\s*new\s*\("),
    ),
    (
        "shared-owner-import",
        re.compile(r"\bCustomEffectV2WebModelOwner\b"),
    ),
    (
        "legacy-reset-trait",
        re.compile(r"\bCustomEffectV2WebControlReset\b"),
    ),
    (
        "owner-set-model",
        re.compile(r"\bowner\s*\.\s*set_model\s*\("),
    ),
    (
        "standalone-show-model",
        re.compile(r"\bshow\s*:\s*(?:fret_runtime::)?Model\s*<\s*bool\s*>"),
    ),
    (
        "raw-scalar-model-field",
        re.compile(r"\bModel\s*<\s*Vec\s*<\s*f32\s*>\s*>"),
    ),
    (
        "raw-scalar-model-insert",
        re.compile(r"\bapp\s*\.\s*models_mut\s*\(\s*\)\s*\.\s*insert\s*\(\s*vec!\s*\["),
    ),
    (
        "variant-reset-set-model",
        re.compile(r"\breset\s*\.\s*set_model\s*\(\s*&\s*self\."),
    ),
)

GIZMO3D_OWNER = "examples-gizmo3d"

GIZMO3D_DEMO_REQUIRED_MARKERS = (
    "struct Gizmo3dDemoModelBinding",
    "model: fret_runtime::Model<Gizmo3dDemoModel>",
    "fn handle_viewport_input(",
    "fn step_frame_animation(",
    "fn frame_render_snapshot(",
    "model.handle_viewport_input(app, &event)",
    "state.demo.step_frame_animation(app, Instant::now())",
    "state.demo.frame_render_snapshot(app, size)",
)

GIZMO3D_DEMO_FORBIDDEN_RAW_UPDATE_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    (
        "state.demo.update",
        re.compile(r"\bstate\s*\.\s*demo\s*\.\s*update\s*\(\s*app\s*,"),
    ),
    (
        "demo.update",
        re.compile(r"\bdemo\s*\.\s*update\s*\(\s*app\s*,"),
    ),
    (
        "model.update",
        re.compile(r"\bmodel\s*\.\s*update\s*\(\s*app\s*,"),
    ),
)

EMBEDDED_VIEWPORT_OWNER = "examples-embedded-viewport"

EMBEDDED_VIEWPORT_DEMO_REQUIRED_MARKERS = (
    "struct EmbeddedViewportDemoModelOwner<'a>",
    "models: &'a mut ModelStore",
    "fn set_last_input(",
    ".update(&models.last_input",
    "EmbeddedViewportDemoModelOwner::new(app.models_mut()).set_last_input(",
    "fn record_embedded_viewport(",
    ".view_with_hooks::<EmbeddedViewportDemoView>(|d| d.drive_embedded_viewport())?",
)

EMBEDDED_VIEWPORT_DEMO_FORBIDDEN_RAW_WRITE_PATTERNS: tuple[
    tuple[str, re.Pattern[str]], ...
] = (
    (
        "models_mut().update",
        re.compile(r"\bmodels_mut\s*\(\s*\)\s*\.\s*update\s*\("),
    ),
    (
        "ModelStore::update",
        re.compile(r"\bModelStore\s*::\s*update\s*\("),
    ),
)

EXTERNAL_IMPORTS_OWNER = "examples-external-imports"

EXTERNAL_IMPORTS_OWNER_HELPER_REQUIRED_MARKERS = (
    "pub(crate) struct ExternalImportsModelOwner<'a>",
    "models: &'a mut ModelStore",
    "pub(crate) fn toggle_surface(",
    ".update(show, |show|",
)

EXTERNAL_IMPORTS_DEMO_REQUIRED_MARKERS = (
    "use crate::external_imports_owner::ExternalImportsModelOwner;",
    "ExternalImportsModelOwner::new(app.models_mut()).toggle_surface(",
)

EXTERNAL_IMPORTS_DEMO_FORBIDDEN_RAW_WRITE_PATTERNS: tuple[
    tuple[str, re.Pattern[str]], ...
] = (
    (
        "models_mut().update",
        re.compile(r"\bmodels_mut\s*\(\s*\)\s*\.\s*update\s*\("),
    ),
    (
        "ModelStore::update",
        re.compile(r"\bModelStore\s*::\s*update\s*\("),
    ),
)

HOTPATCH_SMOKE_OWNER = "demo-hotpatch-smoke"

HOTPATCH_SMOKE_REQUIRED_MARKERS = (
    "struct HotpatchSmokeModelOwner<'a>",
    "models: &'a mut ModelStore",
    "fn increment_counter(",
    "fn set_debug(",
    "HotpatchSmokeModelOwner::new(app.models_mut())",
    ".increment_counter(&state.counter)",
    ".set_debug(&state.debug, &msg)",
)

HOTPATCH_SMOKE_FORBIDDEN_RAW_WRITE_PATTERNS: tuple[
    tuple[str, re.Pattern[str]], ...
] = (
    (
        "models_mut().update",
        re.compile(
            r"\bmodels_mut\s*\(\s*\)\s*\.\s*update(?:_any)?\s*(?:::\s*<[^>]*>)?\s*\("
        ),
    ),
    (
        "ModelStore::update",
        re.compile(
            r"(?:\bModelStore\s*::\s*|<\s*ModelStore\s*>\s*::\s*)update(?:_any)?\s*(?:::\s*<[^>]*>)?\s*\("
        ),
    ),
)

DOCKING_ARBITRATION_OWNER = "examples-docking-arbitration"

DOCKING_ARBITRATION_REQUIRED_COMPACT_MARKERS = (
    "usefret_runtime::{ModelStore,PlatformCapabilities};",
    "structDockingArbitrationControls{",
    "structDockingArbitrationControlsService{",
    "letcontrols=DockingArbitrationControls::new(app.models_mut());",
    "DockingArbitrationControlsService::default",
    "svc.set(window,controls);",
    "letnext=controls.toggle_drop_mask_disallow_left_edge(host);",
    "controls.set_synth_pointer_debug(app,msg);",
    "controls.set_last_viewport_input(app,msg);",
    "if!synth.enabled&&pressed{return;}",
)

DOCKING_ARBITRATION_FORBIDDEN_COMPACT_MARKERS = (
    "structDockingArbitrationPanelModels",
    "DockingArbitrationPanelModelsService",
    "structViewportDebugService",
    "last_event:HashMap<AppWindowId,Model<Arc<str>>>",
    "structDockingArbitrationModelOwner",
    "DockingArbitrationModelOwner::new(host.models_mut()).toggle_drop_mask_disallow_left_edge(&drop_mask_disallow_left_edge);",
    "set_synth_pointer_debug(&models.synth_pointer_debug,msg)",
    "set_last_viewport_input(&model,msg.clone())",
)

DOCKING_ARBITRATION_FORBIDDEN_RAW_WRITE_PATTERNS: tuple[
    tuple[str, re.Pattern[str]], ...
] = (
    (
        "models_mut().update",
        re.compile(
            r"\bmodels_mut\s*\(\s*\)\s*\.\s*update(?:_any)?\s*(?:::\s*<[^>]*>)?\s*\("
        ),
    ),
    (
        "ModelStore::update",
        re.compile(
            r"(?:\bModelStore\s*::\s*|<\s*ModelStore\s*>\s*::\s*)update(?:_any)?\s*(?:::\s*<[^>]*>)?\s*\("
        ),
    ),
)

WORKSPACE_SHELL_OWNER = "examples-workspace-shell"

WORKSPACE_SHELL_DRIVER_REQUIRED_COMPACT_MARKERS = (
    "structWorkspaceShellModelBundle{",
    "fnnew(models:&mutModelStore,window_layout:WorkspaceWindowLayout,file_tree_items:Vec<TreeItem>,file_tree_state:TreeState,)->Self{",
    "letmodels=WorkspaceShellModelBundle::new(app.models_mut(),window_layout,items_value,state_value,);",
    "structWorkspaceShellModelOwner<'a>{",
    "models:&'amutModelStore,",
    "fnupdate<T:Any,R>(&mutself,model:&Model<T>,f:implFnOnce(&mutT)->R)->Option<R>{",
    "fnset<T:Any>(&mutself,model:&Model<T>,value:T)->bool{",
    "fnupdate_window_layout<R>(",
    "fnopen_dirty_close_prompt(",
    "fnclear_dirty_close_prompt(",
    "fntoggle_tabstrip_two_row_pinned(&mutself,model:&Model<bool>)->bool{",
    "fnworkspace_shell_update_window_layout<R>(",
    "fnworkspace_shell_open_dirty_close_prompt(",
    "fnworkspace_shell_clear_dirty_close_prompt(",
    "fnworkspace_shell_host_clear_dirty_close_prompt(",
    "workspace_shell_host_clear_dirty_close_prompt(host,&prompt_model,&open_model,);",
    "workspace_shell_open_dirty_close_prompt(app,state,WorkspaceShellDirtyClosePrompt::window_close(req),);",
    "workspace_shell_clear_dirty_close_prompt(app,state);",
    "WorkspaceShellModelOwner::new(app.models_mut()).toggle_tabstrip_two_row_pinned(&state.tabstrip_two_row_pinned);",
)

WORKSPACE_SHELL_DRIVER_FORBIDDEN_COMPACT_MARKERS = (
    "fnworkspace_shell_update_model",
    "fnworkspace_shell_host_update_model",
    "fnworkspace_shell_set_model",
    "fnworkspace_shell_host_set_model",
)

WORKSPACE_SHELL_DRIVER_FORBIDDEN_RAW_WRITE_PATTERNS: tuple[
    tuple[str, re.Pattern[str]], ...
] = (
    (
        "models_mut().update",
        re.compile(
            r"\bmodels_mut\s*\(\s*\)\s*\.\s*update(?:_any)?\s*(?:::\s*<[^>]*>)?\s*\("
        ),
    ),
    (
        "models_mut().insert",
        re.compile(r"\bmodels_mut\s*\(\s*\)\s*\.\s*insert\s*\("),
    ),
    (
        "ModelStore::update",
        re.compile(
            r"(?:\bModelStore\s*::\s*|<\s*ModelStore\s*>\s*::\s*)update(?:_any)?\s*(?:::\s*<[^>]*>)?\s*\("
        ),
    ),
)

WINDOW_HIT_TEST_PROBE_OWNER = "examples-window-hit-test-probe"

WINDOW_HIT_TEST_PROBE_REQUIRED_MARKERS = (
    "use fret::advanced::KernelApp;",
    "use fret::advanced::interop::run_native_with_compat_driver;",
    "use fret_bootstrap::ui_app_driver::{self, ViewElements};",
    "ui_app_driver::UiAppDriver::new(",
    "run_native_with_compat_driver(config, KernelApp::new(), driver)?;",
)

WINDOW_HIT_TEST_PROBE_FORBIDDEN_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    (
        "advanced::prelude",
        re.compile(r"\badvanced\s*::\s*prelude\s*::\s*\*"),
    ),
    (
        "component::prelude",
        re.compile(r"\bcomponent\s*::\s*prelude\s*::\s*\*"),
    ),
)

COMPONENTS_GALLERY_OWNER = "examples-components-gallery"

COMPONENTS_GALLERY_REQUIRED_MARKERS = (
    "struct ComponentsGalleryModelBundle",
    "fn new(models: &mut ModelStore",
    "ComponentsGalleryModelBundle::new(app.models_mut()",
    "struct ComponentsGalleryModelOwner<'a>",
    "models: &'a mut ModelStore",
    "fn update<T: Any, R>(",
    "fn set<T: Any>(",
    "fn set_last_action(",
    "fn open_command_palette(",
    "fn close_transient_surfaces(",
    "fn components_gallery_set_last_action(",
    "fn components_gallery_open_command_palette(",
    "fn components_gallery_close_transient_surfaces(",
    "components_gallery_close_transient_surfaces(app, state);",
    "components_gallery_open_command_palette(app, state);",
    "components_gallery_set_last_action(app, state, \"context_menu.action\");",
)

COMPONENTS_GALLERY_FORBIDDEN_RAW_WRITE_PATTERNS: tuple[
    tuple[str, re.Pattern[str]], ...
] = (
    (
        "models_mut().update",
        re.compile(r"\bmodels_mut\s*\(\s*\)\s*\.\s*update(?:_any)?\s*\("),
    ),
    (
        "ModelStore::update",
        re.compile(
            r"(?:\bModelStore\s*::\s*update(?:_any)?\s*\(|<\s*ModelStore\s*>\s*::\s*update(?:_any)?\s*\()"
        ),
    ),
    (
        "models_mut().insert",
        re.compile(r"\bmodels_mut\s*\(\s*\)\s*\.\s*insert\s*\("),
    ),
)

VIRTUAL_LIST_STRESS_OWNER = "examples-virtual-list-stress"

VIRTUAL_LIST_STRESS_REQUIRED_MARKERS = (
    "struct VirtualListStressControls",
    "tall_rows_enabled: Model<bool>",
    "reversed: Model<bool>",
    "items_revision: Model<u64>",
    "fn new(models: &mut ModelStore) -> Self",
    "fn toggle_rows_enabled(&self, models: &mut ModelStore) -> bool",
    "fn toggle_reversed_and_bump_revision(&self, models: &mut ModelStore) -> bool",
    "fn layout_snapshot(&self, cx: &mut ElementContext<'_, App>) -> VirtualListStressSnapshot",
    "let controls = VirtualListStressControls::new(app.models_mut());",
    "controls: VirtualListStressControls,",
    "state.controls.toggle_rows_enabled(app.models_mut())",
    "toggle_reversed_and_bump_revision(app.models_mut())",
    "let controls = state.controls.layout_snapshot(cx);",
)

VIRTUAL_LIST_STRESS_FORBIDDEN_RAW_WRITE_PATTERNS: tuple[
    tuple[str, re.Pattern[str]], ...
] = (
    (
        "models_mut().insert",
        re.compile(r"\bmodels_mut\s*\(\s*\)\s*\.\s*insert\s*\("),
    ),
    (
        "models_mut().update",
        re.compile(r"\bmodels_mut\s*\(\s*\)\s*\.\s*update(?:_any)?\s*\("),
    ),
    (
        "ModelStore::update",
        re.compile(
            r"(?:\bModelStore\s*::\s*update(?:_any)?\s*\(|<\s*ModelStore\s*>\s*::\s*update(?:_any)?\s*\()"
        ),
    ),
    (
        "legacy-state-model-field",
        re.compile(
            r"\b(tall_rows_enabled|reversed|items_revision)\s*:\s*fret_app\s*::\s*Model\s*<"
        ),
    ),
    (
        "legacy-state-model-reference",
        re.compile(r"&\s*state\s*\.\s*(?:tall_rows_enabled|reversed|items_revision)\b"),
    ),
    (
        "legacy-model-owner",
        re.compile(r"\bVirtualListStressModelOwner\b"),
    ),
    (
        "legacy-free-helper",
        re.compile(r"\bfn\s+virtual_list_stress_(?:update_model|toggle_model|bump_revision)\b"),
    ),
)

TABLE_STRESS_OWNER = "examples-table-stress"

TABLE_STRESS_REQUIRED_MARKERS = (
    "struct TableStressControls",
    "table_state: Model<TableState>",
    "items_revision: Model<u64>",
    "controls: TableStressControls,",
    "fn new(models: &mut ModelStore, row_count: usize) -> Self",
    "fn render_snapshot(&self, cx: &mut ElementContext<'_, App>) -> TableStressSnapshot",
    "let table_state = state.controls.table_model();",
    "let controls = state.controls.render_snapshot(cx);",
    "struct TableStressModelOwner<'a>",
    "models: &'a mut ModelStore",
    "fn update_table_state(",
    "fn toggle_sorting(&mut self, state: &Model<TableState>) -> bool",
    "fn toggle_role_filter(&mut self, state: &Model<TableState>) -> bool",
    "fn toggle_global_filter(&mut self, state: &Model<TableState>) -> bool",
    "fn clear_filters(&mut self, state: &Model<TableState>) -> bool",
    "fn bump_items_revision(&mut self, revision: &Model<u64>) -> bool",
    "TableStressModelOwner::new(app.models_mut()).toggle_sorting(&self.table_state)",
    "TableStressModelOwner::new(app.models_mut()).toggle_role_filter(&self.table_state)",
    "TableStressModelOwner::new(app.models_mut()).toggle_global_filter(&self.table_state)",
    "TableStressModelOwner::new(app.models_mut()).clear_filters(&self.table_state)",
    "TableStressModelOwner::new(app.models_mut()).bump_items_revision(&self.items_revision)",
    "state.controls.toggle_sorting(app)",
    "state.controls.toggle_role_filter(app)",
    "state.controls.toggle_global_filter(app)",
    "state.controls.clear_filters(app)",
    "state.controls.bump_items_revision(app)",
)

TABLE_STRESS_FORBIDDEN_RAW_WRITE_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    (
        "models_mut().insert",
        re.compile(r"\bmodels_mut\s*\(\s*\)\s*\.\s*insert\s*\("),
    ),
    (
        "models_mut().update",
        re.compile(r"\bmodels_mut\s*\(\s*\)\s*\.\s*update(?:_any)?\s*\("),
    ),
    (
        "ModelStore::update",
        re.compile(
            r"(?:\bModelStore\s*::\s*update(?:_any)?\s*\(|<\s*ModelStore\s*>\s*::\s*update(?:_any)?\s*\()"
        ),
    ),
    (
        "legacy-state-model-reference",
        re.compile(r"&\s*state\s*\.\s*(?:table_state|items_revision)\b"),
    ),
    (
        "legacy-driver-command",
        re.compile(
            r"\bTableStressDriver\s*::\s*(?:toggle_sorting|toggle_role_filter|toggle_global_filter|clear_filters|bump_items_revision)\b"
        ),
    ),
)

CANVAS_DATAGRID_STRESS_OWNER = "examples-canvas-datagrid-stress"

CANVAS_DATAGRID_STRESS_REQUIRED_COMPACT_MARKERS = (
    "usefret::app::{AppLocalStateExtas_,AppLocalStateTxnExtas_,LocalState};",
    "grid_output:LocalState<shadcn::DataGridCanvasOutput>,",
    "letgrid_output=app.local_state(shadcn::DataGridCanvasOutput::default());",
    "letgrid=app.local_state_txn(|tx|tx.value_or_default(&state.grid_output));",
    "letgrid=state.grid_output.layout_value(cx);",
    ".output_model(state.grid_output.clone())",
    "structCanvasDataGridStressControls{",
    "variable_sizes:Model<bool>,",
    "clamp_rows:Model<bool>,",
    "revision:Model<u64>,",
    "fnnew(app:&mutApp)->Self{",
    "letcontrols=CanvasDataGridStressControls::new(app);",
    "controls:CanvasDataGridStressControls,",
    "letcontrols=state.controls.layout_snapshot(cx);",
    "letmutaxis=shadcn::DataGridCanvasAxis::new(Arc::clone(&rows),controls.revision,Px(24.0)",
    "letmutaxis=shadcn::DataGridCanvasAxis::new(Arc::clone(&cols),controls.revision,Px(120.0)",
)

CANVAS_DATAGRID_STRESS_FORBIDDEN_COMPACT_PATTERNS: tuple[
    tuple[str, re.Pattern[str]], ...
] = (
    (
        "legacy-window-state-control-field",
        re.compile(
            r"structCanvasDataGridStressWindowState\{[^}]*variable_sizes:Model<bool>,[^}]*clamp_rows:Model<bool>,[^}]*revision:Model<u64>,"
        ),
    ),
    (
        "legacy-grid-output-model",
        re.compile(
            r"(?:grid_output:Model<shadcn::DataGridCanvasOutput>|app\.models_mut\(\)\.insert\(shadcn::DataGridCanvasOutput::default\(\)\))"
        ),
    ),
    (
        "legacy-state-control-reference",
        re.compile(r"&state\.(?:variable_sizes|clamp_rows|revision)\b"),
    ),
)

CANVAS_DATAGRID_STRESS_FORBIDDEN_RAW_WRITE_PATTERNS: tuple[
    tuple[str, re.Pattern[str]], ...
] = (
    (
        "direct-variable-size-model-insert",
        re.compile(
            r"\blet\s+variable_sizes\s*=\s*app\s*\.\s*models_mut\s*\(\s*\)\s*\.\s*insert\s*\("
        ),
    ),
    (
        "direct-clamp-rows-model-insert",
        re.compile(
            r"\blet\s+clamp_rows\s*=\s*app\s*\.\s*models_mut\s*\(\s*\)\s*\.\s*insert\s*\("
        ),
    ),
    (
        "direct-revision-model-insert",
        re.compile(
            r"\blet\s+revision\s*=\s*app\s*\.\s*models_mut\s*\(\s*\)\s*\.\s*insert\s*\("
        ),
    ),
)

DATATABLE_OWNER = "examples-datatable"

DATATABLE_REQUIRED_COMPACT_MARKERS = (
    "usefret::app::AppLocalStateExtas_;",
    "usefret::app::LocalState;",
    "table_output:LocalState<shadcn::DataTableViewOutput>,",
    "lettable_output=app.local_state(shadcn::DataTableViewOutput::default());",
    "lettable_output=state.table_output.clone();",
    "let_=table_output.layout_value(cx);",
    "shadcn::DataTablePagination::new(&table_state,table_output.clone())",
    ".output_model(table_output.clone())",
)

DATATABLE_FORBIDDEN_COMPACT_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    (
        "legacy-output-model",
        re.compile(r"Model<shadcn::DataTableViewOutput>"),
    ),
    (
        "legacy-output-model-insert",
        re.compile(r"app\.models_mut\(\)\.insert\(shadcn::DataTableViewOutput::default\(\)\)"),
    ),
    (
        "legacy-observe-model",
        re.compile(r"cx\.observe_model\(&table_output,Invalidation::Layout\);"),
    ),
    (
        "legacy-model-import",
        re.compile(r"usefret_app::\{App,CommandId,Effect,Model,WindowRequest\};"),
    ),
)

EDITOR_NOTES_OWNER = "examples-editor-notes"

EDITOR_NOTES_REQUIRED_COMPACT_MARKERS = (
    "structEditorAssetModels{",
    "name:Model<String>,",
    "notes:Model<String>,",
    "notes_outcome:Model<String>,",
    "summary_status:Model<String>,",
    "models:EditorAssetModels,",
    "structEditorThemePresetBinding{",
    "theme:EditorThemePresetBinding,",
    "fneditor_asset_paint_snapshot(",
    "structEditorNotesModelOwner<'a>{",
    "models:&'amutModelStore,",
    "fnset_text(&mutself,model:&Model<String>,value:implInto<String>)->bool{",
    "fnset_notes_outcome(&self,models:&mutModelStore,value:implInto<String>)->bool{",
    "fnset_summary_status(&self,models:&mutModelStore,value:implInto<String>)->bool{",
    "EditorNotesModelOwner::new(models).set_text(&self.notes_outcome,value)",
    "EditorNotesModelOwner::new(models).set_text(&self.summary_status,value)",
    "models.set_notes_outcome(host.models_mut(),next",
    "models.set_notes_outcome(host.models_mut(),\"Committed\"",
    "models.set_notes_outcome(host.models_mut(),\"Canceled\"",
    "models.set_summary_status(host.models_mut(),draft_commit_status.clone()",
    "models.set_summary_status(host.models_mut(),draft_discard_status.clone()",
    "models.set_summary_status(host.models_mut(),summary_status_next.clone()",
    "EditorThemePresetPicker::new(theme.picker_model())",
    "EditorThemePresetBinding::new(app)",
    "editor_asset_paint_snapshot(cx,&asset)",
)

EDITOR_NOTES_FORBIDDEN_RAW_WRITE_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    (
        "models_mut().update",
        re.compile(r"\bmodels_mut\s*\(\s*\)\s*\.\s*update(?:_any)?\s*\("),
    ),
    (
        "ModelStore::update",
        re.compile(
            r"(?:\bModelStore\s*::\s*update(?:_any)?\s*\(|<\s*ModelStore\s*>\s*::\s*update(?:_any)?\s*\()"
        ),
    ),
    (
        "legacy-public-model-field",
        re.compile(
            r"\bpub\s*\(\s*crate\s*\)\s*(?:name|notes|notes_outcome|summary_status)_model\s*:\s*Model\s*<"
        ),
    ),
    (
        "legacy-theme-model-field",
        re.compile(r"\btheme_preset_model\s*:\s*Model\s*<"),
    ),
    (
        "legacy-model-field",
        re.compile(
            r"\b(?:asset\s*\.\s*)?(?:name|notes|notes_outcome|summary_status)_model\b"
        ),
    ),
    (
        "legacy-free-helper",
        re.compile(r"\bfn\s+editor_notes_host_(?:update_model|set_model|set_text)\b"),
    ),
)


def _fret_examples_custom_effect_v2_web_surface(filename: str, variant: str) -> SurfacePath:
    return SurfacePath(
        f"apps/fret-examples/src/{filename}",
        "advanced_manual",
        (
            f"{filename} remains classified as an advanced examples surface because the {variant} "
            "custom-effect v2 web proof owns manual runner/bootstrap, binding-backed common "
            "parameter controls, variant-specific scalar control bindings, and low-level "
            "effect-layer composition"
        ),
        owner=CUSTOM_EFFECT_V2_WEB_OWNER,
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


def _fret_examples_renderer_lab_surface(
    path: str,
    reason: str,
    allowed_raw_seams: tuple[str, ...],
    owner: str | None = None,
) -> SurfacePath:
    stem = path.removesuffix(".rs").replace("/", "-").replace("_", "-")
    return SurfacePath(
        f"apps/fret-examples/src/{path}",
        "renderer_lab",
        f"{path} remains classified as a renderer lab because {reason}",
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
    SurfacePath(
        "apps/fret-examples/src/custom_effect_v2_web_owner.rs",
        "internal_harness",
        "shared private owner helper for custom-effect v2 web demo raw ModelStore writes",
        owner=CUSTOM_EFFECT_V2_WEB_OWNER,
        allowed_raw_seams=CUSTOM_EFFECT_V2_WEB_OWNER_HELPER_ALLOWED_RAW_SEAMS,
    ),
    SurfacePath(
        "apps/fret-examples/src/external_imports_owner.rs",
        "internal_harness",
        "shared private owner helper for external import visibility writes",
        owner=EXTERNAL_IMPORTS_OWNER,
        allowed_raw_seams=("fret_runtime", "ModelStore"),
    ),
    SurfacePath(
        "apps/fret-demo/src/bin/hotpatch_smoke_demo.rs",
        "internal_harness",
        "dev-only hotpatch maintainer smoke harness with manual driver hooks and owner-routed model writes",
        owner=HOTPATCH_SMOKE_OWNER,
        allowed_raw_seams=(
            "fret_app",
            "fret_core",
            "fret_runtime",
            "fret_ui",
            "ElementContext",
            "ModelStore",
            "UiTree",
        ),
    ),
    _fret_examples_internal_harness(
        "lib.rs",
        "the examples crate root owns shared native/web harness helpers, launch glue, and theme "
        "interop helpers for demo shells",
        ("fret::advanced", "fret_app", "fret_core", "fret_launch", "fret_runtime"),
        owner="examples-harness-root",
    ),
    _fret_examples_internal_harness(
        "first_frame_smoke_demo.rs",
        "the first-frame smoke harness owns manual FnDriver startup, explicit render hooks, and "
        "auto-close behavior for launch/backend validation",
        ("fret_app", "fret_core", "fret_launch", "FnDriver"),
        owner="examples-first-frame-smoke",
    ),
    _fret_examples_internal_harness(
        "cjk_conformance_demo.rs",
        "the CJK text conformance harness owns manual UiTree rendering, font catalog inspection, "
        "hot reload hooks, and direct frame layout/paint validation",
        (
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "AnyElement",
            "FnDriver",
            "UiTree",
        ),
        owner="examples-cjk-conformance",
    ),
    _fret_examples_internal_harness(
        "emoji_conformance_demo.rs",
        "the emoji text conformance harness owns manual UiTree rendering, font catalog "
        "inspection, local font override controls, hot reload hooks, and direct frame "
        "layout/paint validation",
        (
            "fret::advanced",
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "AnyElement",
            "FnDriver",
            "UiTree",
        ),
        owner="examples-emoji-conformance",
    ),
    _fret_examples_internal_harness(
        "ime_smoke_demo.rs",
        "the IME conformance smoke harness owns manual UiTree rendering, command/keymap glue, "
        "IME event recording, hot reload hooks, and direct frame layout/paint validation",
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
        owner="examples-ime-conformance-smoke",
    ),
    _fret_examples_internal_harness(
        "image_heavy_memory_demo.rs",
        "the image-heavy memory harness owns GPU image allocation, upload/drop controls, "
        "engine-frame hooks, and retained image rendering for memory diagnostics",
        (
            "fret::advanced",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "UiTree",
        ),
        owner="examples-image-heavy-memory",
    ),
    _fret_examples_internal_harness(
        "text_heavy_memory_demo.rs",
        "the text-heavy memory harness deliberately grows shaping caches and color emoji atlas "
        "pressure while using low-level text/container mechanisms",
        ("fret_core", "fret_ui", "ElementContext"),
        owner="examples-text-heavy-memory",
    ),
    _fret_examples_internal_harness(
        "extras_marquee_perf_demo.rs",
        "the extras marquee perf harness owns advanced KernelApp startup and low-level text "
        "composition for diagnostics-focused perf probing",
        (
            "fret::advanced",
            "fret_core",
            "fret_ui",
            "AnyElement",
            "ElementContext",
        ),
        owner="examples-extras-marquee-perf",
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
            "ModelStore",
            "UiTree",
        ),
        owner=DOCKING_ARBITRATION_OWNER,
    ),
    _fret_examples_internal_harness(
        "plot_stress_demo.rs",
        "the plot stress harness owns manual driver state and env-driven perf controls, while "
        "plot model mutation and panel wiring route through LinePlotPanelBinding",
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
        "the chart stress harness owns manual driver state and env-driven perf controls, while "
        "chart engine statistics and panel wiring route through ChartCanvasPanelBinding",
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
        "virtual_list_stress_demo.rs",
        "the virtual-list stress harness owns manual driver state, renderer perf hooks, and "
        "env-driven scroll controls, while shared model allocation, command writes, and render "
        "snapshots route through VirtualListStressControls",
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
            "ModelStore",
            "UiTree",
        ),
        owner=VIRTUAL_LIST_STRESS_OWNER,
    ),
    _fret_examples_internal_harness(
        "table_stress_demo.rs",
        "the table stress harness owns manual driver state, renderer/allocation perf hooks, and "
        "retained table plumbing with app text roles, while shared model allocation, command writes, render "
        "subscriptions, and readout snapshots route through TableStressControls and "
        "TableStressModelOwner",
        (
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "ElementContext",
            "FnDriver",
            "ModelStore",
            "UiTree",
        ),
        owner=TABLE_STRESS_OWNER,
    ),
    _fret_examples_internal_harness(
        "canvas_datagrid_stress_demo.rs",
        "the canvas datagrid stress harness owns manual driver state, renderer perf hooks, "
        "app-facing LocalState grid telemetry, and retained stress controls behind "
        "CanvasDataGridStressControls",
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
        owner=CANVAS_DATAGRID_STRESS_OWNER,
    ),
    _fret_examples_internal_harness(
        "container_queries_docking_demo.rs",
        "the container-query docking proof owns manual FnDriver, UiTree, diagnostics/script "
        "driving, docking runtime, and accessibility bridge hooks",
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
        owner="examples-container-query-docking-harness",
    ),
    _fret_examples_internal_harness(
        "simple_todo_demo/driver.rs",
        "the simple-todo driver module owns native/web compatibility launch signatures for demo shells",
        ("fret_launch",),
        owner="examples-simple-todo-driver",
    ),
    _fret_examples_internal_harness(
        "plot_image_demo/driver.rs",
        "the plot-image driver module owns native/web compatibility launch signatures for demo shells",
        ("fret_launch",),
        owner="examples-plot-image-driver",
    ),
    _fret_examples_internal_harness(
        "tags_demo/driver.rs",
        "the plot-tags driver module owns native/web compatibility launch signatures for demo shells",
        ("fret_launch",),
        owner="examples-plot-tags-driver",
    ),
    _fret_examples_internal_harness(
        "todo_demo_runtime_tests.rs",
        "the todo demo runtime test harness owns direct view-runtime rendering, cache transition "
        "checks, and fake UI services for semantics snapshots",
        (
            "fret::advanced",
            "fret_core",
            "fret_runtime",
            "fret_ui",
            "UiTree",
        ),
        owner="examples-todo-runtime-tests",
    ),
)


RENDERER_LAB_SURFACES: tuple[SurfacePath, ...] = (
    _fret_examples_renderer_lab_surface(
        "alpha_mode_demo.rs",
        "it validates straight-vs-premultiplied alpha upload and renderer image compositing "
        "semantics through manual FnDriver scene ops",
        (
            "fret::advanced",
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "FnDriver",
        ),
        owner="examples-alpha-mode-renderer-lab",
    ),
    _fret_examples_renderer_lab_surface(
        "drop_shadow_demo.rs",
        "it is a DropShadowV1 renderer semantics and perf-baseline surface rather than an "
        "app-authoring lesson",
        ("fret_core", "fret_ui", "AnyElement", "ElementContext"),
        owner="examples-drop-shadow-renderer-lab",
    ),
    _fret_examples_renderer_lab_surface(
        "effects_demo.rs",
        "it owns manual FnDriver renderer hooks, direct SceneOp effect composition, and "
        "env-driven renderer perf reporting rather than an app-authoring lesson",
        ("fret_app", "fret_core", "fret_launch", "FnDriver"),
        owner="examples-effects-renderer-lab",
    ),
    _fret_examples_renderer_lab_surface(
        "image_upload_demo.rs",
        "it validates keyed image asset upload, eviction, and direct SceneOp image rendering "
        "through manual FnDriver hooks",
        ("fret::advanced", "fret_app", "fret_core", "fret_launch", "FnDriver"),
        owner="examples-image-upload-renderer-lab",
    ),
    _fret_examples_renderer_lab_surface(
        "assets_demo.rs",
        "it is a retained ADR probe for UiAppDriver ImageAssetCache/SvgAssetCache GPU-ready "
        "integration, including advanced SVG registration hooks, rather than a first-contact "
        "asset authoring lesson",
        ("fret::advanced",),
        owner="examples-assets-cache-renderer-lab",
    ),
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
        owner=WORKSPACE_SHELL_OWNER,
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
            "ModelStore",
            "UiTree",
        ),
        retirement=(
            "Replace with a public workspace-shell starter once AppUi wrappers own command, "
            "overlay, virtual-list, diagnostics, and window lifecycle flows"
        ),
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
            "ModelStore",
            "UiTree",
        ),
        owner=COMPONENTS_GALLERY_OWNER,
    ),
    _fret_examples_advanced_surface(
        "datatable_demo.rs",
        "the datatable demo owns manual driver/table harness plumbing while keeping DataTable "
        "output on app-facing LocalState instead of raw shared model output handles",
        (
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "FnDriver",
            "UiTree",
        ),
        owner=DATATABLE_OWNER,
    ),
    *(
        _fret_examples_manual_chart_surface(filename)
        for filename in MANUAL_CHART_DEMO_FILENAMES
    ),
    *(
        _fret_examples_custom_effect_reference_surface(filename)
        for filename in CUSTOM_EFFECT_REFERENCE_DEMO_FILENAMES
    ),
    _fret_examples_advanced_surface(
        "liquid_glass_demo.rs",
        "the liquid-glass reference owns explicit renderer capability checks, backdrop warp/custom "
        "effect composition, and effect/control graph validation for the backdrop-warp contract",
        (
            "fret::advanced",
            "fret_core",
            "fret_runtime",
            "fret_ui",
            "AnyElement",
            "ElementContext",
        ),
        owner="examples-liquid-glass-reference",
    ),
    _fret_examples_advanced_surface(
        "postprocess_theme_demo.rs",
        "the theme postprocess reference owns explicit renderer/theme bridge custom-effect "
        "composition and high-ceiling postprocess controls",
        (
            "fret::advanced",
            "fret_core",
            "fret_ui",
        ),
        owner="examples-postprocess-theme-reference",
    ),
    *(
        _fret_examples_streaming_import_surface(filename)
        for filename in STREAMING_IMPORT_DEMO_FILENAMES
    ),
    _fret_examples_advanced_surface(
        "embedded_viewport_demo.rs",
        "the embedded viewport interop proof owns explicit advanced driver hooks, render target "
        "interop, and a small demo-local model owner for forwarded-input readouts",
        (
            "fret::advanced",
            "fret_core",
            "fret_runtime",
            "fret_ui",
            "AnyElement",
            "ModelStore",
        ),
        owner=EMBEDDED_VIEWPORT_OWNER,
    ),
    _fret_examples_advanced_surface(
        "launcher_utility_window_demo.rs",
        "the utility-window proof owns manual UiAppDriver hooks, frameless window style effects, "
        "drag/resize pointer regions, timers, and raw window diagnostics",
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
        owner="examples-launcher-utility-window",
    ),
    _fret_examples_advanced_surface(
        "launcher_utility_window_materials_demo.rs",
        "the utility-window materials proof owns manual UiAppDriver hooks, background material "
        "window style requests, and raw window diagnostics",
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
        owner="examples-launcher-utility-window-materials",
    ),
    _fret_examples_advanced_surface(
        "date_picker_demo.rs",
        "the shadcn date-picker behavior proof owns a manual FnDriver/AppUi render-root bridge, "
        "retained UiTree lifecycle, overlay state, app text roles, and low-level container composition",
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
        owner="examples-shadcn-date-picker-demo",
    ),
    _fret_examples_advanced_surface(
        "form_demo.rs",
        "the shadcn form behavior proof owns a manual FnDriver/AppUi render-root bridge, "
        "retained UiTree lifecycle, form registry validation, and overlay state",
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
        owner="examples-shadcn-form-demo",
    ),
    _fret_examples_advanced_surface(
        "table_demo.rs",
        "the shadcn table behavior/perf proof owns a manual FnDriver runner, retained UiTree, "
        "virtual-list scroll handle, grouping/header-menu state, and app text table cells",
        (
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "FnDriver",
            "UiTree",
        ),
        owner="examples-shadcn-table-demo",
    ),
    _fret_examples_advanced_surface(
        "sonner_demo.rs",
        "the shadcn sonner/toast behavior proof owns a manual FnDriver/AppUi render-root bridge, "
        "retained UiTree lifecycle, raw action host adapter plumbing, and toast promise lifecycle",
        (
            "fret::advanced",
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "FnDriver",
            "UiActionHostAdapter",
            "UiTree",
        ),
        owner="examples-shadcn-sonner-demo",
    ),
    _fret_examples_advanced_surface(
        "drag_demo.rs",
        "the plot drag-overlay proof owns manual FnDriver/UiTree runner hooks, plot binding "
        "state mutation, and direct pointer-event driven overlay updates",
        (
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "FnDriver",
            "UiTree",
        ),
        owner="examples-plot-drag-demo",
    ),
    _fret_examples_advanced_surface(
        "markdown_demo.rs",
        "the native markdown proof owns remote image fetch/decode, markdown-specific low-level "
        "image/SVG props and pressable image-placeholder affordances",
        (
            "fret::advanced",
            "fret_core",
            "fret_ui",
        ),
        owner="examples-markdown-demo",
    ),
    _fret_examples_advanced_surface(
        "genui_demo.rs",
        "the GenUI reference deliberately owns generator/runtime/editor integration, raw model "
        "ownership, action queue plumbing, and host-generic render helper boundaries",
        (
            "fret::advanced",
            "fret_core",
            "fret_runtime",
            "fret_ui",
            "AnyElement",
            "ElementContext",
            "ModelStore",
            "UiActionHostAdapter",
        ),
        owner="examples-genui-demo",
    ),
    _fret_examples_advanced_surface(
        "imui_editor_proof_demo.rs",
        "the IMUI editor proof owns editor-grade multi-window/docking/embedded-viewport runtime "
        "integration, GPU viewport bridges, and low-level editor proof helper surfaces",
        (
            "fret::advanced",
            "fret_app",
            "fret_core",
            "fret_runtime",
            "fret_ui",
            "AnyElement",
        ),
        owner="examples-imui-editor-proof",
    ),
    _fret_examples_advanced_surface(
        "imui_node_graph_demo.rs",
        "the IMUI node-graph compatibility proof intentionally retains raw Model handles and "
        "retained node-graph bridge wiring until declarative node-graph surfaces replace it",
        (
            "fret_runtime",
        ),
        owner="examples-imui-node-graph-compat",
    ),
    _fret_examples_advanced_surface(
        "editor_notes_demo.rs",
        "the editor notes demo owns editor app model bindings, shell-mounted rails, and theme "
        "preset wiring while keeping app-facing writes behind EditorAssetModels, "
        "EditorNotesModelOwner, and EditorThemePresetBinding",
        (
            "fret_app",
            "fret_core",
            "fret_runtime",
            "fret_ui",
            "AnyElement",
            "ModelStore",
        ),
        owner=EDITOR_NOTES_OWNER,
    ),
    _fret_examples_advanced_surface(
        "editor_notes_device_shell_demo.rs",
        "the editor notes device-shell demo owns adaptive shell composition and reuses the "
        "editor notes asset/theme bindings across desktop rails and mobile drawer surfaces",
        (
            "fret_core",
            "fret_ui",
        ),
        owner="examples-editor-notes-device-shell",
    ),
    _fret_examples_advanced_surface(
        "external_texture_imports_demo.rs",
        "the native external texture import proof owns advanced view hooks, launch import targets, "
        "raw renderer interop, and a shared private visibility owner",
        (
            "fret::advanced",
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "ElementContext",
            "UiTree",
        ),
        owner=EXTERNAL_IMPORTS_OWNER,
    ),
    _fret_examples_advanced_surface(
        "external_texture_imports_web_demo.rs",
        "the web external texture import proof owns a manual web runner, retained UiTree, launch "
        "import targets, raw renderer interop, and a shared private visibility owner",
        (
            "fret::advanced",
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "ElementContext",
            "FnDriver",
            "UiTree",
        ),
        owner=EXTERNAL_IMPORTS_OWNER,
    ),
    _fret_examples_advanced_surface(
        "external_video_imports_avf_demo.rs",
        "the AVFoundation external video import proof owns advanced view hooks, launch import "
        "targets, raw renderer interop, and a shared private visibility owner",
        (
            "fret::advanced",
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "ElementContext",
            "UiTree",
        ),
        owner=EXTERNAL_IMPORTS_OWNER,
    ),
    _fret_examples_advanced_surface(
        "external_video_imports_mf_demo.rs",
        "the Media Foundation external video import proof owns advanced view hooks, launch import "
        "targets, raw renderer interop, and a shared private visibility owner",
        (
            "fret::advanced",
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "ElementContext",
            "UiTree",
        ),
        owner=EXTERNAL_IMPORTS_OWNER,
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
        "window_hit_test_probe_demo.rs",
        "the hit-test passthrough probe owns manual compatibility-driver startup, explicit "
        "KernelApp window creation, and runtime window-style diagnostics",
        (
            "fret::advanced",
            "fret_app",
            "fret_core",
            "fret_launch",
            "fret_runtime",
            "fret_ui",
            "AnyElement",
            "ElementContext",
        ),
        owner=WINDOW_HIT_TEST_PROBE_OWNER,
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
        "the linked multi-axis proof still owns manual runner/bootstrap seams, while linked "
        "chart engine, output, brush, axis-pointer, and domain-window model wiring is routed "
        "through ChartCanvasLinkedGroupBinding and ChartCanvasLinkedPanelBinding",
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
    "apps/fret-examples/src/query_demo.rs",
    "apps/fret-examples/src/query_async_tokio_demo.rs",
    "apps/fret-examples/src/assets_demo.rs",
    "apps/fret-examples/src/container_queries_docking_demo.rs",
    "apps/fret-examples/src/launcher_utility_window_demo.rs",
    "apps/fret-examples/src/launcher_utility_window_materials_demo.rs",
    "apps/fret-examples/src/async_playground_demo.rs",
    "apps/fret-examples/src/date_picker_demo.rs",
    "apps/fret-examples/src/form_demo.rs",
    "apps/fret-examples/src/table_demo.rs",
    "apps/fret-examples/src/sonner_demo.rs",
    "apps/fret-examples/src/drag_demo.rs",
    "apps/fret-examples/src/plot_image_demo.rs",
    "apps/fret-examples/src/tags_demo.rs",
    "apps/fret-examples/src/markdown_demo.rs",
    "apps/fret-examples/src/genui_demo.rs",
    "apps/fret-examples/src/imui_editor_proof_demo.rs",
    "apps/fret-examples/src/imui_node_graph_demo.rs",
    "apps/fret-examples/src/echarts_demo.rs",
    "apps/fret-examples/src/echarts_multi_grid_demo.rs",
    "apps/fret-examples/src/chart_multi_axis_demo.rs",
    "apps/fret-examples/src/chart_stress_demo.rs",
    "apps/fret-examples/src/virtual_list_stress_demo.rs",
    "apps/fret-examples/src/table_stress_demo.rs",
    "apps/fret-examples/src/canvas_datagrid_stress_demo.rs",
    "apps/fret-examples/src/datatable_demo.rs",
    *(
        f"apps/fret-examples/src/{filename}"
        for filename in MANUAL_CHART_DEMO_FILENAMES
    ),
    *(
        f"apps/fret-examples/src/{filename}"
        for filename in CUSTOM_EFFECT_REFERENCE_DEMO_FILENAMES
    ),
    *(
        f"apps/fret-examples/src/{filename}"
        for filename in STREAMING_IMPORT_DEMO_FILENAMES
    ),
    "apps/fret-examples/src/effects_demo.rs",
    "apps/fret-examples/src/first_frame_smoke_demo.rs",
    "apps/fret-examples/src/alpha_mode_demo.rs",
    "apps/fret-examples/src/drop_shadow_demo.rs",
    "apps/fret-examples/src/image_upload_demo.rs",
    "apps/fret-examples/src/cjk_conformance_demo.rs",
    "apps/fret-examples/src/emoji_conformance_demo.rs",
    "apps/fret-examples/src/ime_smoke_demo.rs",
    "apps/fret-examples/src/image_heavy_memory_demo.rs",
    "apps/fret-examples/src/text_heavy_memory_demo.rs",
    "apps/fret-examples/src/extras_marquee_perf_demo.rs",
    "apps/fret-examples/src/liquid_glass_demo.rs",
    "apps/fret-examples/src/postprocess_theme_demo.rs",
    "apps/fret-examples/src/custom_effect_v2_web_demo.rs",
    "apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs",
    "apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs",
    "apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs",
    "apps/fret-examples/src/imui_editor_proof_demo/authoring_parity",
    "apps/fret-examples/src/simple_todo_demo.rs",
    "apps/fret-examples/src/todo_demo.rs",
    "apps/fret-examples/src/components_gallery.rs",
    "apps/fret-examples/src/embedded_viewport_demo.rs",
    "apps/fret-examples/src/editor_notes_demo.rs",
    "apps/fret-examples/src/editor_notes_device_shell_demo.rs",
    "apps/fret-examples/src/external_texture_imports_demo.rs",
    "apps/fret-examples/src/external_texture_imports_web_demo.rs",
    "apps/fret-examples/src/external_video_imports_avf_demo.rs",
    "apps/fret-examples/src/external_video_imports_mf_demo.rs",
    "apps/fret-examples/src/docking_demo.rs",
    "apps/fret-examples/src/docking_arbitration_demo.rs",
    "apps/fret-examples/src/window_hit_test_probe_demo.rs",
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


def _compact_source(text: str) -> str:
    return "".join(text.split())


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


def _code_lines_for_scan(
    path: Path, text: str, *, skip_rust_cfg_test_modules: bool = False
) -> list[tuple[int, str]]:
    if path.suffix == ".rs" and skip_rust_cfg_test_modules:
        return _rust_lines_for_scan(text)
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


def _rust_lines_for_scan(text: str) -> list[tuple[int, str]]:
    lines: list[tuple[int, str]] = []
    pending_cfg_test = False
    skipping_cfg_test_mod = False
    skip_brace_depth = 0

    for line_no, line in enumerate(text.splitlines(), start=1):
        stripped = line.strip()
        if skipping_cfg_test_mod:
            skip_brace_depth += line.count("{") - line.count("}")
            if skip_brace_depth <= 0:
                skipping_cfg_test_mod = False
            continue

        if re.fullmatch(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]", stripped):
            pending_cfg_test = True
            continue

        if pending_cfg_test:
            if re.match(r"(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{", stripped):
                skip_brace_depth = line.count("{") - line.count("}")
                if skip_brace_depth > 0:
                    skipping_cfg_test_mod = True
                pending_cfg_test = False
                continue
            if stripped.startswith("#["):
                continue
            pending_cfg_test = False

        lines.append((line_no, line))

    return lines


def _is_rust_source_line_ignorable(line: str) -> bool:
    stripped = line.strip()
    return stripped.startswith("//") or stripped.startswith("///") or stripped.startswith("//!")


def _scan_custom_effect_v2_web_owner_boundary(
    root: Path, spec: SurfacePath
) -> list[SurfaceViolation]:
    if spec.owner != CUSTOM_EFFECT_V2_WEB_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        if path.name == "custom_effect_v2_web_owner.rs":
            required_markers = CUSTOM_EFFECT_V2_WEB_OWNER_HELPER_REQUIRED_MARKERS
        else:
            required_markers = CUSTOM_EFFECT_V2_WEB_DEMO_REQUIRED_MARKERS
        missing_markers = [
            marker
            for marker in required_markers
            if marker not in text
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="advanced-surface-custom-effect-owner-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "custom-effect v2 web surfaces must keep common controls behind the "
                        f"shared private binding helper; missing binding markers: {', '.join(missing_markers)}"
                    ),
                )
            )

        if path.name != "custom_effect_v2_web_owner.rs":
            for marker, pattern in CUSTOM_EFFECT_V2_WEB_DEMO_FORBIDDEN_PATTERNS:
                if not pattern.search(text):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="advanced-surface-custom-effect-owner-boundary",
                        path=path,
                        line_no=1,
                        message=(
                            "custom-effect v2 web demos must use the shared private binding "
                            f"helper; legacy per-demo owner marker `{marker}` is not allowed"
                        ),
                    )
                )

        for line_no, line in _code_lines_for_scan(path, text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in CUSTOM_EFFECT_V2_WEB_DIRECT_RAW_WRITE_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="advanced-surface-custom-effect-owner-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "custom-effect v2 web raw model writes must go through the shared "
                            f"private binding helper; direct `{seam}` bypasses the reset/toggle binding boundary"
                        ),
                        source=line.strip(),
                    )
                )
                break

    return violations


def _scan_gizmo3d_owner_boundary(root: Path, spec: SurfacePath) -> list[SurfaceViolation]:
    if spec.owner != GIZMO3D_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        missing_markers = [
            marker for marker in GIZMO3D_DEMO_REQUIRED_MARKERS if marker not in text
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="advanced-surface-gizmo3d-owner-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "gizmo3d demo model access must stay behind "
                        f"Gizmo3dDemoModelBinding; missing binding markers: {', '.join(missing_markers)}"
                    ),
                )
            )

        for line_no, line in _code_lines_for_scan(path, text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in GIZMO3D_DEMO_FORBIDDEN_RAW_UPDATE_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="advanced-surface-gizmo3d-owner-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "gizmo3d app/driver model writes must use named "
                            f"Gizmo3dDemoModelBinding methods; direct `{seam}` bypasses the owner boundary"
                        ),
                        source=line.strip(),
                    )
                )
                break

    return violations


def _scan_embedded_viewport_owner_boundary(
    root: Path, spec: SurfacePath
) -> list[SurfaceViolation]:
    if spec.owner != EMBEDDED_VIEWPORT_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        missing_markers = [
            marker
            for marker in EMBEDDED_VIEWPORT_DEMO_REQUIRED_MARKERS
            if marker not in text
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="advanced-surface-embedded-viewport-owner-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "embedded viewport demo model writes must stay behind "
                        "EmbeddedViewportDemoModelOwner; missing owner markers: "
                        f"{', '.join(missing_markers)}"
                    ),
                )
            )

        for line_no, line in _code_lines_for_scan(path, text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in EMBEDDED_VIEWPORT_DEMO_FORBIDDEN_RAW_WRITE_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="advanced-surface-embedded-viewport-owner-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "embedded viewport app/driver writes must use "
                            f"EmbeddedViewportDemoModelOwner; direct `{seam}` bypasses the owner boundary"
                        ),
                        source=line.strip(),
                    )
                )
                break

    return violations


def _scan_external_imports_owner_boundary(
    root: Path, spec: SurfacePath
) -> list[SurfaceViolation]:
    if spec.owner != EXTERNAL_IMPORTS_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        if path.name == "external_imports_owner.rs":
            required_markers = EXTERNAL_IMPORTS_OWNER_HELPER_REQUIRED_MARKERS
        else:
            required_markers = EXTERNAL_IMPORTS_DEMO_REQUIRED_MARKERS
        missing_markers = [
            marker for marker in required_markers if marker not in text
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="advanced-surface-external-imports-owner-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "external import visibility writes must stay behind "
                        f"ExternalImportsModelOwner; missing owner markers: {', '.join(missing_markers)}"
                    ),
                )
            )

        if path.name == "external_imports_owner.rs":
            continue

        for line_no, line in _code_lines_for_scan(path, text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in EXTERNAL_IMPORTS_DEMO_FORBIDDEN_RAW_WRITE_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="advanced-surface-external-imports-owner-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "external import app/driver visibility writes must use "
                            f"ExternalImportsModelOwner; direct `{seam}` bypasses the owner boundary"
                        ),
                        source=line.strip(),
                    )
                )
                break

    return violations


def _scan_hotpatch_smoke_owner_boundary(
    root: Path, spec: SurfacePath
) -> list[SurfaceViolation]:
    if spec.owner != HOTPATCH_SMOKE_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        production_text = text.split("#[cfg(test)]", 1)[0]
        missing_markers = [
            marker for marker in HOTPATCH_SMOKE_REQUIRED_MARKERS if marker not in production_text
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="internal_harness-hotpatch-smoke-owner-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "hotpatch smoke event/command writes must stay behind "
                        f"HotpatchSmokeModelOwner; missing markers: {', '.join(missing_markers)}"
                    ),
                )
            )

        for line_no, line in _code_lines_for_scan(path, production_text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in HOTPATCH_SMOKE_FORBIDDEN_RAW_WRITE_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="internal_harness-hotpatch-smoke-owner-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "hotpatch smoke event/command writes must use "
                            f"HotpatchSmokeModelOwner; direct `{seam}` bypasses the owner boundary"
                        ),
                        source=line.strip(),
                    )
                )
                break

    return violations


def _scan_docking_arbitration_controls_boundary(
    root: Path, spec: SurfacePath
) -> list[SurfaceViolation]:
    if spec.owner != DOCKING_ARBITRATION_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        production_text = text.split("#[cfg(test)]", 1)[0]
        compact_production = _compact_source(production_text)
        missing_markers = [
            marker
            for marker in DOCKING_ARBITRATION_REQUIRED_COMPACT_MARKERS
            if marker not in compact_production
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="internal_harness-docking-arbitration-controls-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "docking arbitration diagnostic model writes must stay behind "
                        f"DockingArbitrationControls/Service; missing compact markers: "
                        f"{', '.join(missing_markers)}"
                    ),
                )
            )

        for marker in DOCKING_ARBITRATION_FORBIDDEN_COMPACT_MARKERS:
            if marker not in compact_production:
                continue
            violations.append(
                SurfaceViolation(
                    rule="internal_harness-docking-arbitration-controls-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "docking arbitration diagnostic state must not regress to legacy "
                        f"panel/debug model owners; compact `{marker}` bypasses the controls boundary"
                    ),
                )
            )

        for line_no, line in _code_lines_for_scan(path, production_text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in DOCKING_ARBITRATION_FORBIDDEN_RAW_WRITE_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="internal_harness-docking-arbitration-controls-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "docking arbitration diagnostic model writes must route through "
                            f"DockingArbitrationControls; direct `{seam}` bypasses the controls boundary"
                        ),
                        source=line.strip(),
                    )
                )
                break

        model_store_aliases: list[str] = []
        for statement in production_text.split(";"):
            compact_statement = _compact_source(statement)
            rest = compact_statement.removeprefix("let")
            if rest == compact_statement:
                continue
            if "=" not in rest:
                continue
            alias, rhs = rest.split("=", 1)
            if not re.fullmatch(r"(?:[A-Za-z_][A-Za-z0-9_]*\.)?models_mut\(\)", rhs):
                continue
            model_store_aliases.append(alias.strip().removeprefix("mut"))

        for alias in model_store_aliases:
            for forbidden in (
                f"{alias}.update(",
                f"{alias}.update::<",
                f"{alias}.update_any(",
                f"{alias}.update_any::<",
            ):
                if forbidden not in compact_production:
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="internal_harness-docking-arbitration-controls-boundary",
                        path=path,
                        line_no=1,
                        message=(
                            "docking arbitration diagnostic model writes must route through "
                            f"DockingArbitrationControls; ModelStore alias `{forbidden}` "
                            "bypasses the controls boundary"
                        ),
                    )
                )

    return violations


def _scan_workspace_shell_driver_owner_boundary(
    root: Path, spec: SurfacePath
) -> list[SurfaceViolation]:
    if spec.owner != WORKSPACE_SHELL_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        if path.name != "driver.rs":
            continue

        text = _read_text(path)
        production_text = text.split("#[cfg(test)]", 1)[0]
        compact_production = _compact_source(production_text)
        missing_markers = [
            marker
            for marker in WORKSPACE_SHELL_DRIVER_REQUIRED_COMPACT_MARKERS
            if marker not in compact_production
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="advanced-surface-workspace-shell-driver-owner-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "workspace shell driver model allocation and writes must stay behind "
                        f"WorkspaceShellModelBundle/Owner; missing compact markers: "
                        f"{', '.join(missing_markers)}"
                    ),
                )
            )

        for marker in WORKSPACE_SHELL_DRIVER_FORBIDDEN_COMPACT_MARKERS:
            if marker not in compact_production:
                continue
            violations.append(
                SurfaceViolation(
                    rule="advanced-surface-workspace-shell-driver-owner-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "workspace shell driver must not regress to raw helper or direct "
                        f"ModelStore allocation seams; compact `{marker}` bypasses the owner boundary"
                    ),
                )
            )

        for line_no, line in _code_lines_for_scan(path, production_text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in WORKSPACE_SHELL_DRIVER_FORBIDDEN_RAW_WRITE_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="advanced-surface-workspace-shell-driver-owner-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "workspace shell driver model allocation and writes must route through "
                            f"WorkspaceShellModelBundle/Owner; direct `{seam}` bypasses the owner boundary"
                        ),
                        source=line.strip(),
                    )
                )
                break

        model_store_aliases: list[str] = []
        for statement in production_text.split(";"):
            compact_statement = _compact_source(statement)
            rest = compact_statement.removeprefix("let")
            if rest == compact_statement:
                continue
            if "=" not in rest:
                continue
            alias, rhs = rest.split("=", 1)
            if not re.fullmatch(r"(?:[A-Za-z_][A-Za-z0-9_]*\.)?models_mut\(\)", rhs):
                continue
            model_store_aliases.append(alias.strip().removeprefix("mut"))

        for alias in model_store_aliases:
            for forbidden in (
                f"{alias}.update(",
                f"{alias}.update::<",
                f"{alias}.update_any(",
                f"{alias}.update_any::<",
                f"{alias}.insert(",
            ):
                if forbidden not in compact_production:
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="advanced-surface-workspace-shell-driver-owner-boundary",
                        path=path,
                        line_no=1,
                        message=(
                            "workspace shell driver model allocation and writes must route through "
                            f"WorkspaceShellModelBundle/Owner; ModelStore alias `{forbidden}` "
                            "bypasses the owner boundary"
                        ),
                    )
                )

    return violations


def _scan_window_hit_test_probe_boundary(
    root: Path, spec: SurfacePath
) -> list[SurfaceViolation]:
    if spec.owner != WINDOW_HIT_TEST_PROBE_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        missing_markers = [
            marker for marker in WINDOW_HIT_TEST_PROBE_REQUIRED_MARKERS if marker not in text
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="advanced-surface-window-hit-test-probe-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "window hit-test probe must keep manual driver seams explicit; "
                        f"missing markers: {', '.join(missing_markers)}"
                    ),
                )
            )

        for line_no, line in _code_lines_for_scan(path, text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for marker, pattern in WINDOW_HIT_TEST_PROBE_FORBIDDEN_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="advanced-surface-window-hit-test-probe-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "window hit-test probe must not hide manual runtime seams behind "
                            f"`{marker}::*`; import the required driver/kernel nouns explicitly"
                        ),
                        source=line.strip(),
                    )
                )
                break

    return violations


def _scan_components_gallery_owner_boundary(
    root: Path, spec: SurfacePath
) -> list[SurfaceViolation]:
    if spec.owner != COMPONENTS_GALLERY_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        production_text = text.split("#[cfg(test)]", 1)[0]
        missing_markers = [
            marker
            for marker in COMPONENTS_GALLERY_REQUIRED_MARKERS
            if marker not in production_text
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="advanced-surface-components-gallery-owner-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "components gallery model writes and startup allocation must stay behind "
                        f"ComponentsGalleryModelBundle/Owner; missing markers: {', '.join(missing_markers)}"
                    ),
                )
            )

        for line_no, line in _code_lines_for_scan(path, production_text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in COMPONENTS_GALLERY_FORBIDDEN_RAW_WRITE_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="advanced-surface-components-gallery-owner-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "components gallery app/driver model writes must use "
                            f"ComponentsGalleryModelBundle/Owner; direct `{seam}` bypasses the owner boundary"
                        ),
                        source=line.strip(),
                    )
                )
                break

    return violations


def _scan_virtual_list_stress_controls_boundary(
    root: Path, spec: SurfacePath
) -> list[SurfaceViolation]:
    if spec.owner != VIRTUAL_LIST_STRESS_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        production_text = text.split("#[cfg(test)]", 1)[0]
        missing_markers = [
            marker
            for marker in VIRTUAL_LIST_STRESS_REQUIRED_MARKERS
            if marker not in production_text
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="internal_harness-virtual-list-stress-controls-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "virtual-list stress shared models must stay behind "
                        f"VirtualListStressControls; missing markers: {', '.join(missing_markers)}"
                    ),
                )
            )

        for line_no, line in _code_lines_for_scan(path, production_text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in VIRTUAL_LIST_STRESS_FORBIDDEN_RAW_WRITE_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="internal_harness-virtual-list-stress-controls-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "virtual-list stress startup allocation, command writes, and render "
                            f"snapshots must route through VirtualListStressControls; direct `{seam}` "
                            "bypasses the controls boundary"
                        ),
                        source=line.strip(),
                    )
                )
                break

    return violations


def _scan_table_stress_controls_boundary(
    root: Path, spec: SurfacePath
) -> list[SurfaceViolation]:
    if spec.owner != TABLE_STRESS_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        production_text = text.split("#[cfg(test)]", 1)[0]
        missing_markers = [
            marker for marker in TABLE_STRESS_REQUIRED_MARKERS if marker not in production_text
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="internal_harness-table-stress-controls-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "table stress shared models must stay behind TableStressControls and "
                        f"TableStressModelOwner; missing markers: {', '.join(missing_markers)}"
                    ),
                )
            )

        for line_no, line in _code_lines_for_scan(path, production_text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in TABLE_STRESS_FORBIDDEN_RAW_WRITE_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="internal_harness-table-stress-controls-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "table stress startup allocation, command writes, render "
                            "subscriptions, and readout snapshots must route through "
                            f"TableStressControls/Owner; direct `{seam}` bypasses the controls boundary"
                        ),
                        source=line.strip(),
                    )
                )
                break

    return violations


def _scan_canvas_datagrid_stress_controls_boundary(
    root: Path, spec: SurfacePath
) -> list[SurfaceViolation]:
    if spec.owner != CANVAS_DATAGRID_STRESS_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        production_text = text.split("#[cfg(test)]", 1)[0]
        compact_production = _compact_source(production_text)
        missing_markers = [
            marker
            for marker in CANVAS_DATAGRID_STRESS_REQUIRED_COMPACT_MARKERS
            if marker not in compact_production
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="internal_harness-canvas-datagrid-stress-controls-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "canvas datagrid stress telemetry and retained controls must stay behind "
                        f"LocalState and CanvasDataGridStressControls; missing compact markers: "
                        f"{', '.join(missing_markers)}"
                    ),
                )
            )

        for seam, pattern in CANVAS_DATAGRID_STRESS_FORBIDDEN_COMPACT_PATTERNS:
            if not pattern.search(compact_production):
                continue
            violations.append(
                SurfaceViolation(
                    rule="internal_harness-canvas-datagrid-stress-controls-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "canvas datagrid stress grid telemetry must use LocalState and stress "
                        f"controls must stay bundled; compact `{seam}` bypasses the controls boundary"
                    ),
                )
            )

        for line_no, line in _code_lines_for_scan(path, production_text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in CANVAS_DATAGRID_STRESS_FORBIDDEN_RAW_WRITE_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="internal_harness-canvas-datagrid-stress-controls-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "canvas datagrid stress retained control model allocation must stay "
                            f"inside CanvasDataGridStressControls; direct `{seam}` bypasses the controls boundary"
                        ),
                        source=line.strip(),
                    )
                )
                break

    return violations


def _scan_datatable_output_boundary(root: Path, spec: SurfacePath) -> list[SurfaceViolation]:
    if spec.owner != DATATABLE_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        production_text = text.split("#[cfg(test)]", 1)[0]
        compact_production = _compact_source(production_text)
        missing_markers = [
            marker
            for marker in DATATABLE_REQUIRED_COMPACT_MARKERS
            if marker not in compact_production
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="advanced-surface-datatable-output-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "datatable output must stay on app-facing LocalState; "
                        f"missing compact markers: {', '.join(missing_markers)}"
                    ),
                )
            )

        for seam, pattern in DATATABLE_FORBIDDEN_COMPACT_PATTERNS:
            if not pattern.search(compact_production):
                continue
            violations.append(
                SurfaceViolation(
                    rule="advanced-surface-datatable-output-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "datatable output must not regress to raw shared model plumbing; "
                        f"compact `{seam}` bypasses the LocalState boundary"
                    ),
                )
            )

    return violations


def _scan_editor_notes_bindings_boundary(
    root: Path, spec: SurfacePath
) -> list[SurfaceViolation]:
    if spec.owner != EDITOR_NOTES_OWNER:
        return []

    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        production_text = text.split("#[cfg(test)]", 1)[0]
        compact_production = _compact_source(production_text)
        missing_markers = [
            marker
            for marker in EDITOR_NOTES_REQUIRED_COMPACT_MARKERS
            if marker not in compact_production
        ]
        if missing_markers:
            violations.append(
                SurfaceViolation(
                    rule="advanced-surface-editor-notes-bindings-boundary",
                    path=path,
                    line_no=1,
                    message=(
                        "editor notes model and theme state must stay behind "
                        f"EditorAssetModels/EditorNotesModelOwner/EditorThemePresetBinding; "
                        f"missing compact markers: {', '.join(missing_markers)}"
                    ),
                )
            )

        for line_no, line in _code_lines_for_scan(path, production_text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for seam, pattern in EDITOR_NOTES_FORBIDDEN_RAW_WRITE_PATTERNS:
                if not pattern.search(line):
                    continue
                violations.append(
                    SurfaceViolation(
                        rule="advanced-surface-editor-notes-bindings-boundary",
                        path=path,
                        line_no=line_no,
                        message=(
                            "editor notes app writes and shared model exposure must route through "
                            f"EditorAssetModels/EditorNotesModelOwner/EditorThemePresetBinding; "
                            f"direct `{seam}` bypasses the binding boundary"
                        ),
                        source=line.strip(),
                    )
                )
                break

    return violations


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
        skip_rust_cfg_test_modules = path.name == "custom_effect_v2_web_owner.rs"
        for line_no, line in _code_lines_for_scan(
            path, text, skip_rust_cfg_test_modules=skip_rust_cfg_test_modules
        ):
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
    violations.extend(_scan_custom_effect_v2_web_owner_boundary(root, spec))
    violations.extend(_scan_gizmo3d_owner_boundary(root, spec))
    violations.extend(_scan_embedded_viewport_owner_boundary(root, spec))
    violations.extend(_scan_external_imports_owner_boundary(root, spec))
    violations.extend(_scan_hotpatch_smoke_owner_boundary(root, spec))
    violations.extend(_scan_docking_arbitration_controls_boundary(root, spec))
    violations.extend(_scan_workspace_shell_driver_owner_boundary(root, spec))
    violations.extend(_scan_window_hit_test_probe_boundary(root, spec))
    violations.extend(_scan_components_gallery_owner_boundary(root, spec))
    violations.extend(_scan_virtual_list_stress_controls_boundary(root, spec))
    violations.extend(_scan_table_stress_controls_boundary(root, spec))
    violations.extend(_scan_canvas_datagrid_stress_controls_boundary(root, spec))
    violations.extend(_scan_datatable_output_boundary(root, spec))
    violations.extend(_scan_editor_notes_bindings_boundary(root, spec))
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
