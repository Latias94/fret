pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::RadioGroup::uncontrolled(Some("option-one"))
        .a11y_label("Choose an option")
        .item(shadcn::RadioGroupItem::new("option-one", "Option One"))
        .item(shadcn::RadioGroupItem::new("option-two", "Option Two"))
        .into_element(cx)
        .test_id("ui-gallery-radio-group-usage")
}
// endregion: example
