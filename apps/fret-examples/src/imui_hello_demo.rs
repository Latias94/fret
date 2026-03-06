use fret::prelude::*;

struct ImUiHelloState {
    count: Model<u32>,
    enabled: Model<bool>,
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("imui-hello-demo")
        .window("imui_hello_demo", (520.0, 240.0))
        .ui(init_window, view)?
        .run()?;
    Ok(())
}

fn init_window(app: &mut App, _window: AppWindowId) -> ImUiHelloState {
    ImUiHelloState {
        count: app.models_mut().insert(0),
        enabled: app.models_mut().insert(false),
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut ImUiHelloState) -> fret::ViewElements {
    let count = cx
        .watch_model(&st.count)
        .layout()
        .copied()
        .unwrap_or_default();

    let enabled = cx
        .watch_model(&st.enabled)
        .paint()
        .copied()
        .unwrap_or_default();

    fret_imui::imui_vstack(cx, |ui| {
        use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
        use fret_ui_kit::imui::UiWriterUiKitExt as _;

        let count_line = fret_ui_kit::ui::text(format!("Count: {count}"))
            .text_sm()
            .font_medium();
        ui.add_ui(count_line);
        if ui.button("Increment").clicked() {
            let _ = ui.cx_mut().app.models_mut().update(&st.count, |v| *v += 1);
        }

        ui.separator();

        let enabled_line = fret_ui_kit::ui::text(format!("Enabled: {enabled}"))
            .text_sm()
            .font_medium();
        ui.add_ui(enabled_line);
        let changed = ui.checkbox_model("Enabled", &st.enabled).changed();
        if changed {
            let enabled = ui
                .cx_mut()
                .app
                .models()
                .get_copied(&st.enabled)
                .unwrap_or_default();
            let toggled_line = fret_ui_kit::ui::text(format!("Toggled to: {enabled}"));
            ui.add_ui(toggled_line);
        }
    })
}
