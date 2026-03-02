use super::super::super::*;
use super::super::super::doc_layout::DocSection;

pub(in crate::ui) fn preview_material3_icon_button(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::icon_button::render(cx);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![DocSection::new("Demo", demo)
            .code_rust_from_file_region(snippets::material3::icon_button::SOURCE, "example")],
    );

    vec![page]
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
    let demo = snippets::material3::checkbox::render(cx, checked);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![DocSection::new("Demo", demo)
            .code_rust_from_file_region(snippets::material3::checkbox::SOURCE, "example")],
    );

    vec![page]
}

pub(in crate::ui) fn preview_material3_switch(
    cx: &mut ElementContext<'_, App>,
    selected: Model<bool>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::switch::render(cx, selected);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![DocSection::new("Demo", demo)
            .code_rust_from_file_region(snippets::material3::switch::SOURCE, "example")],
    );

    vec![page]
}

pub(in crate::ui) fn preview_material3_slider(
    cx: &mut ElementContext<'_, App>,
    value: Model<f32>,
) -> Vec<AnyElement> {
    let value_now = cx
        .get_model_copied(&value, Invalidation::Layout)
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let value_for_main_row = value.clone();
    let value_for_ticks_row = value.clone();

    vec![
        cx.text("Material 3 Slider: token-driven track + handle + state layer."),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            move |cx| {
                vec![
                    material3::Slider::new(value_for_main_row.clone())
                        .a11y_label("Material 3 Slider")
                        .test_id("ui-gallery-material3-slider")
                        .into_element(cx),
                    cx.text(format!("value={value_now:.3}"))
                        .test_id("ui-gallery-material3-slider-value"),
                    material3::Slider::new(value_for_main_row.clone())
                        .disabled(true)
                        .a11y_label("Disabled Material 3 Slider")
                        .test_id("ui-gallery-material3-slider-disabled")
                        .into_element(cx),
                ]
            },
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            move |cx| {
                vec![
                    material3::Slider::new(value_for_ticks_row.clone())
                        .with_tick_marks(true)
                        .tick_marks_count(5)
                        .a11y_label("Material 3 Slider (tick marks)")
                        .test_id("ui-gallery-material3-slider-tick-marks")
                        .into_element(cx),
                    cx.text("tick_marks=5"),
                ]
            },
        ),
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
