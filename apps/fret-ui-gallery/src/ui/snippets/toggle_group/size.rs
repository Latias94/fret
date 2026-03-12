pub const SOURCE: &str = include_str!("size.rs");

// region: example
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
    shadcn::ToggleGroup::single_uncontrolled(Some("left"))
        .variant(shadcn::ToggleVariant::Outline)
        .size(size)
        .items([
            text_item(cx, "left", "Left"),
            text_item(cx, "center", "Center"),
            text_item(cx, "right", "Right"),
        ])
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let sm = group(cx, shadcn::ToggleSize::Sm).into_element(cx);
    let default = group(cx, shadcn::ToggleSize::Default).into_element(cx);
    let lg = group(cx, shadcn::ToggleSize::Lg).into_element(cx);

    ui::v_stack(move |cx| {
        vec![
            shadcn::raw::typography::muted("Sm / Default / Lg").into_element(cx),
            ui::h_row(move |_cx| vec![sm, default, lg])
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .into_element(cx)
    .test_id("ui-gallery-toggle-group-size")
}
// endregion: example
