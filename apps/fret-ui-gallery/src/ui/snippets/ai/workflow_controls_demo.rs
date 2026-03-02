pub const SOURCE: &str = include_str!("workflow_controls_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui::action::OnActivate;

    #[derive(Default)]
    struct DemoModels {
        clicks: Option<Model<u32>>,
    }

    let needs_init = cx.with_state(DemoModels::default, |st| st.clicks.is_none());
    if needs_init {
        let clicks = cx.app.models_mut().insert(0u32);
        cx.with_state(DemoModels::default, |st| st.clicks = Some(clicks.clone()));
    }

    let clicks_model = cx
        .with_state(DemoModels::default, |st| st.clicks.clone())
        .expect("clicks");
    let clicks = cx
        .get_model_copied(&clicks_model, Invalidation::Paint)
        .unwrap_or(0);

    let bump: OnActivate = Arc::new({
        let clicks_model = clicks_model.clone();
        move |host, acx, _reason| {
            let _ = host
                .models_mut()
                .update(&clicks_model, |v| *v = v.saturating_add(1));
            host.request_redraw(acx.window);
        }
    });

    let marker = cx
        .text(format!("clicks={clicks}"))
        .test_id("ui-ai-workflow-controls-demo-clicks");

    let controls = ui_ai::WorkflowControls::new([
        ui_ai::WorkflowControlsButton::new("Zoom in", fret_icons::ids::ui::PLUS)
            .test_id("ui-ai-workflow-controls-demo-zoom-in")
            .on_activate(bump.clone())
            .into_element(cx),
        ui_ai::WorkflowControlsButton::new("Zoom out", fret_icons::ids::ui::MINUS)
            .test_id("ui-ai-workflow-controls-demo-zoom-out")
            .on_activate(bump.clone())
            .into_element(cx),
        ui_ai::WorkflowControlsButton::new("Fit", fret_icons::ids::ui::EYE)
            .test_id("ui-ai-workflow-controls-demo-fit")
            .on_activate(bump.clone())
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

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3),
        move |cx| {
            vec![
                cx.text("WorkflowControls (AI Elements): button stack chrome."),
                marker,
                cx.container(props, move |_cx| vec![controls]),
            ]
        },
    )
}
// endregion: example
