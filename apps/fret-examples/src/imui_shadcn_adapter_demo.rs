use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*, component::prelude::*};
use fret_ui_shadcn::facade as shadcn;

struct ImUiShadcnAdapterView {
    count: Model<u32>,
    enabled: Model<bool>,
    value: Model<f32>,
    mode: Model<Option<Arc<str>>>,
    draft: Model<String>,
}

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

        Self {
            count: app.models_mut().insert(0),
            enabled: app.models_mut().insert(false),
            value: app.models_mut().insert(32.0),
            mode: app.models_mut().insert(None::<Arc<str>>),
            draft: app.models_mut().insert(String::new()),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let count = cx.watch_model(&self.count).layout().value_or_default();
        let enabled = cx.watch_model(&self.enabled).paint().value_or_default();
        let value = cx.watch_model(&self.value).paint().value_or_default();
        let mode = cx.watch_model(&self.mode).paint().value_or_default();
        let draft = cx.watch_model(&self.draft).paint().value_or_default();

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
                let _ = ui
                    .cx_mut()
                    .app
                    .models_mut()
                    .update(&self.count, |v| *v += 1);
            }

            let _ = ui.toggle_model("Enabled (toggle wrapper)", &self.enabled);

            let _ = ui.slider_f32_model_ex(
                "Value",
                &self.value,
                SliderOptions {
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                    ..Default::default()
                },
            );

            let _ = ui.select_model_ex(
                "Mode",
                &self.mode,
                &select_items,
                fret_ui_kit::imui::SelectOptions {
                    test_id: Some(Arc::from("imui-shadcn-demo.mode")),
                    ..Default::default()
                },
            );

            let _ = ui.input_text_model_ex(
                &self.draft,
                InputTextOptions {
                    placeholder: Some(Arc::from("Type some text...")),
                    ..Default::default()
                },
            );
        })
    }
}
