pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::ToggleGroup::multiple_uncontrolled(std::iter::empty::<&'static str>())
        .variant(shadcn::ToggleVariant::Outline)
        .items([
            shadcn::ToggleGroupItem::icon("bold", IconId::new_static("lucide.bold"))
                .a11y_label("Toggle bold"),
            shadcn::ToggleGroupItem::icon("italic", IconId::new_static("lucide.italic"))
                .a11y_label("Toggle italic"),
            shadcn::ToggleGroupItem::icon("strikethrough", IconId::new_static("lucide.underline"))
                .a11y_label("Toggle strikethrough"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-demo")
}
// endregion: example
