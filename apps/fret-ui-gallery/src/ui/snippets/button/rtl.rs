pub const SOURCE: &str = include_str!("rtl.rs");

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
        vec![with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
            ui::h_flex(|cx| {
                vec![
                    shadcn::Button::new("Default")
                        .test_id("ui-gallery-button-rtl-default")
                        .into_element(cx),
                    shadcn::Button::new("Secondary")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .test_id("ui-gallery-button-rtl-secondary")
                        .into_element(cx),
                    shadcn::Button::new("Destructive")
                        .variant(shadcn::ButtonVariant::Destructive)
                        .test_id("ui-gallery-button-rtl-destructive")
                        .into_element(cx),
                    shadcn::Button::new("Back")
                        .variant(shadcn::ButtonVariant::Outline)
                        .leading_icon(IconId::new_static("lucide.arrow-left"))
                        .test_id("ui-gallery-button-rtl-back")
                        .into_element(cx),
                    shadcn::Button::new("Next")
                        .variant(shadcn::ButtonVariant::Outline)
                        .trailing_icon(IconId::new_static("lucide.arrow-right"))
                        .test_id("ui-gallery-button-rtl-next")
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .layout(LayoutRefinement::default().w_full())
            .into_element(cx)
            .test_id("ui-gallery-button-rtl-row-inner")
        })]
    })
    .test_id("ui-gallery-button-rtl-row")
}
// endregion: example
