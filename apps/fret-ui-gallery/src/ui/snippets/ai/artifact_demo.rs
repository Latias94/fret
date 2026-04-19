pub const SOURCE: &str = include_str!("artifact_demo.rs");

// region: example
use fret::app::AppRenderActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let present = cx.local_model_keyed("present", || true);

    let is_present = cx.app.models().read(&present, |v| *v).unwrap_or(true);

    let present_for_reset = present.clone();
    let reset = shadcn::Button::new("Reset artifact")
        .variant(shadcn::ButtonVariant::Secondary)
        .on_activate(cx.actions().listen(move |host, action_cx| {
            let _ = host.models_mut().update(&present_for_reset, |v| *v = true);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        }))
        .test_id("ui-ai-artifact-demo-reset")
        .into_element(cx);

    let artifact = if is_present {
        let close = ui_ai::ArtifactClose::new()
            .test_id("ui-ai-artifact-close")
            .on_activate(cx.actions().listen({
                let present = present.clone();
                move |host, action_cx| {
                    let _ = host.models_mut().update(&present, |v| *v = false);
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                }
            }))
            .into_element(cx);

        let header_text = ui::v_flex(move |cx| {
            vec![
                ui_ai::ArtifactTitle::new("Generated UI Spec").into_element(cx),
                ui_ai::ArtifactDescription::new("A structured container with header actions.")
                    .into_element(cx),
            ]
        })
        .layout(LayoutRefinement::default().min_w_0().flex_1())
        .gap(Space::N1)
        .items_start()
        .into_element(cx);

        let export_action = ui_ai::ArtifactAction::new()
            .label("Export")
            .icon(fret_icons::IconId::new_static("lucide.download"))
            .into_element(cx);

        let header_actions = ui::h_flex(move |cx| {
            vec![ui_ai::ArtifactActions::new([export_action, close]).into_element(cx)]
        })
        .layout(LayoutRefinement::default().flex_shrink_0())
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        ui_ai::Artifact::new([
            ui_ai::ArtifactHeader::new([header_text, header_actions]).into_element(cx),
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
