pub const SOURCE: &str = include_str!("icon.rs");

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

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    wrap_row(cx, |cx| {
        vec![
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconSm)
                .a11y_label("Open")
                .icon(IconId::new_static("lucide.arrow-up-right"))
                .into_element(cx),
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .a11y_label("Open")
                .icon(IconId::new_static("lucide.arrow-up-right"))
                .into_element(cx),
            shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconLg)
                .a11y_label("Open")
                .icon(IconId::new_static("lucide.arrow-up-right"))
                .into_element(cx),
        ]
    })
    .test_id("ui-gallery-button-icon-only")
}
// endregion: example
