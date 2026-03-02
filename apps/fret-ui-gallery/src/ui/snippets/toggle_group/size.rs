pub const SOURCE: &str = include_str!("size.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn text_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &'static str,
    label: &'static str,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::new(value, [cx.text(label)]).a11y_label(format!("Toggle {label}"))
}

fn group<H: UiHost>(cx: &mut ElementContext<'_, H>, size: shadcn::ToggleSize) -> AnyElement {
    shadcn::ToggleGroup::single_uncontrolled(Some("left"))
        .variant(shadcn::ToggleVariant::Outline)
        .size(size)
        .items([
            text_item(cx, "left", "Left"),
            text_item(cx, "center", "Center"),
            text_item(cx, "right", "Right"),
        ])
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let sm = group(cx, shadcn::ToggleSize::Sm);
    let default = group(cx, shadcn::ToggleSize::Default);
    let lg = group(cx, shadcn::ToggleSize::Lg);

    stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2).items_start(),
        move |cx| {
            vec![
                shadcn::typography::muted(cx, "Sm / Default / Lg"),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |_cx| vec![sm, default, lg],
                ),
            ]
        },
    )
    .test_id("ui-gallery-toggle-group-size")
}
// endregion: example
