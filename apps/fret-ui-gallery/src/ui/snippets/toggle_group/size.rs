pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn text_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &'static str,
    label: &'static str,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::new(value, [cx.text(label)]).a11y_label(format!("Toggle {label}"))
}

fn group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: shadcn::ToggleSize,
) -> impl IntoUiElement<H> + use<H> {
    shadcn::ToggleGroup::single_uncontrolled(Some("top"))
        .variant(shadcn::ToggleVariant::Outline)
        .size(size)
        .items([
            text_item(cx, "top", "Top"),
            text_item(cx, "bottom", "Bottom"),
            text_item(cx, "left", "Left"),
            text_item(cx, "right", "Right"),
        ])
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let sm = group(cx, shadcn::ToggleSize::Sm).into_element(cx);
    let default = group(cx, shadcn::ToggleSize::Default).into_element(cx);

    ui::v_stack(move |_cx| vec![sm, default])
        .gap(Space::N4)
        .items_start()
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-size")
}
// endregion: example
