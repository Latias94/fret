pub const SOURCE: &str = include_str!("checkbox.rs");

// region: example
use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, checked: Model<bool>) -> AnyElement {
    let tristate = cx.local_model_keyed("tristate", || None::<bool>);

    let value = cx
        .get_model_copied(&checked, Invalidation::Layout)
        .unwrap_or(false);
    let tristate_value = cx
        .get_model_cloned(&tristate, Invalidation::Layout)
        .unwrap_or(None);

    let row = ui::h_row(move |cx| {
        let selected_accent = fret_ui_kit::colors::linear_from_hex_rgb(0x33_cc_66);
        let override_style = material3::CheckboxStyle::default()
            .icon_color(WidgetStateProperty::new(None).when(
                WidgetStates::SELECTED,
                Some(ColorRef::Color(selected_accent)),
            ))
            .outline_color(WidgetStateProperty::new(None).when(
                WidgetStates::SELECTED,
                Some(ColorRef::Color(selected_accent)),
            ));
        vec![
            material3::Checkbox::new(checked.clone())
                .a11y_label("Material 3 Checkbox")
                .test_id("ui-gallery-material3-checkbox")
                .into_element(cx),
            material3::Checkbox::new(checked.clone())
                .a11y_label("Material 3 Checkbox (override)")
                .style(override_style)
                .test_id("ui-gallery-material3-checkbox-overridden")
                .into_element(cx),
            cx.text(format!("checked={}", value as u8)),
            material3::Checkbox::new(checked.clone())
                .a11y_label("Disabled Material 3 Checkbox")
                .disabled(true)
                .test_id("ui-gallery-material3-checkbox-disabled")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let tristate_row = ui::h_row(move |cx| {
        let label = match tristate_value {
            Some(true) => "checked",
            Some(false) => "unchecked",
            None => "indeterminate",
        };
        vec![
            material3::Checkbox::new_optional(tristate.clone())
                .a11y_label("Material 3 Checkbox (tri-state)")
                .test_id("ui-gallery-material3-checkbox-tristate")
                .into_element(cx),
            cx.text(format!("state={label}"))
                .test_id("ui-gallery-material3-checkbox-tristate-state"),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text(
                "Material 3 Checkbox: token-driven sizing/colors + state layer + bounded ripple.",
            ),
            row,
            cx.text(
                "Material 3 Checkbox (tri-state): `checked: None` represents indeterminate/mixed.",
            ),
            tristate_row,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .items_start()
    .into_element(cx)
    .into()
}

// endregion: example
