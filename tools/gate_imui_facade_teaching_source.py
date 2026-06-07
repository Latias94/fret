from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "imui facade teaching source"

COMMON_DELETED_OR_NON_TEACHING = [
    "select_model_ex(",
    "window_ex(",
    "window_open_ex(",
    "floating_area_show_ex(",
    "begin_disabled(",
    "button_adapter(",
    "checkbox_model_adapter(",
    "fret_ui_kit::imui::adapters",
]

ACTIVE_FACADE_TEACHING_PATHS = [
    "apps/fret-cookbook/examples/imui_action_basics.rs",
    "apps/fret-cookbook/examples/imui_debug_draw_basics.rs",
    "apps/fret-cookbook/examples/imui_editor_controls_basics.rs",
    "apps/fret-cookbook/examples/imui_plot_basics.rs",
    "apps/fret-examples/src/imui_editor_workbench_demo.rs",
    "apps/fret-examples/src/imui_editor_proof_demo.rs",
    "apps/fret-examples/src/imui_editor_proof_demo/authoring_parity.rs",
    "apps/fret-examples/src/imui_editor_proof_demo/authoring_parity/models.rs",
    "apps/fret-examples/src/imui_editor_proof_demo/authoring_parity/shared_state.rs",
    "apps/fret-examples/src/imui_editor_proof_demo/collection.rs",
    "apps/fret-examples/src/imui_editor_proof_demo/workbench_shell.rs",
    "apps/fret-examples-imui/src/imui_hello_demo.rs",
    "apps/fret-examples-imui/src/imui_floating_windows_demo.rs",
    "apps/fret-examples-imui/src/imui_interaction_showcase_demo.rs",
    "apps/fret-examples-imui/src/imui_response_signals_demo.rs",
    "apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs",
    "apps/fret-cookbook/README.md",
    "apps/fret-cookbook/EXAMPLES.md",
    "docs/examples/README.md",
    "ecosystem/fret/README.md",
    "README.md",
]

ACTIVE_FACADE_DIRECT_CRATE_FORBIDDEN = [
    "fret_imui::",
    "use fret_imui",
    "fret_ui_kit::imui::",
    "use fret_ui_kit::imui",
]

COOKBOOK_EDITOR_DIRECT_CRATE_FORBIDDEN = [
    "use fret_ui_editor",
    "fret_ui_editor::",
]


@dataclass(frozen=True)
class SourceCheck:
    path: Path
    required: list[str]
    forbidden: list[str]


@dataclass(frozen=True)
class SourceSliceCheck:
    path: Path
    start_marker: str
    end_marker: str
    required: list[str]
    forbidden: list[str]


def normalize(text: str) -> str:
    return "".join(text.split())


def read_source(path: Path) -> str:
    full_path = WORKSPACE_ROOT / path
    try:
        return full_path.read_text(encoding="utf-8")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.as_posix()}: {exc}")


def normalized_source(path: Path) -> str:
    return normalize(read_source(path))


def check_source(check: SourceCheck, failures: list[str]) -> None:
    source = normalized_source(check.path)
    for marker in check.required:
        normalized = normalize(marker)
        if normalized not in source:
            failures.append(f"{check.path.as_posix()}: missing {marker}")
    for marker in [*COMMON_DELETED_OR_NON_TEACHING, *check.forbidden]:
        normalized = normalize(marker)
        if normalized in source:
            failures.append(f"{check.path.as_posix()}: forbidden {marker}")


def check_source_slice(check: SourceSliceCheck, failures: list[str]) -> None:
    source = read_source(check.path)
    try:
        start = source.index(check.start_marker)
    except ValueError:
        failures.append(f"{check.path.as_posix()}: missing slice start {check.start_marker}")
        return
    try:
        end = source.index(check.end_marker, start)
    except ValueError:
        failures.append(f"{check.path.as_posix()}: missing slice end {check.end_marker}")
        return

    normalized_slice = normalize(source[start:end])
    for marker in check.required:
        normalized = normalize(marker)
        if normalized not in normalized_slice:
            failures.append(f"{check.path.as_posix()}: missing slice marker {marker}")
    for marker in check.forbidden:
        normalized = normalize(marker)
        if normalized in normalized_slice:
            failures.append(f"{check.path.as_posix()}: forbidden slice marker {marker}")


def main() -> None:
    checks = [
        SourceCheck(
            Path("apps/fret-examples-imui/src/imui_hello_demo.rs"),
            required=[
                "Reference/smoke demo: tiny IMUI hello surface.",
                "no longer the best",
                "first-contact teaching surface for the immediate-mode lane.",
                "Prefer `apps/fret-cookbook/examples/imui_action_basics.rs`",
                "`apps/fret-examples/src/imui_editor_proof_demo.rs`",
                "use fret::{FretApp, advanced::prelude::*, imui::prelude::*};",
                "imui_in(cx, |ui| {",
                "let enabled = enabled_state.paint_value_in(ui.cx_mut());",
                'cx.text(format!("Count: {count}")).test_id(TEST_ID_COUNT_TEXT)',
                'ui.checkbox_model("Enabled", enabled_state.model())',
                'ui.button("Increment").clicked()',
            ],
            forbidden=[
                "fret_imui::imui_in(cx, |ui| {",
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
                "enabled_state.value_in(ui.cx_mut().app.models())",
                'fret_ui_kit::ui::text(format!("Count: {count}"))',
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples-imui/src/imui_floating_windows_demo.rs"),
            required=[
                "use fret::{FretApp, advanced::prelude::*, imui::prelude::*};",
                "imui_in(cx, |ui| {",
                "kit::WindowOptions::default()",
                "kit::FloatingWindowResizeOptions::default()",
                "ui.window_with_options(",
                "ui.combo_model_with_options(",
                "kit::MenuItemOptions {",
                "kit::ComboModelOptions {",
            ],
            forbidden=[
                "use fret_imui::prelude::UiWriter;",
                "fret_imui::imui_in(cx, |ui| {",
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
                "fret_ui_kit::imui::WindowOptions::default()",
                "fret_ui_kit::imui::FloatingWindowResizeOptions::default()",
                "fret_ui_kit::imui::MenuItemOptions",
                "fret_ui_kit::imui::ComboModelOptions",
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples-imui/src/imui_response_signals_demo.rs"),
            required=[
                "use fret::{FretApp, advanced::prelude::*, imui::prelude::*};",
                "imui_in(cx, |ui| {",
                "kit::SliderOptions {",
                "kit::InputTextOptions {",
                "kit::MenuItemOptions {",
                "kit::ComboOptions {",
                "kit::SelectableOptions {",
                "kit::ComboModelOptions {",
                "kit::MenuBarOptions {",
                "kit::BeginMenuOptions {",
                "kit::BeginSubmenuOptions {",
                "kit::TabBarOptions {",
                "kit::TabItemOptions {",
                "click.secondary_clicked()",
                "drag.drag_started()",
                "trigger.context_menu_requested()",
                "let menu_lifecycle = ui.menu_item_with_options(",
                "menu_lifecycle.activated()",
                "menu_lifecycle.deactivated()",
                "let combo_resp = ui.combo_with_options(",
                "combo_resp.response().activated()",
                "combo_resp.response().deactivated()",
                "let combo_model_resp = ui.combo_model_with_options(",
                "combo_model_resp.edited()",
                "combo_model_resp.deactivated_after_edit()",
                "let file_menu = ui.begin_menu_with_options(",
                "file_menu.opened()",
                "file_menu.closed()",
                "let recent_menu = ui.begin_submenu_with_options(",
                "recent_menu.toggled()",
                "let tab_response = ui.tab_bar_with_options(",
                "tab_response.selected_changed()",
                "if let Some(scene_tab) = tab_response.trigger(\"scene\") {",
                "scene_tab.clicked()",
                "scene_tab.activated()",
                "scene_tab.deactivated()",
                "let left_clicks = cx.state().local_init(|| 0u32);",
                "let drag_offset = cx.state().local_init(Point::default);",
                "let left_clicks_value = left_clicks.layout_value(cx);",
                "let drag_offset_value = drag_offset.layout_value(cx);",
                "let last_anchor_value = last_context_menu_anchor.layout_value(cx);",
            ],
            forbidden=[
                "fret_imui::imui_in(cx, |ui| {",
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
                "fret_ui_kit::imui::SliderOptions {",
                "fret_ui_kit::imui::InputTextOptions {",
                "fret_ui_kit::imui::MenuItemOptions {",
                "fret_ui_kit::imui::ComboOptions {",
                "fret_ui_kit::imui::SelectableOptions {",
                "fret_ui_kit::imui::ComboModelOptions {",
                "fret_ui_kit::imui::MenuBarOptions {",
                "fret_ui_kit::imui::BeginMenuOptions {",
                "fret_ui_kit::imui::BeginSubmenuOptions {",
                "fret_ui_kit::imui::TabBarOptions {",
                "fret_ui_kit::imui::TabItemOptions {",
                "cx.use_local_with(|| 0u32)",
                "cx.use_local_with(Point::default)",
                "left_clicks.layout(cx).value_or_default()",
                "drag_offset.layout(cx).value_or_default()",
                "last_context_menu_anchor.layout(cx).value_or_default()",
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples-imui/src/imui_interaction_showcase_demo.rs"),
            required=[
                "Showcase surface for immediate-mode interaction affordances.",
                "Current proof/contract surface stays in `imui_response_signals_demo`.",
                "This file intentionally uses the root `fret::imui` lane with shadcn shell chrome",
                "use fret::{FretApp, advanced::prelude::*, imui::prelude::*};",
                "use fret_ui_shadcn::facade as shadcn;",
                "imui(cx, move |ui| {",
                "const TEST_ID_INSPECTOR",
                "TEST_ID_INSPECTOR_SUMMARY",
                "imui-interaction-showcase.inspector.flag.",
                "ShowcaseInspectorState::default",
                "render_response_inspector_card(",
                "record_showcase_response(",
                "bookmark_slot.layout_value_in(ui.cx_mut())",
                "tool_mode.layout_value_in(ui.cx_mut())",
                "autosave_enabled.layout_value_in(ui.cx_mut())",
                "exposure_value.layout_value_in(ui.cx_mut())",
                "review_mode.layout_value_in(ui.cx_mut())",
                "selected_tab.layout_value_in(ui.cx_mut())",
                "context_toggle.layout_value_in(ui.cx_mut())",
                "fn push_showcase_event(",
                "let id = next_id.value_in_or_default(app.models());",
                "pulse.press_holding()",
                "drag.drag_stopped()",
                "quick_actions.context_menu_requested()",
                "kit::ButtonOptions {",
                "kit::ButtonArrowDirection::Left",
                "kit::RadioOptions {",
                "kit::SliderOptions {",
                "kit::ComboModelOptions {",
                "kit::InputTextOptions {",
                "kit::MenuBarOptions {",
                "kit::BeginMenuOptions {",
                "kit::BeginSubmenuOptions {",
                "kit::MenuItemOptions::default()",
                "kit::TabBarOptions {",
                "kit::TabItemOptions {",
                "kit::ChildRegionOptions {",
                "kit::ScrollOptions {",
                "ui.begin_menu_with_options(",
                "ui.tab_bar_with_options(",
                "ui.begin_popup_context_menu(",
                "let pulse_count = cx.state().local_init(|| 0u32);",
                "let autosave_enabled = cx.state().local_init(|| true);",
                "let selected_tab = cx.state().local_init(|| Some(Arc::<str>::from(\"overview\")));",
                "let pulse_count_value = pulse_count.layout_value(cx);",
                "let autosave_enabled_value = autosave_enabled.layout_value(cx);",
                "let selected_tab_value = selected_tab.layout_value(cx);",
                "let timeline_value = timeline.layout_value(cx);",
                "ui.switch_model(\"Autosave snapshots\", autosave_enabled.model())",
                "const SHOWCASE_COMPACT_RAIL_MIN_WIDTH: Px = Px(272.0);",
                "const SHOWCASE_COMPACT_RAIL_MAX_WIDTH: Px = Px(352.0);",
                "const SHOWCASE_REGULAR_SIDE_COLUMN_WIDTH: Px = Px(336.0);",
                ".basis(LengthRefinement::Fraction(0.32))",
                ".min_w(SHOWCASE_COMPACT_RAIL_MIN_WIDTH)",
                ".max_w(SHOWCASE_COMPACT_RAIL_MAX_WIDTH)",
                ".w_px(SHOWCASE_REGULAR_SIDE_COLUMN_WIDTH)",
            ],
            forbidden=[
                "fret_imui::imui(cx, move |ui| {",
                "This file intentionally mixes `fret_imui` control flow",
                "UiWriterImUiFacadeExt as _",
                "UiWriterUiKitExt as _",
                "fret_ui_kit::imui::ChildRegionOptions",
                "fret_ui_kit::imui::ScrollOptions",
                "fret_ui_kit::imui::ButtonOptions",
                "fret_ui_kit::imui::SliderOptions",
                "fret_ui_kit::imui::ComboModelOptions",
                "fret_ui_kit::imui::InputTextOptions",
                "fret_ui_kit::imui::MenuBarOptions",
                "fret_ui_kit::imui::BeginMenuOptions",
                "fret_ui_kit::imui::BeginSubmenuOptions",
                "fret_ui_kit::imui::MenuItemOptions",
                "fret_ui_kit::imui::TabBarOptions",
                "fret_ui_kit::imui::TabItemOptions",
                "bookmark_slot.value_in(ui.cx_mut().app.models())",
                "tool_mode.value_in(ui.cx_mut().app.models())",
                "autosave_enabled.value_in(ui.cx_mut().app.models())",
                "exposure_value.value_in(ui.cx_mut().app.models())",
                "review_mode.value_in(ui.cx_mut().app.models())",
                "selected_tab.value_in(ui.cx_mut().app.models())",
                "context_toggle.value_in(ui.cx_mut().app.models())",
                "cx.use_local_with(|| 0u32)",
                "cx.use_local_with(|| true)",
                "pulse_count.layout(cx).value_or_default()",
                "autosave_enabled.layout(cx).value_or_default()",
                "selected_tab.layout(cx).value_or_default()",
                "const SHOWCASE_SIDE_COLUMN_WIDTH: Px = Px(320.0);",
                "side_column_width: Px,",
                ".w_px(responsive.side_column_width)",
                "assert_eq!(layout.side_column_width, SHOWCASE_SIDE_COLUMN_WIDTH);",
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs"),
            required=[
                "Product-validation IMUI surface for the shared control-chrome lane.",
                "use fret::{FretApp, advanced::prelude::*, imui::prelude::*};",
                "imui_in(cx, |ui| {",
                "ui.add_ui(root);",
                "imui(cx, move |ui| {",
                'const TEST_ID_ROOT: &str = "imui-shadcn-demo.root";',
                'const TEST_ID_INCREMENT: &str = "imui-shadcn-demo.controls.increment";',
                'const TEST_ID_ENABLED: &str = "imui-shadcn-demo.controls.enabled";',
                'const TEST_ID_VALUE: &str = "imui-shadcn-demo.controls.value";',
                'const TEST_ID_MODE: &str = "imui-shadcn-demo.controls.mode";',
                'const TEST_ID_DRAFT: &str = "imui-shadcn-demo.controls.draft";',
                "summary_badge(",
                "kit::ButtonOptions {",
                "kit::SwitchOptions {",
                "kit::SliderOptions {",
                "kit::ComboModelOptions {",
                "kit::InputTextOptions {",
                "ui.combo_model_with_options(",
                "enum InspectorSort {",
                "fn direction(self) -> kit::TableSortDirection",
                "kit::TableSortDirection::Ascending",
                'kit::TableColumn::fill("Signal###inspector-signal")',
                ".sorted(inspector_sort.direction())",
                "kit::TableOptions {",
                "let table_response = ui.table_with_options(",
                "table_response",
                ".header(sort_column_id)",
                "kit::VirtualListOptions {",
                "kit::VirtualListMeasureMode::Fixed",
                "ui.virtual_list_with_options(",
            ],
            forbidden=[
                "fret_imui::imui_in(cx, |ui| {",
                "fret_imui::imui(cx, move |ui| {",
                "UiWriterImUiFacadeExt as _",
                "UiWriterUiKitExt as _",
                "fret_ui_kit::imui::TableSortDirection",
                "fret_ui_kit::imui::ButtonOptions",
                "fret_ui_kit::imui::SwitchOptions",
                "fret_ui_kit::imui::SliderOptions",
                "fret_ui_kit::imui::ComboModelOptions",
                "fret_ui_kit::imui::InputTextOptions",
                "fret_ui_kit::imui::TableColumn",
                "fret_ui_kit::imui::TableOptions",
                "fret_ui_kit::imui::VirtualListOptions",
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/imui_node_graph_demo.rs"),
            required=[
                "fret_imui::imui_in(cx, |ui| {",
                "compatibility-oriented and should not be treated as the default downstream",
                "Prefer the declarative node-graph surfaces for normal downstream guidance.",
                "use fret_imui::prelude::UiWriter as _;",
                "use fret_ui_kit::declarative::text as decl_text;",
                "fn compat_section_text<",
                "decl_text::text_section_chrome_label(cx, text)",
                'compat_section_text(cx, "imui node-graph compatibility proof")',
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "NodeGraphSurfaceCompatRetainedProps::new(",
                "node_graph_surface_compat_retained(",
                "Retained-bridge IMUI demo for `fret-node`.",
            ],
            forbidden=[
                'fret_ui_kit::ui::text("imui node-graph compatibility proof")',
                ".font_semibold()",
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/imui_editor_proof_demo.rs"),
            required=[
                "use fret::imui::{kit::ImUiMultiSelectState, prelude::*};",
                "use fret_ui_editor::imui as editor_imui;",
                "imui(cx, |ui| {",
                "imui_build(cx, out, |ui| {",
                "imui_build(cx, &mut out, move |ui| {",
                "imui_build(cx, out, f);",
                "mod proof_helpers;",
                "use proof_helpers::*;",
                "proof_imui_section_text(",
                "proof_imui_readout_text(",
                "proof_imui_compact_paragraph_text(",
                "kit::TooltipOptions {",
                "kit::CollapsingHeaderOptions {",
                "F: for<'cx, 'a> FnOnce(&mut ImUi<'cx, 'a, H>) + 'static,",
                "editor_imui::property_grid(",
                "editor_imui::numeric_input(",
                "editor_imui::gradient_editor(",
                'const ENV_EDITOR_PRESET: &str = "FRET_IMUI_EDITOR_PRESET";',
                "editor_theme_preset_from_env(ENV_EDITOR_PRESET)",
                "EditorThemePresetV1::ImguiLikeDense",
                ".defaults(Defaults {",
                "shadcn: false,",
                "install_imui_editor_proof_theme(app);",
                "shadcn::themes::apply_shadcn_new_york(",
            ],
            forbidden=[
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
                "use fret_ui_kit::imui",
                "fret_ui_kit::imui::",
                "fret_imui::ImUi",
                "fret_imui::imui(cx, |ui| {",
                "fret_imui::imui(cx, move |ui| {",
                "fret_imui::imui_build(cx, out, |ui| {",
                "fret_imui::imui_build(cx, &mut out, move |ui| {",
                "fret_imui::imui_build(cx, out, f);",
                "retained_bridge::",
                "RetainedSubtreeProps",
                "UiTreeRetainedExt as _",
                "retained_subtree_with(",
                "fret_node::imui::",
                "shadcn::app::install_with_theme(",
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/imui_editor_proof_demo.rs"),
            required=[
                "fn render_editor_name_assist_surface(",
                "fn render_authoring_parity_surface(",
                "authoring_parity::render_shared_state(",
                "fn render_authoring_parity_declarative_group(",
                "fn render_authoring_parity_imui_group(",
                "fn render_authoring_parity_imui_host<H, F>(",
                ") -> impl IntoUiElement<KernelApp> + use<> {",
                ") -> impl IntoUiElement<H> + use<H, F>",
                "Sortable math stays app-owned. `imui` only provides typed payloads + drop positions.",
                "let outliner_items = proof_outliner_items_snapshot(ui.cx_mut().app, &outliner_items_model);",
                "let outliner_order = proof_outliner_order_line_for_model(ui.cx_mut().app, &outliner_items_model);",
            ],
            forbidden=[
                "ui.cx_mut().app.models().read(&outliner_items_model, |items| items.clone())",
                "ui.cx_mut().app.models().read(&outliner_items_model, |items| { proof_outliner_order_line(items) })",
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/imui_editor_proof_demo/authoring_parity.rs"),
            required=[
                "mod models;",
                "mod shared_state;",
                "pub(super) use models::{",
                "pub(super) use shared_state::render_shared_state;",
            ],
            forbidden=COMMON_DELETED_OR_NON_TEACHING,
        ),
        SourceCheck(
            Path("apps/fret-examples/src/imui_editor_proof_demo/authoring_parity/shared_state.rs"),
            required=[
                "pub(in super::super) fn render_shared_state(",
                ") -> impl IntoUiElement<super::super::KernelApp> + use<> {",
                "proof_compact_readout_element(",
            ],
            forbidden=COMMON_DELETED_OR_NON_TEACHING,
        ),
        SourceCheck(
            Path("apps/fret-examples/src/imui_editor_proof_demo/workbench_shell.rs"),
            required=[
                "use fret::imui::prelude::*;",
                "imui(cx, move |ui| {",
                "ui.id(&panel_key, |ui| {",
                "fn embedded_target_for_window(app: &KernelApp, window: AppWindowId) -> fret_core::RenderTargetId {",
                "fn ensure_dock_graph_inner(app: &mut KernelApp, window: AppWindowId, force: bool) {",
                "let target = embedded_target_for_window(app, window);",
            ],
            forbidden=[
                "fret_imui::",
                "fret_ui_kit::imui::",
                "retained_bridge::",
                "RetainedSubtreeProps",
                "UiTreeRetainedExt as _",
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/imui_editor_proof_demo/proof_helpers.rs"),
            required=[
                "pub(super) fn proof_compact_readout<H: UiHost>(",
                "pub(super) fn proof_compact_readout_element<H: UiHost>(",
                "pub(super) fn proof_section_chrome_label<H: UiHost>(",
                "pub(super) fn proof_imui_section_text(",
                "pub(super) fn proof_imui_readout_text(",
                "pub(super) fn proof_imui_compact_paragraph_text(",
                "decl_text::text_control_readout(cx, readout.clone())",
                "decl_text::text_control_readout(cx, readout.clone()).test_id(test_id.into())",
                "decl_text::text_section_chrome_label(cx, text)",
                "decl_text::text_compact_paragraph(cx, text)",
                "pub(super) fn proof_outliner_items_snapshot(",
                "read(model, |items| items.clone())",
                "pub(super) fn proof_outliner_order_line_for_model(",
                "read(model, |items| proof_outliner_order_line(items))",
            ],
            forbidden=[
                "fret_imui::",
                "fret_ui_kit::imui::",
                "retained_bridge::",
                "RetainedSubtreeProps",
                "UiTreeRetainedExt as _",
            ],
        ),
        SourceCheck(
            Path(
                "docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/"
                "IMUI_EDITOR_PROOF_APP_OWNER_AUDIT_2026-04-16.md"
            ),
            required=[
                "outliner reorder math and dock bootstrap still belong to explicit app-owned helpers",
                "`proof_outliner_items_snapshot(...)`",
                "`proof_outliner_order_line_for_model(...)`",
                "`embedded_target_for_window(...)`",
                "do not justify new framework surface",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/workspace_shell_demo.rs"),
            required=[
                "use fret::{imui::prelude::*, shadcn, shadcn::themes::ShadcnColorScheme};",
                "imui_build(cx, out, move |ui| {",
                "kit::ChildRegionOptions {",
                "kit::ScrollOptions {",
                "kit::HorizontalOptions {",
            ],
            forbidden=[
                "fret_ui_kit::imui::",
                "use fret_ui_kit::imui",
                "fret_imui::imui_build(cx, out, move |ui| {",
                "UiWriterImUiFacadeExt as _",
            ],
        ),
        SourceCheck(
            Path("docs/examples/README.md"),
            required=[
                "Immediate-mode sidecar (when you intentionally want the IMUI lane):",
                "First-party authoring policy: use the root `fret::imui` lane",
                "`use fret::imui::prelude::*;`",
                "`use fret::imui::{kit::..., prelude::*};`",
                "No first-party IMUI example uses the retained node-graph canvas.",
                "imui_action_basics",
                "imui_plot_basics",
                "imui_editor_proof_demo",
                "imui_hello_demo",
                "imui_interaction_showcase_demo",
                "imui_response_signals_demo",
                "imui_shadcn_adapter_demo",
                "imui_floating_windows_demo",
                "cargo run -p fretboard -- dev native --package fret-demo --bin imui_hello_demo",
                "cargo run -p fretboard -- dev native --package fret-examples-imui --bin imui_hello_demo",
                "both define `imui_hello_demo`",
                "Golden pair:",
                "Reference/smoke:",
                "Reference/contract proof:",
                "Reference/product-validation:",
                "Advanced/reference:",
                "Mounting rule for the immediate-mode lane:",
                "On the explicit `fret::imui` lane, `imui(...)` is now the safe default",
                "`use fret::imui::prelude::*;`.",
                "`imui_raw(...)` is the advanced seam",
                "`imui_action_basics` demonstrates the explicit layout-host + raw shape on the root `fret::imui`",
                "Stable identity rule for the immediate-mode lane:",
                "`ui.for_each_unkeyed(...)` is acceptable.",
                "`ui.for_each_keyed(...)` or `ui.id(key, ...)`.",
                "Rebuild rows each frame; do not treat element values as cloneable reusable UI.",
                "`imui_editor_workbench_demo` is the canonical product workbench route, while",
                "`imui_editor_proof_demo` remains the supporting dense panel proof where explicit stable identity",
                "it mounts the editor-notes workflow directly",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md"),
            required=[
                "Any future `fret-ui-kit::imui` public helper widening must name at least two real first-party proof",
                "For P0, the current minimum proof budget is the frozen immediate-mode golden pair:",
                "`apps/fret-cookbook/examples/imui_action_basics.rs`",
                "`apps/fret-examples/src/imui_editor_proof_demo.rs`",
                "Reference, advanced, or compatibility-only surfaces do not count by themselves.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-editor-grade-product-closure-v1/P0_ROOT_HOSTING_RULE_2026-04-12.md"),
            required=[
                "If your IMUI content already lives under an explicit layout host, prefer",
                "If you are mounting IMUI directly at the view root or under a non-layout parent, prefer",
                "`imui_raw(...)` is the advanced explicit-layout seam, not evidence that generic helper growth",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-editor-grade-product-closure-v1/P0_STABLE_IDENTITY_RULE_2026-04-12.md"),
            required=[
                "For static lists whose order never changes, `ui.for_each_unkeyed(...)` is acceptable.",
                "For dynamic collections that can insert, remove, reorder, or preserve per-row state, prefer",
                "Rebuild UI rows each frame; do not treat elements as cloneable reusable values.",
                "already uses explicit `ui.id(...)` where stable panel identity matters",
            ],
            forbidden=[],
        ),
    ]

    retained_bridge_forbidden = [
        "retained_bridge::",
        "RetainedSubtreeProps",
        "UiTreeRetainedExt as _",
        "retained_subtree_with(",
        "fret_node::imui::",
    ]
    for path in [
        "apps/fret-examples-imui/src/imui_hello_demo.rs",
        "apps/fret-examples-imui/src/imui_floating_windows_demo.rs",
        "apps/fret-examples-imui/src/imui_interaction_showcase_demo.rs",
        "apps/fret-examples-imui/src/imui_response_signals_demo.rs",
        "apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs",
    ]:
        checks.append(SourceCheck(Path(path), required=[], forbidden=retained_bridge_forbidden))

    for path in ACTIVE_FACADE_TEACHING_PATHS:
        checks.append(
            SourceCheck(
                Path(path),
                required=[],
                forbidden=ACTIVE_FACADE_DIRECT_CRATE_FORBIDDEN,
            )
        )

    checks.append(
        SourceCheck(
            Path("apps/fret-cookbook/examples/imui_editor_controls_basics.rs"),
            required=[],
            forbidden=COOKBOOK_EDITOR_DIRECT_CRATE_FORBIDDEN,
        )
    )

    slice_checks = [
        SourceSliceCheck(
            Path("apps/fret-examples/src/imui_editor_proof_demo.rs"),
            start_marker="fn render_authoring_parity_imui_group(",
            end_marker="fn build_authoring_parity_gradient_editor(",
            required=[
                "render_authoring_parity_imui_host(cx, move |ui| {",
                "editor_imui::property_group(",
                "editor_imui::property_grid(",
                "editor_imui::text_field(",
                "editor_imui::drag_value(",
                "editor_imui::numeric_input(",
                "editor_imui::slider(",
                "editor_imui::field_status_badge(",
                "editor_imui::checkbox(",
                "editor_imui::enum_select(",
                "let gradient_editor = build_authoring_parity_gradient_editor(",
                "editor_imui::gradient_editor(ui, gradient_editor);",
            ],
            forbidden=[
                "FieldStatusBadge::new(FieldStatus::Dirty).into_element(cx)",
                "GradientEditor::new(",
            ],
        ),
        SourceSliceCheck(
            Path("apps/fret-examples/src/imui_editor_proof_demo/workbench_shell.rs"),
            start_marker="fn ensure_dock_graph_inner(",
            end_marker="struct WindowBootstrapService {",
            required=["let target = embedded_target_for_window(app, window);"],
            forbidden=[
                "embedded::models(app, window).and_then(|m| app.models().read(&m.target, |v| *v).ok()).unwrap_or_default()"
            ],
        ),
    ]
    failures: list[str] = []
    for check in checks:
        check_source(check, failures)
    for check in slice_checks:
        check_source_slice(check, failures)

    if failures:
        fail(GATE_NAME, f"{len(failures)} source marker problem(s):\n  - " + "\n  - ".join(failures))

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
