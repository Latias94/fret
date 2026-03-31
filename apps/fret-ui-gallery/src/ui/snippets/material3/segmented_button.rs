pub const SOURCE: &str = include_str!("segmented_button.rs");

// region: example
use std::collections::BTreeSet;
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_ui_material3::{SegmentedButtonItem, SegmentedButtonSet};
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let single_value = cx.local_model_keyed("single_value", || Arc::<str>::from("alpha"));
    let multi_value = cx.local_model_keyed("multi_value", || {
        [Arc::<str>::from("alpha")]
            .into_iter()
            .collect::<BTreeSet<_>>()
    });

    let single_current = cx
        .get_model_cloned(&single_value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));
    let multi_current_len = cx
        .get_model_cloned(&multi_value, Invalidation::Layout)
        .map(|set| set.len())
        .unwrap_or(0);

    ui::v_flex(|cx| {
        vec![
            cx.text(
                "Material 3 Segmented Buttons: token-driven outcomes + roving focus + selection.",
            ),
            SegmentedButtonSet::single(single_value.clone())
                .items(vec![
                    SegmentedButtonItem::new("alpha", "Alpha")
                        .icon(fret_icons::ids::ui::SEARCH)
                        .test_id("ui-gallery-material3-segmented-single-alpha"),
                    SegmentedButtonItem::new("beta", "Beta")
                        .icon(fret_icons::ids::ui::SETTINGS)
                        .test_id("ui-gallery-material3-segmented-single-beta"),
                    SegmentedButtonItem::new("gamma", "Gamma (disabled)")
                        .disabled(true)
                        .icon(fret_icons::ids::ui::MORE_HORIZONTAL)
                        .test_id("ui-gallery-material3-segmented-single-gamma-disabled"),
                ])
                .a11y_label("Material 3 Segmented Button (single)")
                .test_id("ui-gallery-material3-segmented-single")
                .into_element(cx),
            cx.text(format!("single={}", single_current.as_ref())),
            SegmentedButtonSet::multi(multi_value.clone())
                .items(vec![
                    SegmentedButtonItem::new("alpha", "Alpha")
                        .test_id("ui-gallery-material3-segmented-multi-alpha"),
                    SegmentedButtonItem::new("beta", "Beta")
                        .test_id("ui-gallery-material3-segmented-multi-beta"),
                    SegmentedButtonItem::new("gamma", "Gamma (disabled)")
                        .disabled(true)
                        .test_id("ui-gallery-material3-segmented-multi-gamma-disabled"),
                ])
                .a11y_label("Material 3 Segmented Button (multi)")
                .test_id("ui-gallery-material3-segmented-multi")
                .into_element(cx),
            cx.text(format!("multi_count={multi_current_len}")),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .items_start()
    .into_element(cx)
}

// endregion: example
