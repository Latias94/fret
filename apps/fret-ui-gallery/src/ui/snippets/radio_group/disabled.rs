pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
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
