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


@dataclass(frozen=True)
class SourceCheck:
    path: Path
    required: list[str]
    forbidden: list[str]


def normalize(text: str) -> str:
    return "".join(text.split())


def normalized_source(path: Path) -> str:
    full_path = WORKSPACE_ROOT / path
    try:
        return normalize(full_path.read_text(encoding="utf-8"))
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.as_posix()}: {exc}")


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


def main() -> None:
    checks = [
        SourceCheck(
            Path("apps/fret-examples/src/imui_hello_demo.rs"),
            required=[
                "Reference/smoke demo: tiny IMUI hello surface.",
                "no longer the best",
                "first-contact teaching surface for the immediate-mode lane.",
                "Prefer `apps/fret-cookbook/examples/imui_action_basics.rs`",
                "`apps/fret-examples/src/imui_editor_proof_demo.rs`",
                "use fret::{FretApp, advanced::prelude::*, imui::prelude::*};",
                "imui_in(cx, |ui| {",
                'ui.text(format!("Count: {count}"));',
                'ui.checkbox_model("Enabled", enabled_state.model())',
                'ui.button("Increment").clicked()',
            ],
            forbidden=[
                "fret_imui::imui_in(cx, |ui| {",
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
                'fret_ui_kit::ui::text(format!("Count: {count}"))',
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/imui_floating_windows_demo.rs"),
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
            Path("apps/fret-examples/src/imui_response_signals_demo.rs"),
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
                "menu_lifecycle.activated()",
                "combo_resp.trigger.activated()",
                "combo_model_resp.deactivated_after_edit()",
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
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/imui_interaction_showcase_demo.rs"),
            required=[
                "Showcase surface for immediate-mode interaction affordances.",
                "Current proof/contract surface stays in `imui_response_signals_demo`.",
                "use fret::{FretApp, advanced::prelude::*, imui::prelude::*};",
                "use fret_ui_shadcn::facade as shadcn;",
                "imui(cx, move |ui| {",
                "const TEST_ID_INSPECTOR",
                "TEST_ID_INSPECTOR_SUMMARY",
                "imui-interaction-showcase.inspector.flag.",
                "ShowcaseInspectorState::default",
                "render_response_inspector_card(",
                "record_showcase_response(",
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
            ],
            forbidden=[
                "fret_imui::imui(cx, move |ui| {",
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
            ],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/imui_shadcn_adapter_demo.rs"),
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
                "TableSortDirection::Ascending",
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
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "NodeGraphSurfaceCompatRetainedProps::new(",
                "node_graph_surface_compat_retained(",
                "Retained-bridge IMUI demo for `fret-node`.",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("apps/fret-examples/src/imui_editor_proof_demo.rs"),
            required=[
                "use fret::imui::prelude::*;",
                "use fret_ui_editor::imui as editor_imui;",
                "use fret_ui_kit::imui::ImUiMultiSelectState;",
                "imui(cx, |ui| {",
                "imui(cx, move |ui| {",
                "imui_build(cx, out, |ui| {",
                "imui_build(cx, &mut out, move |ui| {",
                "imui_build(cx, out, f);",
                "editor_imui::property_grid(",
                "editor_imui::numeric_input(",
                "editor_imui::gradient_editor(",
            ],
            forbidden=[
                "use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;",
                "use fret_ui_kit::imui::UiWriterUiKitExt as _;",
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
            ],
        ),
        SourceCheck(
            Path("docs/examples/README.md"),
            required=[
                "Immediate-mode sidecar (when you intentionally want the IMUI lane):",
                "First-party authoring policy: use the root `fret::imui` lane",
                "`use fret::imui::prelude::*;`",
                "`use fret::imui::{kit::..., prelude::*};`",
                "deliberate exception is `imui_node_graph_demo`",
                "compatibility-only retained-bridge",
                "imui_action_basics",
                "imui_editor_proof_demo",
                "imui_hello_demo",
                "imui_interaction_showcase_demo",
                "imui_response_signals_demo",
                "imui_shadcn_adapter_demo",
                "imui_floating_windows_demo",
                "imui_node_graph_demo",
                "Golden pair:",
                "Reference/smoke:",
                "Reference/contract proof:",
                "Reference/product-validation:",
                "Compatibility-only:",
                "Mounting rule for the immediate-mode lane:",
                "On the explicit `fret::imui` lane, `imui(...)` is now the safe default",
                "`use fret::imui::prelude::*;`.",
                "`imui_raw(...)` is the advanced seam",
                "`imui_action_basics` demonstrates the explicit layout-host + raw shape on the root `fret::imui`",
                "Stable identity rule for the immediate-mode lane:",
                "`ui.for_each_unkeyed(...)` is acceptable.",
                "`ui.for_each_keyed(...)` or `ui.id(key, ...)`.",
                "Rebuild rows each frame; do not treat element values as cloneable reusable UI.",
                "`imui_editor_proof_demo` is the heavier proof where explicit stable identity is",
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
        "apps/fret-examples/src/imui_hello_demo.rs",
        "apps/fret-examples/src/imui_floating_windows_demo.rs",
        "apps/fret-examples/src/imui_interaction_showcase_demo.rs",
        "apps/fret-examples/src/imui_response_signals_demo.rs",
        "apps/fret-examples/src/imui_shadcn_adapter_demo.rs",
    ]:
        checks.append(SourceCheck(Path(path), required=[], forbidden=retained_bridge_forbidden))

    failures: list[str] = []
    for check in checks:
        check_source(check, failures)

    if failures:
        fail(GATE_NAME, f"{len(failures)} source marker problem(s):\n  - " + "\n  - ".join(failures))

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
