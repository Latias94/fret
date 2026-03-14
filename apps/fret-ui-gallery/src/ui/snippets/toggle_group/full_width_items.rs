pub const SOURCE: &str = include_str!("full_width_items.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn text_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &'static str,
    label: &'static str,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::new(value, [cx.text(label)]).a11y_label(format!("Toggle {label}"))
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::ToggleGroup::multiple_uncontrolled(["left"])
        .variant(shadcn::ToggleVariant::Outline)
        .items_full_width(true)
        .refine_layout(LayoutRefinement::default().w_full())
        .items([
            text_item(cx, "left", "Left"),
            text_item(cx, "center", "Center"),
            text_item(cx, "right", "Right"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-full-width-items")
}
// endregion: example
