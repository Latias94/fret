pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::ToggleGroup::single_uncontrolled(Option::<&'static str>::None)
        .disabled(true)
        .items([
            shadcn::ToggleGroupItem::icon("bold", IconId::new_static("lucide.bold"))
                .a11y_label("Toggle bold"),
            shadcn::ToggleGroupItem::icon("italic", IconId::new_static("lucide.italic"))
                .a11y_label("Toggle italic"),
            shadcn::ToggleGroupItem::icon("strikethrough", IconId::new_static("lucide.underline"))
                .a11y_label("Toggle strikethrough"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-disabled")
}
// endregion: example
