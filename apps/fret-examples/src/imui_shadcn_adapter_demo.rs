use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*};
use fret_ui_shadcn::facade as shadcn;

struct ImUiShadcnAdapterView;

pub fn run() -> anyhow::Result<()> {
    FretApp::new("imui-shadcn-adapter-demo")
        .window("imui_shadcn_adapter_demo", (840.0, 560.0))
        .view::<ImUiShadcnAdapterView>()?
        .run()?;
    Ok(())
}

impl View for ImUiShadcnAdapterView {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        shadcn::themes::apply_shadcn_new_york(
            app,
            shadcn::themes::ShadcnBaseColor::Slate,
            shadcn::themes::ShadcnColorScheme::Light,
        );

        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let count_state = cx.state().local_init(|| 0u32);
        let enabled_state = cx.state().local_init(|| false);
        let value_state = cx.state().local_init(|| 32.0f32);
        let mode_state = cx.state().local_init(|| None::<Arc<str>>);
        let draft_state = cx.state().local_init(String::new);

        let count = count_state.layout_value(cx);
        let enabled = enabled_state.paint_value(cx);
        let value = value_state.paint_value(cx);
        let mode = mode_state.paint_value(cx);
        let draft = draft_state.paint_value(cx);

        let mode_label: Arc<str> = mode.unwrap_or_else(|| Arc::from("none"));

        fret_imui::imui_vstack(cx.elements(), |ui| {
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
                let _ = count_state.update_in(ui.cx_mut().app.models_mut(), |v| *v += 1);
            }

            let _ = ui.switch_model("Enabled (switch)", enabled_state.model());

            let _ = ui.slider_f32_model_with_options(
                "Value",
                value_state.model(),
                SliderOptions {
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                    ..Default::default()
                },
            );

            let _ = ui.select_model_with_options(
                "Mode",
                mode_state.model(),
                &select_items,
                fret_ui_kit::imui::SelectOptions {
                    test_id: Some(Arc::from("imui-shadcn-demo.mode")),
                    ..Default::default()
                },
            );

            let _ = ui.input_text_model_with_options(
                draft_state.model(),
                InputTextOptions {
                    placeholder: Some(Arc::from("Type some text...")),
                    ..Default::default()
                },
            );
        })
    }
}
