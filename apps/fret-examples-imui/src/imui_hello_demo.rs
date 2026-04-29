//! Reference/smoke demo: tiny IMUI hello surface.
//!
//! This file stays useful as the smallest runnable facade smoke, but it is no longer the best
//! first-contact teaching surface for the immediate-mode lane.
//! Prefer `apps/fret-cookbook/examples/imui_action_basics.rs` for the generic/default immediate
//! path, then `apps/fret-examples/src/imui_editor_proof_demo.rs` for the editor-grade path.

use fret::{FretApp, advanced::prelude::*, imui::prelude::*};

struct ImUiHelloView;

pub fn run() -> anyhow::Result<()> {
    FretApp::new("imui-hello-demo")
        .window("imui_hello_demo", (520.0, 240.0))
        .view::<ImUiHelloView>()?
        .run()?;
    Ok(())
}

impl View for ImUiHelloView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let count_state = cx.state().local_init(|| 0u32);
        let enabled_state = cx.state().local_init(|| false);

        let count = count_state.layout_value(cx);
        let enabled = enabled_state.paint_value(cx);

        // This demo mounts IMUI directly at the view root, so the default `imui(...)` entrypoint
        // should own the stacked host for us.
        imui_in(cx, |ui| {
            ui.text(format!("Count: {count}"));
            if ui.button("Increment").clicked() {
                let _ = count_state.update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }

            ui.separator();

            ui.text(format!("Enabled: {enabled}"));
            let changed = ui
                .checkbox_model("Enabled", enabled_state.model())
                .changed();
            if changed {
                let enabled = enabled_state.paint_value_in(ui.cx_mut());
                ui.text(format!("Toggled to: {enabled}"));
            }
        })
    }
}
