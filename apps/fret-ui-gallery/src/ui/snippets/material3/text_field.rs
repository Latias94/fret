pub const SOURCE: &str = include_str!("text_field.rs");

// region: example
use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};
use fret_ui_material3 as material3;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: Model<String>,
    disabled: Model<bool>,
    error: Model<bool>,
) -> AnyElement {
    #[derive(Default)]
    struct LocalState {
        icons_value: Option<Model<String>>,
    }

    let icons_value = cx.with_state(LocalState::default, |st| st.icons_value.clone());
    let icons_value = if let Some(model) = icons_value {
        model
    } else {
        let model = cx.app.models_mut().insert(String::new());
        cx.with_state(LocalState::default, |st| {
            st.icons_value = Some(model.clone())
        });
        model
    };

    let disabled_now = cx
        .get_model_copied(&disabled, Invalidation::Layout)
        .unwrap_or(false);
    let error_now = cx
        .get_model_copied(&error, Invalidation::Layout)
        .unwrap_or(false);

    let toggles = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                cx.text("disabled"),
                material3::Switch::new(disabled.clone())
                    .a11y_label("Disable text field")
                    .test_id("ui-gallery-material3-text-field-disabled")
                    .into_element(cx),
                cx.text("error"),
                material3::Switch::new(error.clone())
                    .a11y_label("Toggle error state")
                    .test_id("ui-gallery-material3-text-field-error")
                    .into_element(cx),
            ]
        },
    );

    let supporting = if error_now {
        "Error: required"
    } else {
        "Supporting text"
    };

    let outlined_field = material3::TextField::new(value.clone())
        .variant(material3::TextFieldVariant::Outlined)
        .label("Name")
        .placeholder("Type here")
        .supporting_text(supporting)
        .leading_icon(fret_icons::ids::ui::SEARCH)
        .disabled(disabled_now)
        .error(error_now)
        .test_id("ui-gallery-material3-text-field")
        .into_element(cx);

    let outlined_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Outlined").into_element(cx),
            shadcn::CardDescription::new("Animated label + outline \"notch\" patch (best-effort).")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![outlined_field]).into_element(cx),
    ])
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

    let filled_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Filled").into_element(cx),
            shadcn::CardDescription::new(
                "Active indicator bottom border + filled container + hover state layer via foundation indication (best-effort).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![filled_field]).into_element(cx),
    ])
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
    let override_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Override").into_element(cx),
            shadcn::CardDescription::new(
                "ADR 0220: partial per-state overrides via TextFieldStyle.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![override_field]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start(),
        |cx| {
            vec![
                cx.text(
                    "Material 3 Text Field: outlined + filled variants (token-driven chrome + label/placeholder outcomes).",
                ),
                toggles,
                outlined_card,
                filled_card,
                override_card,
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Icons").into_element(cx),
                        shadcn::CardDescription::new(
                            "Leading/trailing icon slots with minimum touch target hit regions.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
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
                    ])
                    .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx),
            ]
        },
    )
    .into()
}

// endregion: example

