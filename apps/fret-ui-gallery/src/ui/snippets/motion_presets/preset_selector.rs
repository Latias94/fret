pub const SOURCE: &str = include_str!("preset_selector.rs");

// region: example
use fret_app::App;
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(
    cx: &mut ElementContext<'_, App>,
    motion_preset: Model<Option<Arc<str>>>,
    motion_preset_open: Model<bool>,
) -> AnyElement {
    let preset = cx
        .watch_model(&motion_preset)
        .paint()
        .value_or_default()
        .unwrap_or_else(|| Arc::from("theme"));

    let select = shadcn::Select::new(motion_preset, motion_preset_open)
        .value(shadcn::SelectValue::new().placeholder("Motion preset"))
        .trigger_test_id("ui-gallery-motion-presets-preset-trigger")
        .items([
            shadcn::SelectItem::new("theme", "Theme (baseline)")
                .test_id("ui-gallery-motion-preset-item-theme"),
            shadcn::SelectItem::new("reduced", "Reduced motion (0)")
                .test_id("ui-gallery-motion-preset-item-reduced"),
            shadcn::SelectItem::new("snappy", "Snappy")
                .test_id("ui-gallery-motion-preset-item-snappy"),
            shadcn::SelectItem::new("bouncy", "Bouncy")
                .test_id("ui-gallery-motion-preset-item-bouncy"),
            shadcn::SelectItem::new("gentle", "Gentle")
                .test_id("ui-gallery-motion-preset-item-gentle"),
        ])
        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
        .into_element(cx);

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Preset selector").into_element(cx),
            shadcn::CardDescription::new(
                "Applies a ThemeConfig patch (durations/easings/spring params) on top of the current theme preset.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([ui::h_flex(move |cx| {
                vec![
                    select,
                    shadcn::Badge::new(format!("active: {}", preset.as_ref()))
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                ]
            })
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N4)
                .items_center().into_element(cx)])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(760.0)).min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-motion-presets-selector-card")
}
// endregion: example
