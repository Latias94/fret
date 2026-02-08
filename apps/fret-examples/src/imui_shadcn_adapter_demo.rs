use std::sync::Arc;

use fret_kit::prelude::*;
use fret_ui_shadcn as shadcn;

struct ImUiShadcnAdapterState {
    count: Model<u32>,
    enabled: Model<bool>,
    value: Model<f32>,
    mode: Model<Option<Arc<str>>>,
    draft: Model<String>,
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app("imui-shadcn-adapter-demo", init_window, view)?
        .with_main_window("imui_shadcn_adapter_demo", (840.0, 560.0))
        .run()?;
    Ok(())
}

fn init_window(app: &mut App, _window: AppWindowId) -> ImUiShadcnAdapterState {
    shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        app,
        shadcn::shadcn_themes::ShadcnBaseColor::Slate,
        shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    ImUiShadcnAdapterState {
        count: app.models_mut().insert(0),
        enabled: app.models_mut().insert(false),
        value: app.models_mut().insert(32.0),
        mode: app.models_mut().insert(None::<Arc<str>>),
        draft: app.models_mut().insert(String::new()),
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut ImUiShadcnAdapterState) -> ViewElements {
    let count = cx.watch_model(&st.count).layout().copied_or_default();
    let enabled = cx.watch_model(&st.enabled).paint().copied_or_default();
    let value = cx.watch_model(&st.value).paint().copied_or_default();
    let mode = cx.watch_model(&st.mode).paint().cloned_or_default();
    let draft = cx.watch_model(&st.draft).paint().cloned_or_default();

    let mode_label: Arc<str> = mode.unwrap_or_else(|| Arc::from("none"));

    fret_imui::imui_vstack(cx, |ui| {
        use fret_ui_kit::imui::{InputTextOptions, SliderOptions, UiWriterImUiFacadeExt as _};

        let select_items = [
            Arc::<str>::from("Alpha"),
            Arc::<str>::from("Beta"),
            Arc::<str>::from("Gamma"),
        ];

        let summary_card = {
            let cx = ui.cx_mut();
            let header = shadcn::CardHeader::new([
                shadcn::CardTitle::new("imui + shadcn adapter (minimal)").into_element(cx),
                shadcn::CardDescription::new(
                    "Control flow stays immediate-mode; visuals can come from shadcn recipes.",
                )
                .into_element(cx),
            ])
            .into_element(cx);

            let content = shadcn::CardContent::new([
                shadcn::Badge::new(format!("count: {count}"))
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
                shadcn::Badge::new(format!("enabled: {enabled}"))
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
                shadcn::Badge::new(format!("value: {:.1}", value))
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
                shadcn::Badge::new(format!("mode: {mode_label}"))
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
                shadcn::Badge::new(format!("draft: {draft}"))
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
            ])
            .into_element(cx);

            shadcn::Card::new([header, content]).into_element(cx)
        };
        ui.add(summary_card);

        if ui.button("Increment count (imui button)").clicked() {
            let _ = ui.cx_mut().app.models_mut().update(&st.count, |v| *v += 1);
        }

        let _ = ui.toggle_model("Enabled (toggle wrapper)", &st.enabled);

        let _ = ui.slider_f32_model_ex(
            "Value",
            &st.value,
            SliderOptions {
                min: 0.0,
                max: 100.0,
                step: 1.0,
                ..Default::default()
            },
        );

        let _ = ui.select_model_ex(
            "Mode",
            &st.mode,
            &select_items,
            fret_ui_kit::imui::SelectOptions {
                test_id: Some(Arc::from("imui-shadcn-demo.mode")),
                ..Default::default()
            },
        );

        let _ = ui.input_text_model_ex(
            &st.draft,
            InputTextOptions {
                placeholder: Some(Arc::from("Type some text...")),
                ..Default::default()
            },
        );
    })
}
