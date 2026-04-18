pub const SOURCE: &str = include_str!("text_field.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};
use fret_ui_material3 as material3;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let demo_field = material3::TextField::uncontrolled(cx);
    let value = demo_field.value_model();
    let disabled_toggle = material3::Switch::uncontrolled(cx, false);
    let disabled = disabled_toggle.selected_model();
    let error_toggle = material3::Switch::uncontrolled(cx, false);
    let error = error_toggle.selected_model();
    let icons_value = cx.local_model_keyed("icons_value", String::new);

    let disabled_now = cx
        .get_model_copied(&disabled, Invalidation::Layout)
        .unwrap_or(false);
    let error_now = cx
        .get_model_copied(&error, Invalidation::Layout)
        .unwrap_or(false);

    let toggles = ui::h_row(move |cx| {
        vec![
            cx.text("disabled"),
            disabled_toggle
                .clone()
                .a11y_label("Disable text field")
                .test_id("ui-gallery-material3-text-field-disabled")
                .into_element(cx),
            cx.text("error"),
            error_toggle
                .clone()
                .a11y_label("Toggle error state")
                .test_id("ui-gallery-material3-text-field-error")
                .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .items_center()
    .into_element(cx);

    let supporting = if error_now {
        "Error: required"
    } else {
        "Supporting text"
    };

    let outlined_field = demo_field
        .clone()
        .variant(material3::TextFieldVariant::Outlined)
        .label("Name")
        .placeholder("Type here")
        .supporting_text(supporting)
        .leading_icon(fret_icons::ids::ui::SEARCH)
        .disabled(disabled_now)
        .error(error_now)
        .test_id("ui-gallery-material3-text-field")
        .into_element(cx);

    let outlined_card = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Outlined"),
                    shadcn::card_description(
                        "Animated label + outline \"notch\" patch (best-effort).",
                    ),
                ]
            }),
            shadcn::card_content(move |_cx| vec![outlined_field]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let filled_field = material3::TextField::new(value.clone())
        .variant(material3::TextFieldVariant::Filled)
        .label("Email")
        .placeholder("name@example.com")
        .supporting_text(supporting)
        .leading_icon(fret_icons::ids::ui::FILE)
        .disabled(disabled_now)
        .error(error_now)
        .test_id("ui-gallery-material3-text-field-filled")
        .into_element(cx);

    let filled_card = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Filled"),
                    shadcn::card_description(
                        "Active indicator bottom border + filled container + hover state layer via foundation indication (best-effort).",
                    ),
                ]
            }),
            shadcn::card_content(move |_cx| vec![filled_field]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let focus_accent = fret_ui_kit::colors::linear_from_hex_rgb(0x33_cc_66);
    let hover_accent = fret_ui_kit::colors::linear_from_hex_rgb(0xe5_33_e5);

    let override_style = material3::TextFieldStyle::default()
        .outline_color(WidgetStateProperty::new(None).when(
            WidgetStates::FOCUS_VISIBLE,
            Some(ColorRef::Color(focus_accent)),
        ))
        .caret_color(WidgetStateProperty::new(Some(ColorRef::Color(
            focus_accent,
        ))))
        .placeholder_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_accent))),
        );
    let override_field = material3::TextField::new(value)
        .variant(material3::TextFieldVariant::Outlined)
        .label("Override")
        .placeholder("Hover/focus to see overrides")
        .supporting_text("Caret + focus outline + hover placeholder via TextFieldStyle")
        .style(override_style)
        .disabled(disabled_now)
        .error(error_now)
        .test_id("ui-gallery-material3-text-field-overridden")
        .into_element(cx);
    let override_card = shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Override"),
                    shadcn::card_description(
                        "ADR 0220: partial per-state overrides via TextFieldStyle.",
                    ),
                ]
            }),
            shadcn::card_content(move |_cx| vec![override_field]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    ui::v_flex(|cx| {
            vec![
                cx.text(
                    "Material 3 Text Field: outlined + filled variants (token-driven chrome + label/placeholder outcomes).",
                ),
                toggles,
                outlined_card,
                filled_card,
                override_card,
                shadcn::card(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_header(|cx| {
                            ui::children![
                                cx;
                                shadcn::card_title("Icons"),
                                shadcn::card_description(
                                    "Leading/trailing icon slots with minimum touch target hit regions.",
                                ),
                            ]
                        }),
                        shadcn::card_content(|cx| {
                            vec![
                                material3::TextField::new(icons_value)
                                    .variant(material3::TextFieldVariant::Outlined)
                                    .label("Search")
                                    .placeholder("Query")
                                    .supporting_text(
                                        "Leading icon is decorative; trailing icon is pressable.",
                                    )
                                    .leading_icon(fret_icons::ids::ui::SEARCH)
                                    .trailing_icon(fret_icons::ids::ui::CHEVRON_DOWN)
                                    .trailing_icon_a11y_label("Toggle suggestions")
                                    .test_id("ui-gallery-material3-text-field-icons")
                                    .into_element(cx),
                            ]
                        }),
                    ]
                })
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx),
            ]
        })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start().into_element(cx)
}

// endregion: example
