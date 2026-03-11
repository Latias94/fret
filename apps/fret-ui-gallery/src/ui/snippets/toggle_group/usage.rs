pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn text_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &'static str,
    label: &'static str,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::new(value, [cx.text(label)]).a11y_label(format!("Toggle {label}"))
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ToggleGroup::single_uncontrolled(Option::<&'static str>::None)
        .items([
            text_item(cx, "a", "A"),
            text_item(cx, "b", "B"),
            text_item(cx, "c", "C"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-usage")
}
// endregion: example
