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
    shadcn::ToggleGroup::single_uncontrolled(Some("top"))
        .variant(shadcn::ToggleVariant::Outline)
        .size(shadcn::ToggleSize::Sm)
        .spacing(Space::N2)
        .items([
            text_item(cx, "top", "Top"),
            text_item(cx, "bottom", "Bottom"),
            text_item(cx, "left", "Left"),
            text_item(cx, "right", "Right"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-spacing")
}
// endregion: example

