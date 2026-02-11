use super::super::super::*;

pub(in crate::ui) fn preview_material3_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let row = |cx: &mut ElementContext<'_, App>,
               variant: material3::ButtonVariant,
               label: &'static str| {
        let (hover_container, hover_label) = cx.with_theme(|theme| {
            (
                theme.color_required("md.sys.color.tertiary-container"),
                theme.color_required("md.sys.color.on-tertiary-container"),
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

                let accent = fret_core::Color {
                    r: 0.9,
                    g: 0.2,
                    b: 0.9,
                    a: 1.0,
                };
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

    vec![
        cx.text("Material 3 Buttons: token-driven colors + state layer + bounded ripple."),
        row(cx, material3::ButtonVariant::Filled, "Filled"),
        row(cx, material3::ButtonVariant::Tonal, "Tonal"),
        row(cx, material3::ButtonVariant::Elevated, "Elevated"),
        row(cx, material3::ButtonVariant::Outlined, "Outlined"),
        row(cx, material3::ButtonVariant::Text, "Text"),
    ]
}
