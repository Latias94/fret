pub const SOURCE: &str = include_str!("theming.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn token_chip(cx: &mut AppComponentCx<'_>, token: &'static str) -> AnyElement {
    let props = decl_style::container_props(
        &cx.theme().snapshot(),
        ChromeRefinement::default()
            .rounded(Radius::Md)
            .border_1()
            .border_color(ColorRef::Color(cx.theme().color_token("border")))
            .bg(ColorRef::Color(cx.theme().color_token(token)))
            .px(Space::N3)
            .py(Space::N2),
        LayoutRefinement::default().min_w(Px(120.0)),
    );

    cx.container(props, move |cx| {
        vec![
            ui::text(token)
                .text_sm()
                .font_medium()
                .text_color(ColorRef::Color(cx.theme().color_token("background")))
                .into_element(cx),
        ]
    })
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
        vec![
            ui::h_flex(|cx| {
                vec![
                    token_chip(cx, "chart-1"),
                    token_chip(cx, "chart-2"),
                    token_chip(cx, "chart-3"),
                ]
            })
            .gap(Space::N3)
            .wrap()
            .items_start()
            .into_element(cx),
            shadcn::raw::typography::muted(
                "Map chart colors from theme tokens so light/dark palette changes stay stable across charts, legends, and tooltip recipes.",
            )
            .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
}
// endregion: example
