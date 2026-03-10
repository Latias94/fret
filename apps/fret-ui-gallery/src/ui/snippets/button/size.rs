pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn wrap_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}

fn row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    text_size: shadcn::ButtonSize,
    icon_size: shadcn::ButtonSize,
) -> AnyElement {
    wrap_row(cx, |cx| {
        vec![
            shadcn::Button::new(label)
                .variant(shadcn::ButtonVariant::Outline)
                .size(text_size)
                .into_element(cx),
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Outline)
                .size(icon_size)
                .a11y_label("Submit")
                .icon(IconId::new_static("lucide.arrow-up-right"))
                .into_element(cx),
        ]
    })
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
        vec![
            row(
                cx,
                "Extra Small",
                shadcn::ButtonSize::Xs,
                shadcn::ButtonSize::IconXs,
            ),
            row(
                cx,
                "Small",
                shadcn::ButtonSize::Sm,
                shadcn::ButtonSize::IconSm,
            ),
            row(
                cx,
                "Default",
                shadcn::ButtonSize::Default,
                shadcn::ButtonSize::Icon,
            ),
            row(
                cx,
                "Large",
                shadcn::ButtonSize::Lg,
                shadcn::ButtonSize::IconLg,
            ),
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-button-size")
}
// endregion: example
