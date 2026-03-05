pub const SOURCE: &str = include_str!("buttons.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let button = |cx: &mut ElementContext<'_, H>,
                  label: &'static str,
                  variant: Option<shadcn::ButtonVariant>| {
        let mut btn = shadcn::Button::new(label)
            .disabled(true)
            .size(shadcn::ButtonSize::Sm);
        if let Some(variant) = variant {
            btn = btn.variant(variant);
        }
        btn.children([
            shadcn::Spinner::new().into_element(cx),
            ui::text(cx, label).font_medium().nowrap().into_element(cx),
        ])
        .into_element(cx)
    };

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_center()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                button(cx, "Loading...", None),
                button(cx, "Please wait", Some(shadcn::ButtonVariant::Outline)),
                button(cx, "Processing", Some(shadcn::ButtonVariant::Secondary)),
            ]
        },
    )
    .test_id("ui-gallery-spinner-buttons")
}

// endregion: example
