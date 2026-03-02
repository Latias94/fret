// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn text_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &'static str,
    label: &'static str,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::new(value, [cx.text(label)]).a11y_label(format!("Toggle {label}"))
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::ToggleGroup::single_uncontrolled(Some("list"))
                .variant(shadcn::ToggleVariant::Outline)
                .items([
                    text_item(cx, "list", "List"),
                    text_item(cx, "grid", "Grid"),
                    text_item(cx, "cards", "Cards"),
                ])
                .into_element(cx)
        },
    )
    .test_id("ui-gallery-toggle-group-rtl")
}
// endregion: example
