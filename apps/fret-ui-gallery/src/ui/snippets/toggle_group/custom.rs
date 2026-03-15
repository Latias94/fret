pub const SOURCE: &str = include_str!("custom.rs");

// region: example
use fret::{UiChild, UiCx};
use std::sync::Arc;

use fret_core::{Color, FontWeight, Px};
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn weight_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: &'static str,
    label: &'static str,
    weight: FontWeight,
    muted: Color,
) -> shadcn::ToggleGroupItem {
    shadcn::ToggleGroupItem::new(
        value,
        [ui::v_flex(|cx| {
            vec![
                ui::text("Aa")
                    .text_size_px(Px(24.0))
                    .line_height_px(Px(24.0))
                    .font_weight(weight)
                    .into_element(cx),
                ui::text(label)
                    .text_size_px(Px(12.0))
                    .line_height_px(Px(12.0))
                    .text_color(ColorRef::Color(muted))
                    .into_element(cx),
            ]
        })
        .gap(Space::N1)
        .items_center()
        .justify_center()
        .w_px(Px(64.0))
        .h_px(Px(64.0))
        .into_element(cx)],
    )
    .a11y_label(label)
    .refine_layout(LayoutRefinement::default().w_px(Px(64.0)).h_px(Px(64.0)))
    .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || Some(Arc::<str>::from("normal")));
    let current = cx
        .app
        .models()
        .get_cloned(&value)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("normal"));
    let muted = cx.with_theme(|theme| theme.color_token("muted-foreground"));

    shadcn::Field::new([
        shadcn::FieldLabel::new("Font Weight").into_element(cx),
        shadcn::ToggleGroup::single(value.clone())
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Lg)
            .spacing(Space::N2)
            .items([
                weight_item(cx, "light", "Light", FontWeight::LIGHT, muted),
                weight_item(cx, "normal", "Normal", FontWeight::NORMAL, muted),
                weight_item(cx, "medium", "Medium", FontWeight::MEDIUM, muted),
                weight_item(cx, "bold", "Bold", FontWeight::BOLD, muted),
            ])
            .into_element(cx),
        shadcn::FieldDescription::new(format!(
            "Use `font-{}` to set the font weight.",
            current.as_ref()
        ))
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-toggle-group-custom")
}
// endregion: example
