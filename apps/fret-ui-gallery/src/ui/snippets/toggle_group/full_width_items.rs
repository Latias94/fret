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
