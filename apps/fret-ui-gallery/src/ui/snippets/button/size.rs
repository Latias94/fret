pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn wrap_row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
}

fn row<H: UiHost>(
    _cx: &mut ElementContext<'_, H>,
    label: &'static str,
    text_size: shadcn::ButtonSize,
    icon_size: shadcn::ButtonSize,
) -> impl IntoUiElement<H> + use<H> {
    wrap_row(move |cx| {
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
            )
            .into_element(cx),
            row(
                cx,
                "Small",
                shadcn::ButtonSize::Sm,
                shadcn::ButtonSize::IconSm,
            )
            .into_element(cx),
            row(
                cx,
                "Default",
                shadcn::ButtonSize::Default,
                shadcn::ButtonSize::Icon,
            )
            .into_element(cx),
            row(
                cx,
                "Large",
                shadcn::ButtonSize::Lg,
                shadcn::ButtonSize::IconLg,
            )
            .into_element(cx),
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-button-size")
}
// endregion: example
