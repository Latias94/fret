pub const SOURCE: &str = include_str!("large.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

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

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::ToggleGroup::multiple_uncontrolled(std::iter::empty::<&'static str>())
        .size(shadcn::ToggleSize::Lg)
        .items([
            icon_item("bold", "Toggle bold"),
            icon_item("italic", "Toggle italic"),
            icon_item("strikethrough", "Toggle strikethrough"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-large")
}
// endregion: example
