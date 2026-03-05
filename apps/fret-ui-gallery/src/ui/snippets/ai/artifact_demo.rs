pub const SOURCE: &str = include_str!("artifact_demo.rs");

// region: example
use fret_runtime::Model;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    present: Option<Model<bool>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let present = cx.with_state(DemoModels::default, |st| st.present.clone());
    let present = match present {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(DemoModels::default, |st| st.present = Some(model.clone()));
            model
        }
    };

    let is_present = cx.app.models().read(&present, |v| *v).unwrap_or(true);

    let present_for_reset = present.clone();
    let reset = shadcn::Button::new("Reset artifact")
        .variant(shadcn::ButtonVariant::Secondary)
        .on_activate(Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&present_for_reset, |v| *v = true);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        }))
        .test_id("ui-ai-artifact-demo-reset")
        .into_element(cx);

    let artifact = if is_present {
        let close = ui_ai::ArtifactClose::new()
            .test_id("ui-ai-artifact-close")
            .on_activate(Arc::new({
                let present = present.clone();
                move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&present, |v| *v = false);
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                }
            }))
            .into_element(cx);

        ui_ai::Artifact::new([
            ui_ai::ArtifactHeader::new([
                ui_ai::ArtifactTitle::new("Generated UI Spec").into_element(cx),
                ui_ai::ArtifactDescription::new("A structured container with header actions.")
                    .into_element(cx),
                ui_ai::ArtifactActions::new([
                    ui_ai::ArtifactAction::new()
                        .label("Export")
                        .icon(fret_icons::IconId::new_static("lucide.download"))
                        .into_element(cx),
                    close,
                ])
                .into_element(cx),
            ])
            .into_element(cx),
            ui_ai::ArtifactContent::new([
                cx.text("Artifacts are chrome-only: apps own rendering, export, and lifecycle.")
            ])
            .into_element(cx),
        ])
        .test_id_root("ui-ai-artifact-root")
        .into_element(cx)
    } else {
        cx.text("Artifact closed.")
    };

    ui::v_flex(move |cx| {
        vec![
            cx.text("Artifact (AI Elements)"),
            cx.text("Close hides the artifact; reset re-mounts it."),
            reset,
            artifact,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
