pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn icon_item(value: &'static str, label: &'static str) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::icon(
        value,
        IconId::new_static(match value {
            "bold" => "lucide.bold",
            "italic" => "lucide.italic",
            _ => "lucide.underline",
        }),
    )
    .a11y_label(label)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ToggleGroup::multiple_uncontrolled(["bold"])
        .disabled(true)
        .variant(shadcn::ToggleVariant::Outline)
        .items([
            icon_item("bold", "Toggle bold"),
            icon_item("italic", "Toggle italic"),
            icon_item("underline", "Toggle underline"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-disabled")
}
// endregion: example
