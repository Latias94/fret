pub const SOURCE: &str = include_str!("workflow_toolbar_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {

    let toolbar = ui_ai::WorkflowToolbar::new([
        fret_ui_shadcn::Button::new("Run")
            .test_id("ui-ai-workflow-toolbar-demo-run")
            .into_element(cx),
        fret_ui_shadcn::Button::new("Stop")
            .test_id("ui-ai-workflow-toolbar-demo-stop")
            .into_element(cx),
        fret_ui_shadcn::Button::new("Inspect")
            .test_id("ui-ai-workflow-toolbar-demo-inspect")
            .into_element(cx),
    ])
    .test_id("ui-ai-workflow-toolbar-demo-toolbar")
    .into_element(cx);

    let props = cx.with_theme(|theme| {
        let chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(theme.color_token("border")))
            .p(Space::N4);
        decl_style::container_props(theme, chrome, LayoutRefinement::default().w_full())
    });

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3),
        move |cx| {
            vec![
                cx.text("WorkflowToolbar (AI Elements): compact tool row chrome."),
                cx.container(props, move |_cx| vec![toolbar]),
            ]
        },
    )
}
// endregion: example
