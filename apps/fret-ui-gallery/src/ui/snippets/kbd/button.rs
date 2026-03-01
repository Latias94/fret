// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let accept = shadcn::Button::new("Accept")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .refine_style(ChromeRefinement::default().pr(Space::N2))
        .children([shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
            cx,
            fret_icons::IconId::new_static("lucide.corner-down-left"),
        )])
        .into_element(cx)])
        .into_element(cx)
        .test_id("ui-gallery-kbd-button-accept");

    let cancel = shadcn::Button::new("Cancel")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .refine_style(ChromeRefinement::default().pr(Space::N2))
        .children([shadcn::Kbd::new("Esc").into_element(cx)])
        .into_element(cx)
        .test_id("ui-gallery-kbd-button-cancel");

    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |_cx| vec![accept, cancel],
    )
    .test_id("ui-gallery-kbd-button")
}
// endregion: example
