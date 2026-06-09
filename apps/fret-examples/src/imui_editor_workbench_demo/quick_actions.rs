use std::sync::Arc;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret_app::{CommandId, Effect};
use fret_ui::element::AnyElement;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::{IntoUiElementInExt as _, Space};
use fret_ui_shadcn::facade as shadcn;

const TEST_ID_ACTION_STRIP: &str = "imui-editor-workbench.action-strip";
const TEST_ID_ACTION_BUTTONS: &str = "imui-editor-workbench.action-buttons";
const TEST_ID_ACTION_STATUS: &str = "imui-editor-workbench.action-status";
const TEST_ID_ACTION_COMMAND: &str = "imui-editor-workbench.action-command";
const TEST_ID_ACTION_COPY_SELECTED: &str = "imui-editor-workbench.action.copy-selected-command";
const TEST_ID_ACTION_COPY_BUNDLE: &str = "imui-editor-workbench.action.copy-command-bundle";
const TEST_ID_ACTION_COPY_STATUS: &str = "imui-editor-workbench.action-copy-status";
const TEST_ID_ACTION_WORKBENCH: &str = "imui-editor-workbench.action.open-workbench";
const TEST_ID_ACTION_PROOF: &str = "imui-editor-workbench.action.supporting-proof";
const TEST_ID_ACTION_METRICS: &str = "imui-editor-workbench.action.metrics";
const TEST_ID_ACTION_DEBUG: &str = "imui-editor-workbench.action.debug";
const TEST_ID_ACTION_WAYLAND: &str = "imui-editor-workbench.action.wayland";

const WORKBENCH_COMMAND: &str = fret_first_open::demo_metrics_debug::DEMO_EDITOR_WORKBENCH_COMMAND;
const SUPPORTING_PROOF_COMMAND: &str =
    fret_first_open::demo_metrics_debug::DEMO_EDITOR_PROOF_COMMAND;
const METRICS_COMMAND: &str = fret_first_open::demo_metrics_debug::METRICS_STATS_COMMAND;
const DEBUG_COMMAND: &str = fret_first_open::demo_metrics_debug::DEBUG_TRACE_COMMAND;
const WAYLAND_ACCEPTANCE_COMMAND: &str = "FRET_DOCK_TEAROFF_LOG=1 cargo run -p fretboard-dev -- diag campaign run imui-p3-wayland-real-host --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release";

mod act {
    fret::actions!([
        SelectWorkbench = "imui_editor_workbench_demo.action.open_workbench.v1",
        SelectProof = "imui_editor_workbench_demo.action.supporting_proof.v1",
        SelectMetrics = "imui_editor_workbench_demo.action.metrics.v1",
        SelectDebug = "imui_editor_workbench_demo.action.debug.v1",
        SelectWayland = "imui_editor_workbench_demo.action.wayland.v1"
    ]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum WorkbenchQuickAction {
    #[default]
    Workbench,
    Proof,
    Metrics,
    Debug,
    Wayland,
}

#[derive(Debug, Clone, Copy)]
struct WorkbenchQuickActionSpec {
    action: WorkbenchQuickAction,
    label: &'static str,
    category: &'static str,
    command: &'static str,
    purpose: &'static str,
    test_id: &'static str,
    primary: bool,
}

const WORKBENCH_QUICK_ACTIONS: &[WorkbenchQuickActionSpec] = &[
    WorkbenchQuickActionSpec {
        action: WorkbenchQuickAction::Workbench,
        label: "Workbench",
        category: "demo",
        command: WORKBENCH_COMMAND,
        purpose: "Primary product-facing editor route.",
        test_id: TEST_ID_ACTION_WORKBENCH,
        primary: true,
    },
    WorkbenchQuickActionSpec {
        action: WorkbenchQuickAction::Proof,
        label: "Proof",
        category: "supporting demo",
        command: SUPPORTING_PROOF_COMMAND,
        purpose: "Dense editor-control and docking proof surface.",
        test_id: TEST_ID_ACTION_PROOF,
        primary: false,
    },
    WorkbenchQuickActionSpec {
        action: WorkbenchQuickAction::Metrics,
        label: "Metrics",
        category: "metrics",
        command: METRICS_COMMAND,
        purpose: "Read frame, layout, memory, and artifact stats from a diagnostics bundle.",
        test_id: TEST_ID_ACTION_METRICS,
        primary: false,
    },
    WorkbenchQuickActionSpec {
        action: WorkbenchQuickAction::Debug,
        label: "Debug",
        category: "debug",
        command: DEBUG_COMMAND,
        purpose: "Open a diagnostics trace drill-down for the selected bundle.",
        test_id: TEST_ID_ACTION_DEBUG,
        primary: false,
    },
    WorkbenchQuickActionSpec {
        action: WorkbenchQuickAction::Wayland,
        label: "Wayland",
        category: "handoff",
        command: WAYLAND_ACCEPTANCE_COMMAND,
        purpose: "Real-host compositor acceptance remains a runner/backend handoff.",
        test_id: TEST_ID_ACTION_WAYLAND,
        primary: false,
    },
];

pub(super) fn render_workbench_quick_action_strip(cx: &mut AppUi<'_, '_>) -> AnyElement {
    let active_action = cx.state().local_init(WorkbenchQuickAction::default);
    let copy_status_state = cx.state().local_init(|| {
        "Ready to copy the selected command or the full command bundle.".to_string()
    });
    cx.actions()
        .local(&active_action)
        .set::<act::SelectWorkbench>(WorkbenchQuickAction::Workbench);
    cx.actions()
        .local(&active_action)
        .set::<act::SelectProof>(WorkbenchQuickAction::Proof);
    cx.actions()
        .local(&active_action)
        .set::<act::SelectMetrics>(WorkbenchQuickAction::Metrics);
    cx.actions()
        .local(&active_action)
        .set::<act::SelectDebug>(WorkbenchQuickAction::Debug);
    cx.actions()
        .local(&active_action)
        .set::<act::SelectWayland>(WorkbenchQuickAction::Wayland);

    let active_action = cx.state().watch(&active_action).layout().value_or_default();
    let copy_status = copy_status_state.layout_value(cx);

    render_workbench_quick_action_strip_with_state(
        cx,
        active_action,
        copy_status_state,
        copy_status,
    )
}

fn workbench_text_section<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_section_chrome_label(cx, text)
}

fn workbench_text_paragraph<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_paragraph(cx, text)
}

fn workbench_text_readout<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_control_readout(cx, text)
}

fn workbench_quick_action_spec(action: WorkbenchQuickAction) -> &'static WorkbenchQuickActionSpec {
    WORKBENCH_QUICK_ACTIONS
        .iter()
        .find(|spec| spec.action == action)
        .unwrap_or(&WORKBENCH_QUICK_ACTIONS[0])
}

fn workbench_quick_action_command(action: WorkbenchQuickAction) -> CommandId {
    match action {
        WorkbenchQuickAction::Workbench => act::SelectWorkbench.into(),
        WorkbenchQuickAction::Proof => act::SelectProof.into(),
        WorkbenchQuickAction::Metrics => act::SelectMetrics.into(),
        WorkbenchQuickAction::Debug => act::SelectDebug.into(),
        WorkbenchQuickAction::Wayland => act::SelectWayland.into(),
    }
}

fn workbench_quick_action_command_bundle_text() -> String {
    WORKBENCH_QUICK_ACTIONS
        .iter()
        .map(|spec| format!("{}: {}", spec.label, spec.command))
        .collect::<Vec<_>>()
        .join("\n")
}

fn workbench_copy_text_on_activate(
    text: String,
    copy_status: LocalState<String>,
    next_status: String,
) -> fret_ui::action::OnActivate {
    Arc::new(move |host, action_cx, _reason| {
        let token = host.next_clipboard_token();
        host.push_effect(Effect::ClipboardWriteText {
            window: action_cx.window,
            token,
            text: text.clone(),
        });
        let _ = copy_status.set_in(host.models_mut(), next_status.clone());
        host.request_redraw(action_cx.window);
    })
}

fn render_workbench_quick_action_strip_with_state(
    cx: &mut AppUi<'_, '_>,
    active: WorkbenchQuickAction,
    copy_status_state: LocalState<String>,
    copy_status: String,
) -> AnyElement {
    let active_spec = *workbench_quick_action_spec(active);
    let mut action_buttons = WORKBENCH_QUICK_ACTIONS
        .iter()
        .map(|spec| {
            let selected = spec.action == active;
            let variant = if selected {
                shadcn::ButtonVariant::Default
            } else if spec.primary {
                shadcn::ButtonVariant::Secondary
            } else {
                shadcn::ButtonVariant::Outline
            };
            shadcn::Button::new(spec.label)
                .variant(variant)
                .size(shadcn::ButtonSize::Sm)
                .on_click(workbench_quick_action_command(spec.action))
                .test_id(spec.test_id)
                .ui()
                .into_element_in(cx)
        })
        .collect::<Vec<_>>();
    action_buttons.push(
        shadcn::Button::new("Copy command")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .on_activate(workbench_copy_text_on_activate(
                active_spec.command.to_string(),
                copy_status_state.clone(),
                format!("Copied {} command.", active_spec.label),
            ))
            .test_id(TEST_ID_ACTION_COPY_SELECTED)
            .ui()
            .into_element_in(cx),
    );
    action_buttons.push(
        shadcn::Button::new("Copy commands")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .on_activate(workbench_copy_text_on_activate(
                workbench_quick_action_command_bundle_text(),
                copy_status_state,
                "Copied Demo/Metrics/Debug command bundle.".to_string(),
            ))
            .test_id(TEST_ID_ACTION_COPY_BUNDLE)
            .ui()
            .into_element_in(cx),
    );
    let action_buttons = ui::h_flex(move |_cx| action_buttons)
        .gap(Space::N2)
        .items_center()
        .into_element_in(cx)
        .test_id(TEST_ID_ACTION_BUTTONS);

    let title = ui::v_flex(|cx| {
        ui::children![
            cx;
            workbench_text_section(cx, "Demo / Metrics / Debug"),
            workbench_text_paragraph(
                cx,
                "Canonical editor workbench actions stay visible next to the editor workflow; DevTools and fretboard own execution.",
            ),
        ]
    })
    .gap(Space::N1)
    .flex_1()
    .min_w_0()
    .into_element_in(cx);

    let status = ui::v_flex(move |cx| {
        ui::children![
            cx;
            workbench_text_readout(
                cx,
                format!(
                    "selected: {} | category: {} | primary: {}",
                    active_spec.label, active_spec.category, active_spec.primary
                ),
            ),
            workbench_text_readout(cx, active_spec.purpose),
            workbench_text_readout(cx, active_spec.command).test_id(TEST_ID_ACTION_COMMAND),
            workbench_text_readout(cx, copy_status).test_id(TEST_ID_ACTION_COPY_STATUS),
        ]
    })
    .gap(Space::N1)
    .flex_1()
    .min_w_0()
    .into_element_in(cx)
    .test_id(TEST_ID_ACTION_STATUS);

    ui::h_flex(|cx| ui::children![cx; title, action_buttons, status])
        .gap(Space::N4)
        .items_center()
        .justify_between()
        .w_full()
        .p(Space::N3)
        .rounded_md()
        .border_1()
        .into_element_in(cx)
        .test_id(TEST_ID_ACTION_STRIP)
}
