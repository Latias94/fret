pub const SOURCE: &str = include_str!("outline.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::ToggleGroup::multiple_uncontrolled(std::iter::empty::<&'static str>())
        .variant(shadcn::ToggleVariant::Outline)
        .items([
            shadcn::ToggleGroupItem::icon("bold", IconId::new_static("lucide.bold"))
                .child(decl_text::text_button_label(cx, "Bold"))
                .a11y_label("Toggle bold"),
            shadcn::ToggleGroupItem::icon("italic", IconId::new_static("lucide.italic"))
                .child(decl_text::text_button_label(cx, "Italic"))
                .a11y_label("Toggle italic"),
            shadcn::ToggleGroupItem::icon("strikethrough", IconId::new_static("lucide.underline"))
                .child(decl_text::text_button_label(cx, "Underline"))
                .a11y_label("Toggle strikethrough"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-outline")
}
// endregion: example
