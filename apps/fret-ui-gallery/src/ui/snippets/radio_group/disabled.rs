pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::RadioGroup::uncontrolled(Some("option-two"))
        .a11y_label("Options")
        .disabled(true)
        .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
        .item(shadcn::RadioGroupItem::new("option-one", "Option One"))
        .item(shadcn::RadioGroupItem::new("option-two", "Option Two"))
        .item(shadcn::RadioGroupItem::new("option-three", "Option Three"))
        .into_element(cx)
        .test_id("ui-gallery-radio-group-disabled")
}
// endregion: example
