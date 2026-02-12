use super::super::super::*;

pub(in crate::ui) fn preview_material3_icon_button(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    use fret_icons::ids;
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let row = |cx: &mut ElementContext<'_, App>,
               variant: material3::IconButtonVariant,
               label: &'static str| {
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                let override_style = material3::IconButtonStyle::default()
                    .icon_color(WidgetStateProperty::new(None).when(
                        WidgetStates::HOVERED,
                        Some(ColorRef::Color(fret_core::Color {
                            r: 0.9,
                            g: 0.2,
                            b: 0.9,
                            a: 1.0,
                        })),
                    ))
                    .state_layer_color(WidgetStateProperty::new(None).when(
                        WidgetStates::HOVERED,
                        Some(ColorRef::Color(fret_core::Color {
                            r: 0.9,
                            g: 0.2,
                            b: 0.9,
                            a: 1.0,
                        })),
                    ));
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
        |cx| {
            vec![
                material3::IconButton::new(ids::ui::CHECK)
                    .variant(material3::IconButtonVariant::Filled)
                    .toggle(true)
                    .selected(false)
                    .a11y_label("Toggle off")
                    .into_element(cx),
                material3::IconButton::new(ids::ui::CHECK)
                    .variant(material3::IconButtonVariant::Filled)
                    .toggle(true)
                    .selected(true)
                    .a11y_label("Toggle on")
                    .into_element(cx),
                material3::IconButton::new(ids::ui::CHECK)
                    .variant(material3::IconButtonVariant::Outlined)
                    .toggle(true)
                    .selected(false)
                    .a11y_label("Outlined off")
                    .into_element(cx),
                material3::IconButton::new(ids::ui::CHECK)
                    .variant(material3::IconButtonVariant::Outlined)
                    .toggle(true)
                    .selected(true)
                    .a11y_label("Outlined on")
                    .into_element(cx),
            ]
        },
    );

    vec![
        cx.text("Material 3 Icon Buttons: token-driven colors + state layer + bounded ripple."),
        row(cx, material3::IconButtonVariant::Standard, "Standard"),
        row(cx, material3::IconButtonVariant::Filled, "Filled"),
        row(cx, material3::IconButtonVariant::Tonal, "Tonal"),
        row(cx, material3::IconButtonVariant::Outlined, "Outlined"),
        toggles,
    ]
}

pub(in crate::ui) fn preview_material3_fab(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;
    use fret_ui::action::OnActivate;

    fn on_activate(id: &'static str, last_action: Model<Arc<str>>) -> OnActivate {
        Arc::new(move |host, _acx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from(id);
            });
        })
    }

    let row = {
        let last_action = last_action.clone();
        move |cx: &mut ElementContext<'_, App>,
              variant: material3::FabVariant,
              label: &'static str| {
            let last_action = last_action.clone();
            stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                move |cx| {
                    vec![
                        material3::Fab::new(ids::ui::SEARCH)
                            .variant(variant)
                            .a11y_label(label)
                            .on_activate(on_activate(
                                "material3.fab.activated",
                                last_action.clone(),
                            ))
                            .into_element(cx),
                        material3::Fab::new(ids::ui::SEARCH)
                            .variant(variant)
                            .a11y_label("Small")
                            .size(material3::FabSize::Small)
                            .on_activate(on_activate(
                                "material3.fab.small.activated",
                                last_action.clone(),
                            ))
                            .into_element(cx),
                        material3::Fab::new(ids::ui::SEARCH)
                            .variant(variant)
                            .a11y_label("Large")
                            .size(material3::FabSize::Large)
                            .on_activate(on_activate(
                                "material3.fab.large.activated",
                                last_action.clone(),
                            ))
                            .into_element(cx),
                        material3::Fab::new(ids::ui::SEARCH)
                            .variant(variant)
                            .a11y_label("Lowered")
                            .lowered(true)
                            .on_activate(on_activate(
                                "material3.fab.lowered.activated",
                                last_action.clone(),
                            ))
                            .into_element(cx),
                        material3::Fab::new(ids::ui::SEARCH)
                            .variant(variant)
                            .a11y_label("Disabled")
                            .disabled(true)
                            .into_element(cx),
                    ]
                },
            )
        }
    };

    let extended = {
        let last_action = last_action.clone();
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::Fab::new(ids::ui::SEARCH)
                        .variant(material3::FabVariant::Surface)
                        .label("Create")
                        .on_activate(on_activate(
                            "material3.extended_fab.activated",
                            last_action.clone(),
                        ))
                        .into_element(cx),
                    material3::Fab::new(ids::ui::SEARCH)
                        .variant(material3::FabVariant::Primary)
                        .label("Create")
                        .on_activate(on_activate(
                            "material3.extended_fab.primary.activated",
                            last_action.clone(),
                        ))
                        .into_element(cx),
                    material3::Fab::new(ids::ui::SEARCH)
                        .variant(material3::FabVariant::Surface)
                        .label("Reroute")
                        .icon(None)
                        .on_activate(on_activate(
                            "material3.extended_fab.no_icon.activated",
                            last_action.clone(),
                        ))
                        .into_element(cx),
                ]
            },
        )
    };

    vec![
        cx.text(
            "Material 3 FAB: token-driven variants + focus ring + state layer + bounded ripple.",
        ),
        row(cx, material3::FabVariant::Surface, "Surface"),
        row(cx, material3::FabVariant::Primary, "Primary"),
        row(cx, material3::FabVariant::Secondary, "Secondary"),
        row(cx, material3::FabVariant::Tertiary, "Tertiary"),
        extended,
    ]
}

pub(in crate::ui) fn preview_material3_checkbox(
    cx: &mut ElementContext<'_, App>,
    checked: Model<bool>,
) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let value = cx
        .get_model_copied(&checked, Invalidation::Layout)
        .unwrap_or(false);

    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            let override_style = material3::CheckboxStyle::default()
                .icon_color(WidgetStateProperty::new(None).when(
                    WidgetStates::SELECTED,
                    Some(ColorRef::Color(fret_core::Color {
                        r: 0.2,
                        g: 0.8,
                        b: 0.4,
                        a: 1.0,
                    })),
                ))
                .outline_color(WidgetStateProperty::new(None).when(
                    WidgetStates::SELECTED,
                    Some(ColorRef::Color(fret_core::Color {
                        r: 0.2,
                        g: 0.8,
                        b: 0.4,
                        a: 1.0,
                    })),
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
        },
    );

    vec![
        cx.text("Material 3 Checkbox: token-driven sizing/colors + state layer + bounded ripple."),
        row,
    ]
}

pub(in crate::ui) fn preview_material3_switch(
    cx: &mut ElementContext<'_, App>,
    selected: Model<bool>,
) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let value = cx
        .get_model_copied(&selected, Invalidation::Layout)
        .unwrap_or(false);

    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            let override_style = material3::SwitchStyle::default()
                .track_color(WidgetStateProperty::new(None).when(
                    WidgetStates::SELECTED,
                    Some(ColorRef::Color(fret_core::Color {
                        r: 0.2,
                        g: 0.8,
                        b: 0.4,
                        a: 1.0,
                    })),
                ))
                .state_layer_color(WidgetStateProperty::new(None).when(
                    WidgetStates::HOVERED,
                    Some(ColorRef::Color(fret_core::Color {
                        r: 0.9,
                        g: 0.2,
                        b: 0.9,
                        a: 1.0,
                    })),
                ));
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

    vec![
        cx.text("Material 3 Switch: token-driven sizing/colors + state layer + bounded ripple."),
        row,
    ]
}

pub(in crate::ui) fn preview_material3_radio(
    cx: &mut ElementContext<'_, App>,
    group_value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    #[derive(Default)]
    struct RadioPageModels {
        standalone_selected: Option<Model<bool>>,
    }

    let standalone_selected = cx.with_state(RadioPageModels::default, |st| {
        st.standalone_selected.clone()
    });
    let standalone_selected = match standalone_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(RadioPageModels::default, |st| {
                st.standalone_selected = Some(model.clone())
            });
            model
        }
    };

    let current = cx
        .get_model_cloned(&group_value, Invalidation::Layout)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let group_value_for_row = group_value.clone();
    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                material3::RadioGroup::new(group_value_for_row.clone())
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
        },
    );

    let override_style = material3::RadioStyle::default()
        .icon_color(WidgetStateProperty::new(None).when(
            WidgetStates::SELECTED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.2,
                g: 0.8,
                b: 0.4,
                a: 1.0,
            })),
        ))
        .state_layer_color(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.9,
                g: 0.2,
                b: 0.9,
                a: 1.0,
            })),
        ));

    let group_value_for_group_overridden = group_value.clone();
    let override_style_for_group = override_style.clone();
    let override_style_for_standalone = override_style.clone();
    let group_overridden = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                material3::RadioGroup::new(group_value_for_group_overridden.clone())
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
        },
    );
    let standalone = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                material3::Radio::new(standalone_selected.clone())
                    .a11y_label("Material 3 Radio (standalone)")
                    .test_id("ui-gallery-material3-radio-standalone")
                    .into_element(cx),
                material3::Radio::new(standalone_selected.clone())
                    .a11y_label("Material 3 Radio (override)")
                    .style(override_style_for_standalone.clone())
                    .test_id("ui-gallery-material3-radio-standalone-overridden")
                    .into_element(cx),
            ]
        },
    );

    vec![
        cx.text(
            "Material 3 Radio: group-value binding + roving focus + typeahead + state layer + bounded ripple.",
        ),
        row,
        cx.text("Override preview: RadioGroup::style(...) using RadioStyle."),
        group_overridden,
        cx.text("Override preview: standalone Radio::style(...) using RadioStyle."),
        standalone,
    ]
}
