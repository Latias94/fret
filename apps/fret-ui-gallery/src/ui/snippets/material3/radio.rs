pub const SOURCE: &str = include_str!("radio.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let group = material3::RadioGroup::uncontrolled(cx, None::<Arc<str>>);
    let group_value = group.value_model();
    let standalone = material3::Radio::uncontrolled(cx, false);
    let standalone_selected = standalone.selected_model();

    let current = cx
        .get_model_cloned(&group_value, Invalidation::Layout)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let row = ui::h_row(move |cx| {
        vec![
            group
                .clone()
                .a11y_label("Material 3 RadioGroup")
                .orientation(material3::RadioGroupOrientation::Horizontal)
                .gap(Px(8.0))
                .items(vec![
                    material3::RadioGroupItem::new("Alpha")
                        .a11y_label("Radio Alpha")
                        .test_id("ui-gallery-material3-radio-a"),
                    material3::RadioGroupItem::new("Beta")
                        .a11y_label("Radio Beta")
                        .test_id("ui-gallery-material3-radio-b"),
                    material3::RadioGroupItem::new("Charlie")
                        .a11y_label("Radio Charlie (disabled)")
                        .disabled(true)
                        .test_id("ui-gallery-material3-radio-c-disabled"),
                ])
                .into_element(cx),
            cx.text(format!("value={}", current.as_ref())),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .into_element(cx);

    let selected_accent = fret_ui_kit::colors::linear_from_hex_rgb(0x33_cc_66);
    let hover_accent = fret_ui_kit::colors::linear_from_hex_rgb(0xe5_33_e5);
    let override_style = material3::RadioStyle::default()
        .icon_color(WidgetStateProperty::new(None).when(
            WidgetStates::SELECTED,
            Some(ColorRef::Color(selected_accent)),
        ))
        .state_layer_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_accent))),
        );

    let override_style_for_group = override_style.clone();
    let override_style_for_standalone = override_style.clone();
    let group_overridden = ui::h_row(move |cx| {
        vec![
            material3::RadioGroup::new(group_value.clone())
                .a11y_label("Material 3 RadioGroup (override)")
                .style(override_style_for_group.clone())
                .orientation(material3::RadioGroupOrientation::Horizontal)
                .gap(Px(8.0))
                .items(vec![
                    material3::RadioGroupItem::new("Alpha")
                        .a11y_label("Radio Alpha (override)")
                        .test_id("ui-gallery-material3-radio-a-overridden"),
                    material3::RadioGroupItem::new("Beta")
                        .a11y_label("Radio Beta (override)")
                        .test_id("ui-gallery-material3-radio-b-overridden"),
                    material3::RadioGroupItem::new("Charlie")
                        .a11y_label("Radio Charlie (disabled)")
                        .disabled(true)
                        .test_id("ui-gallery-material3-radio-c-disabled-overridden"),
                ])
                .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .into_element(cx);

    let standalone = ui::h_row(move |cx| {
        vec![
            standalone
                .clone()
                .a11y_label("Material 3 Radio (standalone)")
                .test_id("ui-gallery-material3-radio-standalone")
                .into_element(cx),
            material3::Radio::new(standalone_selected.clone())
                .a11y_label("Material 3 Radio (override)")
                .style(override_style_for_standalone.clone())
                .test_id("ui-gallery-material3-radio-standalone-overridden")
                .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .into_element(cx);

    ui::v_flex(move |cx| {
            vec![
                cx.text("Material 3 Radio: group-value binding + roving focus + typeahead + state layer + bounded ripple."),
                row,
                cx.text("Override preview: RadioGroup::style(...) using RadioStyle."),
                group_overridden,
                cx.text("Override preview: standalone Radio::style(...) using RadioStyle."),
                standalone,
            ]
        })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start().into_element(cx)
}

// endregion: example
