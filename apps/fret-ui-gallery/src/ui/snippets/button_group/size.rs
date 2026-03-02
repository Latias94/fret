pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let small = shadcn::ButtonGroup::new([
        shadcn::Button::new("Small")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into(),
        shadcn::Button::new("Button")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into(),
    ])
    .into_element(cx);

    let medium = shadcn::ButtonGroup::new([
        shadcn::Button::new("Default")
            .variant(shadcn::ButtonVariant::Outline)
            .into(),
        shadcn::Button::new("Button")
            .variant(shadcn::ButtonVariant::Outline)
            .into(),
    ])
    .into_element(cx);

    let large = shadcn::ButtonGroup::new([
        shadcn::Button::new("Large")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Lg)
            .into(),
        shadcn::Button::new("Button")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Lg)
            .into(),
    ])
    .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |_cx| vec![small, medium, large],
    )
    .test_id("ui-gallery-button-group-size")
}

// endregion: example
