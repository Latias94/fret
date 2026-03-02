pub const SOURCE: &str = include_str!("buttons.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let spinner = |cx: &mut ElementContext<'_, H>| shadcn::Spinner::new().into_element(cx);

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N2)
            .items_center()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::Button::new("Submit")
                    .children([spinner(cx)])
                    .into_element(cx),
                shadcn::Button::new("Disabled")
                    .disabled(true)
                    .children([spinner(cx)])
                    .into_element(cx),
                shadcn::Button::new("Small")
                    .size(shadcn::ButtonSize::Sm)
                    .children([spinner(cx)])
                    .into_element(cx),
                shadcn::Button::new("Outline")
                    .variant(shadcn::ButtonVariant::Outline)
                    .disabled(true)
                    .children([spinner(cx)])
                    .into_element(cx),
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Icon)
                    .disabled(true)
                    .children([spinner(cx)])
                    .into_element(cx)
                    .attach_semantics(SemanticsDecoration::default().label("Loading..."))
                    .test_id("ui-gallery-spinner-button-icon-only"),
                shadcn::Button::new("Remove")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .disabled(true)
                    .children([spinner(cx)])
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-spinner-buttons")
}

// endregion: example
