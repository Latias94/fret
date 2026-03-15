pub const SOURCE: &str = include_str!("workflow_connection_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    use fret_core::Point;
    use fret_core::Px;

    let conn = ui_ai::WorkflowConnection::new(
        Point::new(Px(48.0), Px(56.0)),
        Point::new(Px(312.0), Px(176.0)),
    )
    .test_id("ui-ai-workflow-connection-demo-conn")
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
                .h_px(Px(240.0))
                .min_w_0()
                .min_h_0(),
        )
    });

    ui::v_flex(move |cx| {
        vec![
            cx.text("WorkflowConnection (AI Elements): in-progress connection line chrome."),
            cx.container(props, move |_cx| vec![conn]),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .into_element(cx)
}
// endregion: example
