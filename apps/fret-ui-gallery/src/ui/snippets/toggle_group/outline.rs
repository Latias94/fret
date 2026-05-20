pub const SOURCE: &str = include_str!("outline.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::ToggleGroup::single_uncontrolled(Some("all"))
        .variant(shadcn::ToggleVariant::Outline)
        .items([
            shadcn::ToggleGroupItem::new("all", [decl_text::text_button_label(cx, "All")])
                .a11y_label("Toggle all"),
            shadcn::ToggleGroupItem::new("missed", [decl_text::text_button_label(cx, "Missed")])
                .a11y_label("Toggle missed"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-outline")
}
// endregion: example
