pub const SOURCE: &str = include_str!("button.rs");

// region: example
use fret_ui_material3 as material3;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let sizes_with_icons = |cx: &mut ElementContext<'_, H>, variant: material3::ButtonVariant| {
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::Button::new("XS")
                        .variant(variant)
                        .size(material3::ButtonSize::XSmall)
                        .leading_icon(fret_icons::ids::ui::SEARCH)
                        .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                        .test_id("ui-gallery-material3-button-size-xsmall")
                        .into_element(cx),
                    material3::Button::new("S")
                        .variant(variant)
                        .size(material3::ButtonSize::Small)
                        .leading_icon(fret_icons::ids::ui::SEARCH)
                        .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                        .test_id("ui-gallery-material3-button-size-small")
                        .into_element(cx),
                    material3::Button::new("M")
                        .variant(variant)
                        .size(material3::ButtonSize::Medium)
                        .leading_icon(fret_icons::ids::ui::SEARCH)
                        .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                        .test_id("ui-gallery-material3-button-size-medium")
                        .into_element(cx),
                    material3::Button::new("L")
                        .variant(variant)
                        .size(material3::ButtonSize::Large)
                        .leading_icon(fret_icons::ids::ui::SEARCH)
                        .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                        .test_id("ui-gallery-material3-button-size-large")
                        .into_element(cx),
                    material3::Button::new("XL")
                        .variant(variant)
                        .size(material3::ButtonSize::XLarge)
                        .leading_icon(fret_icons::ids::ui::SEARCH)
                        .trailing_icon(fret_icons::ids::ui::CHEVRON_RIGHT)
                        .test_id("ui-gallery-material3-button-size-xlarge")
                        .into_element(cx),
                ]
            },
        )
    };

    let variant_row =
        |cx: &mut ElementContext<'_, H>, variant: material3::ButtonVariant, label: &'static str| {
            let (hover_container, hover_label) = cx.with_theme(|theme| {
                (
                    theme.color_token("md.sys.color.tertiary-container"),
                    theme.color_token("md.sys.color.on-tertiary-container"),
                )
            });

            stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                move |cx| {
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
                            .into_element(cx),
                        material3::Button::new("Override")
                            .variant(variant)
                            .style(override_style)
                            .into_element(cx),
                        material3::Button::new("Disabled")
                            .variant(variant)
                            .disabled(true)
                            .into_element(cx),
                        material3::Button::new("Hover Override")
                            .variant(variant)
                            .style(hover_style)
                            .into_element(cx),
                    ]
                },
            )
        };

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start(),
        move |cx| {
            vec![
                cx.text("Material 3 Buttons: token-driven colors + state layer + bounded ripple."),
                cx.text("Sizes (xsmall..xlarge) + leading/trailing icons:"),
                sizes_with_icons(cx, material3::ButtonVariant::Filled),
                sizes_with_icons(cx, material3::ButtonVariant::Outlined),
                variant_row(cx, material3::ButtonVariant::Filled, "Filled"),
                variant_row(cx, material3::ButtonVariant::Tonal, "Tonal"),
                variant_row(cx, material3::ButtonVariant::Elevated, "Elevated"),
                variant_row(cx, material3::ButtonVariant::Outlined, "Outlined"),
                variant_row(cx, material3::ButtonVariant::Text, "Text"),
            ]
        },
    )
    .into()
}

// endregion: example
