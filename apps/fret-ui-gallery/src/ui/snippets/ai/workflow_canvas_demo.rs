pub const SOURCE: &str = include_str!("workflow_canvas_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    use fret_canvas::view::PanZoom2D;
    use fret_runtime::Model;

    #[derive(Default)]
    struct DemoModels {
        view: Option<Model<PanZoom2D>>,
    }

    let needs_init = cx.with_state(DemoModels::default, |st| st.view.is_none());
    if needs_init {
        let view = cx.app.models_mut().insert(PanZoom2D::default());
        cx.with_state(DemoModels::default, |st| st.view = Some(view.clone()));
    }

    let view = cx
        .with_state(DemoModels::default, |st| st.view.clone())
        .expect("view");

    let canvas = ui_ai::WorkflowCanvas::new([
        ui_ai::WorkflowControls::new([
            ui_ai::WorkflowControlsButton::new("Zoom in", fret_icons::ids::ui::PLUS)
                .into_element(cx),
            ui_ai::WorkflowControlsButton::new("Zoom out", fret_icons::ids::ui::MINUS)
                .into_element(cx),
        ])
        .test_id("ui-ai-workflow-canvas-demo-controls")
        .refine_layout(
            LayoutRefinement::default()
                .absolute()
                .top(Space::N4)
                .left(Space::N4),
        )
        .into_element(cx),
        ui_ai::WorkflowToolbar::new([
            fret_ui_shadcn::Button::new("Action A")
                .test_id("ui-ai-workflow-canvas-demo-toolbar-a")
                .into_element(cx),
            fret_ui_shadcn::Button::new("Action B")
                .test_id("ui-ai-workflow-canvas-demo-toolbar-b")
                .into_element(cx),
        ])
        .test_id("ui-ai-workflow-canvas-demo-toolbar")
        .refine_layout(
            LayoutRefinement::default()
                .absolute()
                .bottom(Space::N4)
                .left(Space::N4),
        )
        .into_element(cx),
    ])
    .view(view)
    .test_id("ui-ai-workflow-canvas-demo-canvas")
    .into_element(cx);

    let props = cx.with_theme(|theme| {
        let chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(theme.color_token("border")));
        decl_style::container_props(
            theme,
            chrome,
            LayoutRefinement::default()
                .w_full()
                .h_px(Px(520.0))
                .min_w_0()
                .min_h_0(),
        )
    });

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3),
        move |cx| {
            vec![
                cx.text("WorkflowCanvas (AI Elements): pan/zoom host + overlay slot."),
                cx.container(props, move |_cx| vec![canvas]),
            ]
        },
    )
}
// endregion: example
