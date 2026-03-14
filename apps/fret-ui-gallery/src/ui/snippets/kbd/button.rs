pub const SOURCE: &str = include_str!("button.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let accept = shadcn::Button::new("Accept")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .refine_style(ChromeRefinement::default().pr(Space::N2))
        .children([shadcn::Kbd::from_children([fret_ui_shadcn::kbd::kbd_icon(
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

    ui::h_row(move |_cx| vec![accept, cancel])
        .gap(Space::N4)
        .items_center()
        .into_element(cx)
        .test_id("ui-gallery-kbd-button")
}
// endregion: example
