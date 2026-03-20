pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::RadioGroup::uncontrolled(Some("option2"))
        .a11y_label("Options")
        .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
        .item(shadcn::RadioGroupItem::new("option1", "Disabled").disabled(true))
        .item(shadcn::RadioGroupItem::new("option2", "Option 2"))
        .item(shadcn::RadioGroupItem::new("option3", "Option 3"))
        .into_element(cx)
        .test_id("ui-gallery-radio-group-disabled")
}
// endregion: example
