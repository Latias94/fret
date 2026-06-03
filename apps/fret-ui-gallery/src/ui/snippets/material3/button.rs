pub const SOURCE: &str = include_str!("button.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let sizes_with_icons = |cx: &mut AppComponentCx<'_>,
                            variant: material3::ButtonVariant,
                            id_prefix: &'static str| {
        ui::h_row(move |cx| {
            vec![
                material3::Button::new("XS")
                    .variant(variant)
                    .size(material3::ButtonSize::XSmall)
                    .leading_icon(fret_icons::ids::ui::SEARCH)
                    .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                    .test_id(format!("{id_prefix}-xsmall"))
                    .into_element(cx),
                material3::Button::new("S")
                    .variant(variant)
                    .size(material3::ButtonSize::Small)
                    .leading_icon(fret_icons::ids::ui::SEARCH)
                    .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                    .test_id(format!("{id_prefix}-small"))
                    .into_element(cx),
                material3::Button::new("M")
                    .variant(variant)
                    .size(material3::ButtonSize::Medium)
                    .leading_icon(fret_icons::ids::ui::SEARCH)
                    .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                    .test_id(format!("{id_prefix}-medium"))
                    .into_element(cx),
                material3::Button::new("L")
                    .variant(variant)
                    .size(material3::ButtonSize::Large)
                    .leading_icon(fret_icons::ids::ui::SEARCH)
                    .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                    .test_id(format!("{id_prefix}-large"))
                    .into_element(cx),
                material3::Button::new("XL")
                    .variant(variant)
                    .size(material3::ButtonSize::XLarge)
                    .leading_icon(fret_icons::ids::ui::SEARCH)
                    .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                    .test_id(format!("{id_prefix}-xlarge"))
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx)
    };

    let variant_row = |cx: &mut AppComponentCx<'_>,
                       variant: material3::ButtonVariant,
                       label: &'static str,
                       id_prefix: &'static str| {
        let (hover_container, hover_label) = cx.with_theme(|theme| {
            (
                theme.color_token("md.sys.color.tertiary-container"),
                theme.color_token("md.sys.color.on-tertiary-container"),
            )
        });

        ui::h_row(move |cx| {
            let hover_style = material3::ButtonStyle::default()
                .container_background(WidgetStateProperty::new(None).when(
                    WidgetStates::HOVERED,
                    Some(ColorRef::Color(hover_container)),
                ))
                .label_color(
                    WidgetStateProperty::new(None)
                        .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_label))),
                );

            let accent = fret_ui_kit::colors::linear_from_hex_rgb(0xe5_33_e5);
            let override_style = material3::ButtonStyle::default()
                .label_color(WidgetStateProperty::new(Some(ColorRef::Color(accent))))
                .state_layer_color(
                    WidgetStateProperty::new(None)
                        .when(WidgetStates::HOVERED, Some(ColorRef::Color(accent))),
                );

            vec![
                material3::Button::new(label)
                    .variant(variant)
                    .test_id(format!("{id_prefix}-default"))
                    .into_element(cx),
                material3::Button::new("Override")
                    .variant(variant)
                    .style(override_style)
                    .test_id(format!("{id_prefix}-override"))
                    .into_element(cx),
                material3::Button::new("Disabled")
                    .variant(variant)
                    .disabled(true)
                    .test_id(format!("{id_prefix}-disabled"))
                    .into_element(cx),
                material3::Button::new("Hover Override")
                    .variant(variant)
                    .style(hover_style)
                    .test_id(format!("{id_prefix}-hover-override"))
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx)
    };

    ui::v_flex(move |cx| {
        vec![
            cx.text("Material 3 Buttons: token-driven colors + state layer + bounded ripple."),
            cx.text("Sizes (xsmall..xlarge) + leading/trailing icons:"),
            sizes_with_icons(
                cx,
                material3::ButtonVariant::Filled,
                "ui-gallery-material3-button-size-filled",
            ),
            sizes_with_icons(
                cx,
                material3::ButtonVariant::Outlined,
                "ui-gallery-material3-button-size-outlined",
            ),
            variant_row(
                cx,
                material3::ButtonVariant::Filled,
                "Filled",
                "ui-gallery-material3-button-filled",
            ),
            variant_row(
                cx,
                material3::ButtonVariant::Tonal,
                "Tonal",
                "ui-gallery-material3-button-tonal",
            ),
            variant_row(
                cx,
                material3::ButtonVariant::Elevated,
                "Elevated",
                "ui-gallery-material3-button-elevated",
            ),
            variant_row(
                cx,
                material3::ButtonVariant::Outlined,
                "Outlined",
                "ui-gallery-material3-button-outlined",
            ),
            variant_row(
                cx,
                material3::ButtonVariant::Text,
                "Text",
                "ui-gallery-material3-button-text",
            ),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .items_start()
    .into_element(cx)
}

// endregion: example
