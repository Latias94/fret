pub const SOURCE: &str = include_str!("spacing.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::ToggleGroup::single_uncontrolled(Some("top"))
        .variant(shadcn::ToggleVariant::Outline)
        .size(shadcn::ToggleSize::Sm)
        .spacing(Space::N2)
        .items([
            shadcn::ToggleGroupItem::new("top", [cx.text("Top")]).a11y_label("Toggle top"),
            shadcn::ToggleGroupItem::new("bottom", [cx.text("Bottom")]).a11y_label("Toggle bottom"),
            shadcn::ToggleGroupItem::new("left", [cx.text("Left")]).a11y_label("Toggle left"),
            shadcn::ToggleGroupItem::new("right", [cx.text("Right")]).a11y_label("Toggle right"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-spacing")
}
// endregion: example
