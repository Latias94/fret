//! Reference/smoke demo: tiny IMUI hello surface.
//!
//! This file stays useful as the smallest runnable facade smoke, but it is no longer the best
//! first-contact teaching surface for the immediate-mode lane.
//! Prefer `apps/fret-cookbook/examples/imui_action_basics.rs` for the generic/default immediate
//! path, then `apps/fret-examples/src/imui_editor_proof_demo.rs` for the editor-grade path.

use fret::{FretApp, advanced::prelude::*};

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
        fret_imui::imui_in(cx, |ui| {
            use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
            use fret_ui_kit::imui::UiWriterUiKitExt as _;

            let count_line = fret_ui_kit::ui::text(format!("Count: {count}"))
                .text_sm()
                .font_medium();
            ui.add_ui(count_line);
            if ui.button("Increment").clicked() {
                let _ = count_state.update_in(ui.cx_mut().app.models_mut(), |value| *value += 1);
            }

            ui.separator();

            let enabled_line = fret_ui_kit::ui::text(format!("Enabled: {enabled}"))
                .text_sm()
                .font_medium();
            ui.add_ui(enabled_line);
            let changed = ui
                .checkbox_model("Enabled", enabled_state.model())
                .changed();
            if changed {
                let enabled = enabled_state.paint_value_in(ui.cx_mut());
                let toggled_line = fret_ui_kit::ui::text(format!("Toggled to: {enabled}"));
                ui.add_ui(toggled_line);
            }
        })
    }
}
