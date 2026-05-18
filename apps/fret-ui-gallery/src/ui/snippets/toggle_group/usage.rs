pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::ToggleGroup::single_uncontrolled(Option::<&'static str>::None)
        .items([
            shadcn::ToggleGroupItem::new("a", [decl_text::text_button_label(cx, "A")])
                .a11y_label("Toggle A"),
            shadcn::ToggleGroupItem::new("b", [decl_text::text_button_label(cx, "B")])
                .a11y_label("Toggle B"),
            shadcn::ToggleGroupItem::new("c", [decl_text::text_button_label(cx, "C")])
                .a11y_label("Toggle C"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-usage")
}
// endregion: example
