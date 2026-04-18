pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn text_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &'static str,
    label: &'static str,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::new(value, [cx.text(label)]).a11y_label(label)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::ToggleGroup::single_uncontrolled(Some("list"))
            .variant(shadcn::ToggleVariant::Outline)
            .items([
                text_item(cx, "list", "قائمة"),
                text_item(cx, "grid", "شبكة"),
                text_item(cx, "cards", "بطاقات"),
            ])
            .into_element(cx)
    })
    .test_id("ui-gallery-toggle-group-rtl")
}
// endregion: example
