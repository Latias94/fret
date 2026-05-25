//! Canonical IMUI editor workbench route.
//!
//! This route owns the stable product-facing editor workbench entrypoint. It now mounts the
//! editor-notes workflow as the first converged editor workflow while the older focused demos stay
//! supporting proof surfaces for shell behavior, dense editor controls, and docking arbitration.

use fret::{Defaults, FretApp};

/// Runs the canonical IMUI editor workbench route.
///
/// Current owner split:
/// - `editor_notes_demo` owns the reusable editor notes workflow view.
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
        .view::<crate::editor_notes_demo::EditorNotesDemoView>()?
        .run()
        .map_err(anyhow::Error::from)
}
