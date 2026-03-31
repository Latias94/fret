pub const SOURCE: &str = include_str!("tabs.rs");

// region: example
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let tabs = material3::Tabs::uncontrolled(cx, "overview");
    let value = tabs.value_model();
    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let hover_accent = fret_ui_kit::colors::linear_from_hex_rgb(0xe5_33_e5);
    let active_accent = fret_ui_kit::colors::linear_from_hex_rgb(0x33_cc_66);

    let fixed_tabs = tabs
        .a11y_label("Material 3 Tabs")
        .test_id("ui-gallery-material3-tabs")
        .items(vec![
            material3::TabItem::new("overview", "Overview")
                .a11y_label("Tab Overview")
                .test_id("ui-gallery-material3-tab-overview"),
            material3::TabItem::new("settings", "Settings")
                .a11y_label("Tab Settings")
                .test_id("ui-gallery-material3-tab-settings"),
            material3::TabItem::new("disabled", "Disabled")
                .disabled(true)
                .a11y_label("Tab Disabled")
                .test_id("ui-gallery-material3-tab-disabled"),
        ])
        .into_element(cx);

    let override_style = material3::TabsStyle::default()
        .label_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_accent))),
        )
        .state_layer_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_accent))),
        )
        .active_indicator_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::SELECTED, Some(ColorRef::Color(active_accent))),
        );
    let fixed_tabs_overridden = material3::Tabs::new(value.clone())
        .a11y_label("Material 3 Tabs (overridden)")
        .test_id("ui-gallery-material3-tabs-overridden")
        .style(override_style)
        .items(vec![
            material3::TabItem::new("overview", "Overview")
                .a11y_label("Tab Overview")
                .test_id("ui-gallery-material3-tab-overview-overridden"),
            material3::TabItem::new("settings", "Settings")
                .a11y_label("Tab Settings")
                .test_id("ui-gallery-material3-tab-settings-overridden"),
            material3::TabItem::new("disabled", "Disabled")
                .disabled(true)
                .a11y_label("Tab Disabled")
                .test_id("ui-gallery-material3-tab-disabled-overridden"),
        ])
        .into_element(cx);

    let scrollable_tabs = material3::Tabs::new(value)
        .a11y_label("Material 3 Tabs (scrollable)")
        .test_id("ui-gallery-material3-tabs-scrollable")
        .scrollable(true)
        .items(vec![
            material3::TabItem::new("overview", "Overview"),
            material3::TabItem::new("settings", "Settings"),
            material3::TabItem::new("typography", "Typography"),
            material3::TabItem::new("very_long_label", "Very Long Label For Layout Probe"),
            material3::TabItem::new("tokens", "Tokens"),
            material3::TabItem::new("motion", "Motion"),
            material3::TabItem::new("disabled", "Disabled").disabled(true),
        ])
        .into_element(cx);

    ui::v_flex(|cx| {
        vec![
            cx.text("Material 3 Tabs: roving focus + state layer + bounded ripple."),
            fixed_tabs,
            cx.text(
                "Override preview: hover label/state-layer + active-indicator color via TabsStyle.",
            ),
            fixed_tabs_overridden,
            cx.text("Scrollable/variable width preview (measurement-driven indicator)."),
            scrollable_tabs,
            cx.text(format!("value={}", current.as_ref())),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .items_start()
    .into_element(cx)
}

// endregion: example
