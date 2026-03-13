pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::radio_group_uncontrolled(
        Some("option-one"),
        vec![
            shadcn::RadioGroupItem::new("option-one", "Option One"),
            shadcn::RadioGroupItem::new("option-two", "Option Two"),
        ],
    )
    .a11y_label("Choose an option")
    .into_element(cx)
    .test_id("ui-gallery-radio-group-usage")
}
// endregion: example
