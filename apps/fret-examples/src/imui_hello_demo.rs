use fret_kit::prelude::*;

struct ImUiHelloState {
    count: Model<u32>,
    enabled: Model<bool>,
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app("imui-hello-demo", init_window, view)?
        .with_main_window("imui_hello_demo", (520.0, 240.0))
        .run()?;
    Ok(())
}

fn init_window(app: &mut App, _window: AppWindowId) -> ImUiHelloState {
    ImUiHelloState {
        count: app.models_mut().insert(0),
        enabled: app.models_mut().insert(false),
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut ImUiHelloState) -> fret_kit::ViewElements {
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
        ui.text(format!("Count: {count}"));
        if ui.button("Increment").clicked() {
            let _ = ui.cx_mut().app.models_mut().update(&st.count, |v| *v += 1);
        }

        ui.separator();

        ui.text(format!("Enabled: {enabled}"));
        let changed = ui.checkbox_model("Enabled", &st.enabled).changed();
        if changed {
            let enabled = ui
                .cx_mut()
                .app
                .models()
                .get_copied(&st.enabled)
                .unwrap_or_default();
            ui.text(format!("Toggled to: {enabled}"));
        }
    })
}
