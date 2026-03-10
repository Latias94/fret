use fret::{FretApp, advanced::prelude::*};

struct ImUiHelloView;

pub fn run() -> anyhow::Result<()> {
    FretApp::new("imui-hello-demo")
        .window("imui_hello_demo", (520.0, 240.0))
        .run_view::<ImUiHelloView>()?;
    Ok(())
}

impl View for ImUiHelloView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let count_state = cx.use_local_with(|| 0u32);
        let enabled_state = cx.use_local_with(|| false);

        let count = count_state.layout(cx).value_or_default();
        let enabled = enabled_state.paint(cx).value_or_default();

        fret_imui::imui_vstack(cx.elements(), |ui| {
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
                let enabled = enabled_state
                    .value_in(ui.cx_mut().app.models())
                    .unwrap_or_default();
                let toggled_line = fret_ui_kit::ui::text(format!("Toggled to: {enabled}"));
                ui.add_ui(toggled_line);
            }
        })
    }
}
