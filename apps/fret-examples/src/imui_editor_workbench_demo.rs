//! Canonical IMUI editor workbench route.
//!
//! This route owns the stable product-facing editor workbench entrypoint. It mounts the
//! editor-notes workflow as the first converged editor workflow and keeps the Dear ImGui-style
//! Demo/Metrics/Debug route visible as persistent workbench chrome while the older focused demos
//! stay supporting proof surfaces for shell behavior, dense editor controls, and docking
//! arbitration.

use fret::app::prelude::*;
use fret::{Defaults, FretApp};
use fret_ui_kit::{IntoUiElementInExt as _, Space};

mod quick_actions;

const TEST_ID_ROOT: &str = "imui-editor-workbench.root";
const TEST_ID_ACTION_STRIP_REGION: &str = "imui-editor-workbench.action-strip-region";
const TEST_ID_WORKFLOW: &str = "imui-editor-workbench.workflow";

struct ImUiEditorWorkbenchView {
    notes: crate::editor_notes_demo::EditorNotesDemoView,
}

/// Runs the canonical IMUI editor workbench route.
///
/// Current owner split:
/// - `editor_notes_demo` owns the reusable editor notes workflow view.
/// - `quick_actions` owns the persistent Demo/Metrics/Debug workbench chrome.
/// - `workspace_shell_demo` remains supporting shell proof evidence.
/// - this module owns the stable product-facing route name.
pub fn run() -> anyhow::Result<()> {
    FretApp::new("imui-editor-workbench-demo")
        .window("imui_editor_workbench_demo", (1180.0, 760.0))
        .defaults(Defaults {
            shadcn: false,
            ..Defaults::desktop_app()
        })
        .setup((
            crate::editor_notes_demo::install_editor_notes_demo_theme,
            fret_icons_lucide::app::install,
        ))
        .view::<ImUiEditorWorkbenchView>()?
        .run()
        .map_err(anyhow::Error::from)
}

impl View for ImUiEditorWorkbenchView {
    fn init(app: &mut App, window: WindowId) -> Self {
        Self {
            notes: crate::editor_notes_demo::EditorNotesDemoView::init(app, window),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let action_strip = quick_actions::render_workbench_quick_action_strip(cx);
        let action_strip_region = ui::container(|_cx| [action_strip])
            .px(Space::N4)
            .pt(Space::N4)
            .w_full()
            .into_element_in(cx)
            .test_id(TEST_ID_ACTION_STRIP_REGION);

        let workflow_elements = self.notes.render(cx);
        let workflow = ui::container(move |_cx| workflow_elements)
            .flex_1()
            .min_h_0()
            .w_full()
            .into_element_in(cx)
            .test_id(TEST_ID_WORKFLOW);

        ui::v_flex(|_cx| [action_strip_region, workflow])
            .items_stretch()
            .size_full()
            .into_element_in(cx)
            .test_id(TEST_ID_ROOT)
            .into()
    }
}
