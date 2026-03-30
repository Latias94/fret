pub const SOURCE: &str = include_str!("spacing.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &'static str,
    label: &'static str,
    icon_id: &'static str,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::new(
        value,
        [icon::icon(cx, IconId::new_static(icon_id)), cx.text(label)],
    )
    .a11y_label(format!("Toggle {}", value))
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::ToggleGroup::multiple_uncontrolled(std::iter::empty::<&'static str>())
        .variant(shadcn::ToggleVariant::Outline)
        .size(shadcn::ToggleSize::Sm)
        .spacing(Space::N2)
        .items([
            item(cx, "star", "Star", "lucide.star"),
            item(cx, "heart", "Heart", "lucide.heart"),
            item(cx, "bookmark", "Bookmark", "lucide.bookmark"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-spacing")
}
// endregion: example
