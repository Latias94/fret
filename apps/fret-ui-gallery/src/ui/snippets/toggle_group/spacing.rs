pub const SOURCE: &str = include_str!("spacing.rs");

// region: example
use fret_core::Color;
use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};
use fret_ui_shadcn::toggle_group::ToggleGroupStyle;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn accent_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &'static str,
    label: &'static str,
    icon: &'static str,
    selected_hex: u32,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::icon(value, IconId::new_static(icon))
        .child(cx.text(label))
        .a11y_label(format!("Toggle {label}"))
        .style(
            ToggleGroupStyle::default()
                .item_background(WidgetStateProperty::new(None).when(
                    WidgetStates::SELECTED,
                    Some(ColorRef::Color(Color::TRANSPARENT)),
                ))
                .item_foreground(WidgetStateProperty::new(None).when(
                    WidgetStates::SELECTED,
                    Some(ColorRef::Color(Color::from_srgb_hex_rgb(selected_hex))),
                )),
        )
        .test_id(format!("ui-gallery-toggle-group-spacing-{value}"))
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::ToggleGroup::multiple_uncontrolled(["star"])
        .variant(shadcn::ToggleVariant::Outline)
        .size(shadcn::ToggleSize::Sm)
        .spacing(Space::N2)
        .items([
            accent_item(cx, "star", "Star", "lucide.star", 0xeab308),
            accent_item(cx, "heart", "Heart", "lucide.heart", 0xef4444),
            accent_item(cx, "bookmark", "Bookmark", "lucide.bookmark", 0x3b82f6),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-spacing")
}
// endregion: example