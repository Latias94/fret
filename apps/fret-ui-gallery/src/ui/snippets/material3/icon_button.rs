pub const SOURCE: &str = include_str!("icon_button.rs");

// region: example
use fret_icons::ids;
use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

#[derive(Default)]
struct Models {
    toggle_checked: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let toggle_checked = cx.with_state(Models::default, |st| st.toggle_checked.clone());
    let toggle_checked = match toggle_checked {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.toggle_checked = Some(m.clone()));
            m
        }
    };

    let row = |cx: &mut ElementContext<'_, H>,
               variant: material3::IconButtonVariant,
               label: &'static str| {
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                let hover_accent = fret_ui_kit::colors::linear_from_hex_rgb(0xe5_33_e5);
                let override_style = material3::IconButtonStyle::default()
                    .icon_color(
                        WidgetStateProperty::new(None)
                            .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_accent))),
                    )
                    .state_layer_color(
                        WidgetStateProperty::new(None)
                            .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_accent))),
                    );
                vec![
                    material3::IconButton::new(ids::ui::CLOSE)
                        .variant(variant)
                        .a11y_label(label)
                        .into_element(cx),
                    material3::IconButton::new(ids::ui::CLOSE)
                        .variant(variant)
                        .a11y_label("Override")
                        .style(override_style)
                        .into_element(cx),
                    material3::IconButton::new(ids::ui::CLOSE)
                        .variant(variant)
                        .a11y_label("Disabled")
                        .disabled(true)
                        .into_element(cx),
                ]
            },
        )
    };

    let toggles = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            let checked = cx
                .get_model_copied(&toggle_checked, Invalidation::Layout)
                .unwrap_or(false);
            vec![
                material3::IconToggleButton::new(toggle_checked.clone(), ids::ui::CHECK)
                    .variant(material3::IconButtonVariant::Filled)
                    .a11y_label("Material 3 Icon Toggle Button")
                    .test_id("ui-gallery-material3-icon-toggle-button")
                    .into_element(cx),
                cx.text(format!("checked={checked}"))
                    .test_id("ui-gallery-material3-icon-toggle-button-checked"),
            ]
        },
    );

    let centered_gate = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            vec![
                material3::IconButton::new(ids::ui::CLOSE)
                    .variant(material3::IconButtonVariant::Filled)
                    .a11y_label("Material 3 icon button (centered chrome)")
                    .test_id("ui-gallery-material3-icon-button-centered")
                    .into_element(cx),
            ]
        },
    );

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start(),
        move |cx| {
            vec![
                cx.text(
                    "Material 3 Icon Buttons: token-driven colors + state layer + bounded ripple.",
                ),
                cx.text(
                    "Centered fixed chrome: hit box can exceed visual chrome (min touch target).",
                ),
                centered_gate,
                row(cx, material3::IconButtonVariant::Standard, "Standard"),
                row(cx, material3::IconButtonVariant::Filled, "Filled"),
                row(cx, material3::IconButtonVariant::Tonal, "Tonal"),
                row(cx, material3::IconButtonVariant::Outlined, "Outlined"),
                toggles,
            ]
        },
    )
    .into()
}

// endregion: example
