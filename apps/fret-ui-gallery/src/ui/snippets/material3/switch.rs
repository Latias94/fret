pub const SOURCE: &str = include_str!("switch.rs");

// region: example
use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

#[derive(Default)]
struct Models {
    icons_both: Option<Model<bool>>,
    icons_selected_only: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, selected: Model<bool>) -> AnyElement {
    let icons_both = cx.with_state(Models::default, |st| st.icons_both.clone());
    let icons_both = match icons_both {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.icons_both = Some(model.clone()));
            model
        }
    };

    let icons_selected_only = cx.with_state(Models::default, |st| st.icons_selected_only.clone());
    let icons_selected_only = match icons_selected_only {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.icons_selected_only = Some(model.clone())
            });
            model
        }
    };

    let value = cx
        .get_model_copied(&selected, Invalidation::Layout)
        .unwrap_or(false);

    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            let selected_accent = fret_ui_kit::colors::linear_from_hex_rgb(0x33_cc_66);
            let hover_accent = fret_ui_kit::colors::linear_from_hex_rgb(0xe5_33_e5);
            let override_style = material3::SwitchStyle::default()
                .track_color(WidgetStateProperty::new(None).when(
                    WidgetStates::SELECTED,
                    Some(ColorRef::Color(selected_accent)),
                ))
                .state_layer_color(
                    WidgetStateProperty::new(None)
                        .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_accent))),
                );
            vec![
                material3::Switch::new(selected.clone())
                    .a11y_label("Material 3 Switch")
                    .test_id("ui-gallery-material3-switch")
                    .into_element(cx),
                material3::Switch::new(selected.clone())
                    .a11y_label("Material 3 Switch (override)")
                    .style(override_style)
                    .test_id("ui-gallery-material3-switch-overridden")
                    .into_element(cx),
                cx.text(format!("selected={}", value as u8)),
                material3::Switch::new(selected.clone())
                    .a11y_label("Disabled Material 3 Switch")
                    .disabled(true)
                    .test_id("ui-gallery-material3-switch-disabled")
                    .into_element(cx),
            ]
        },
    );

    let icons_row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            let icons_both_value = cx
                .get_model_copied(&icons_both, Invalidation::Layout)
                .unwrap_or(false);
            let icons_selected_only_value = cx
                .get_model_copied(&icons_selected_only, Invalidation::Layout)
                .unwrap_or(false);
            vec![
                material3::Switch::new(icons_both.clone())
                    .icons(true)
                    .a11y_label("Material 3 Switch (icons)")
                    .test_id("ui-gallery-material3-switch-icons-both")
                    .into_element(cx),
                cx.text(format!("icons_both={}", icons_both_value as u8)),
                material3::Switch::new(icons_both.clone())
                    .icons(true)
                    .disabled(true)
                    .a11y_label("Disabled Material 3 Switch (icons)")
                    .test_id("ui-gallery-material3-switch-icons-both-disabled")
                    .into_element(cx),
                material3::Switch::new(icons_selected_only.clone())
                    .show_only_selected_icon(true)
                    .a11y_label("Material 3 Switch (selected icon only)")
                    .test_id("ui-gallery-material3-switch-icons-selected-only")
                    .into_element(cx),
                cx.text(format!(
                    "icons_selected_only={}",
                    icons_selected_only_value as u8
                )),
                material3::Switch::new(icons_selected_only.clone())
                    .show_only_selected_icon(true)
                    .disabled(true)
                    .a11y_label("Disabled Material 3 Switch (selected icon only)")
                    .test_id("ui-gallery-material3-switch-icons-selected-only-disabled")
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
                    "Material 3 Switch: token-driven sizing/colors + state layer + bounded ripple.",
                ),
                row,
                icons_row,
            ]
        },
    )
    .into()
}

// endregion: example
