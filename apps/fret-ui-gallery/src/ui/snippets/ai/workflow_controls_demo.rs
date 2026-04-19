pub const SOURCE: &str = include_str!("workflow_controls_demo.rs");

// region: example
use fret::app::AppRenderActionsExt as _;
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    use fret_ui::Invalidation;
    let clicks_model = cx.local_model_keyed("clicks", || 0u32);
    let clicks = cx
        .get_model_copied(&clicks_model, Invalidation::Paint)
        .unwrap_or(0);

    let marker = cx
        .text(format!("clicks={clicks}"))
        .test_id("ui-ai-workflow-controls-demo-clicks");

    let controls = ui_ai::WorkflowControls::new([
        ui_ai::WorkflowControlsButton::new("Zoom in", fret_icons::ids::ui::PLUS)
            .test_id("ui-ai-workflow-controls-demo-zoom-in")
            .on_activate(cx.actions().listen({
                let clicks_model = clicks_model.clone();
                move |host, acx| {
                    let _ = host
                        .models_mut()
                        .update(&clicks_model, |v| *v = v.saturating_add(1));
                    host.request_redraw(acx.window);
                }
            }))
            .into_element(cx),
        ui_ai::WorkflowControlsButton::new("Zoom out", fret_icons::ids::ui::MINUS)
            .test_id("ui-ai-workflow-controls-demo-zoom-out")
            .on_activate(cx.actions().listen({
                let clicks_model = clicks_model.clone();
                move |host, acx| {
                    let _ = host
                        .models_mut()
                        .update(&clicks_model, |v| *v = v.saturating_add(1));
                    host.request_redraw(acx.window);
                }
            }))
            .into_element(cx),
        ui_ai::WorkflowControlsButton::new("Fit", fret_icons::ids::ui::EYE)
            .test_id("ui-ai-workflow-controls-demo-fit")
            .on_activate(cx.actions().listen({
                let clicks_model = clicks_model.clone();
                move |host, acx| {
                    let _ = host
                        .models_mut()
                        .update(&clicks_model, |v| *v = v.saturating_add(1));
                    host.request_redraw(acx.window);
                }
            }))
            .into_element(cx),
    ])
    .test_id("ui-ai-workflow-controls-demo-controls")
    .into_element(cx);

    let props = cx.with_theme(|theme| {
        let chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(theme.color_token("border")))
            .p(Space::N4);
        decl_style::container_props(
            theme,
            chrome,
            LayoutRefinement::default().w_full().min_w_0(),
        )
    });

    ui::v_flex(move |cx| {
        vec![
            cx.text("WorkflowControls (AI Elements): button stack chrome."),
            marker,
            cx.container(props, move |_cx| vec![controls]),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .into_element(cx)
}
// endregion: example
