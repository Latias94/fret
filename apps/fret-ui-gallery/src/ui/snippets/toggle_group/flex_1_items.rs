pub const SOURCE: &str = include_str!("flex_1_items.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn text_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &'static str,
    label: &'static str,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::new(value, [cx.text(label)]).a11y_label(format!("Toggle {label}"))
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ToggleGroup::single_uncontrolled(Some("left"))
        .variant(shadcn::ToggleVariant::Outline)
        .items_flex_1(true)
        .refine_layout(LayoutRefinement::default().w_full())
        .items([
            text_item(cx, "left", "Left").test_id("ui-gallery-toggle-group-stretch-left"),
            text_item(cx, "center", "Center").test_id("ui-gallery-toggle-group-stretch-center"),
            text_item(cx, "right", "Right").test_id("ui-gallery-toggle-group-stretch-right"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-stretch")
}
// endregion: example
