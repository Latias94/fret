use super::super::super::super::*;

pub(in crate::ui) fn preview_material3_chip(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;
    use fret_ui::action::OnActivate;
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    #[derive(Default)]
    struct ChipPageModels {
        filter_selected: Option<Model<bool>>,
        filter_unselected: Option<Model<bool>>,
        input_selected: Option<Model<bool>>,
        input_unselected: Option<Model<bool>>,
    }

    let filter_selected = cx.with_state(ChipPageModels::default, |st| st.filter_selected.clone());
    let filter_selected = match filter_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(ChipPageModels::default, |st| {
                st.filter_selected = Some(model.clone())
            });
            model
        }
    };

    let filter_unselected =
        cx.with_state(ChipPageModels::default, |st| st.filter_unselected.clone());
    let filter_unselected = match filter_unselected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ChipPageModels::default, |st| {
                st.filter_unselected = Some(model.clone())
            });
            model
        }
    };

    let input_selected = cx.with_state(ChipPageModels::default, |st| st.input_selected.clone());
    let input_selected = match input_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(ChipPageModels::default, |st| {
                st.input_selected = Some(model.clone())
            });
            model
        }
    };

    let input_unselected = cx.with_state(ChipPageModels::default, |st| st.input_unselected.clone());
    let input_unselected = match input_unselected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ChipPageModels::default, |st| {
                st.input_unselected = Some(model.clone())
            });
            model
        }
    };

    let last_action_for_activate = last_action.clone();
    let activate: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host.models_mut().update(&last_action_for_activate, |v| {
            *v = Arc::<str>::from("material3.assist_chip.activated");
        });
    });

    let (hover_container, hover_label) = cx.with_theme(|theme| {
        (
            theme.color_required("md.sys.color.tertiary-container"),
            theme.color_required("md.sys.color.on-tertiary-container"),
        )
    });
    let accent = fret_core::Color {
        r: 0.9,
        g: 0.2,
        b: 0.9,
        a: 1.0,
    };

    let override_style = material3::AssistChipStyle::default()
        .label_color(WidgetStateProperty::new(Some(ColorRef::Color(accent))))
        .state_layer_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(accent))),
        )
        .outline_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(accent))),
        )
        .container_background(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(hover_container)),
        ));

    let hover_style = material3::AssistChipStyle::default()
        .label_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_label))),
        )
        .container_background(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(hover_container)),
        ));

    let filter_override_style = material3::FilterChipStyle::default()
        .container_background(WidgetStateProperty::new(None).when(
            WidgetStates::SELECTED,
            Some(ColorRef::Color(hover_container)),
        ))
        .outline_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(accent))),
        );

    let activate_row1 = activate.clone();
    let activate_row2 = activate.clone();
    let activate_row3 = activate.clone();
    let activate_row4 = activate.clone();

    let last_action_for_input_selected = last_action.clone();
    let activate_input_selected_primary: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_input_selected, |v| {
                *v = Arc::<str>::from("material3.input_chip.primary.activated");
            });
    });

    let last_action_for_input_unselected = last_action.clone();
    let activate_input_unselected_primary: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_input_unselected, |v| {
                *v = Arc::<str>::from("material3.input_chip.primary.activated");
            });
    });

    let last_action_for_input_unselected_trailing = last_action.clone();
    let activate_input_unselected_trailing: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_input_unselected_trailing, |v| {
                *v = Arc::<str>::from("material3.input_chip.trailing_icon.activated");
            });
    });

    let override_style_row1 = override_style.clone();
    let override_style_row2 = override_style.clone();
    let hover_style_row1 = hover_style.clone();
    let hover_style_row2 = hover_style.clone();
    let filter_override_style_row = filter_override_style.clone();

    let last_action_for_filter_primary = last_action.clone();
    let activate_filter_primary: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_filter_primary, |v| {
                *v = Arc::<str>::from("material3.filter_chip.primary.activated");
            });
    });

    let last_action_for_filter_trailing = last_action.clone();
    let activate_filter_trailing: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_filter_trailing, |v| {
                *v = Arc::<str>::from("material3.filter_chip.trailing_icon.activated");
            });
    });

    let filter_selected_row1 = filter_selected.clone();
    let filter_unselected_row1 = filter_unselected.clone();
    let filter_selected_row2 = filter_selected.clone();
    let filter_unselected_row2 = filter_unselected.clone();
    let input_selected_row1 = input_selected.clone();
    let input_unselected_row1 = input_unselected.clone();
    let input_unselected_row2 = input_unselected.clone();
    let activate_filter_primary_row = activate_filter_primary.clone();
    let activate_filter_trailing_row = activate_filter_trailing.clone();
    let activate_filter_primary_for_set = activate_filter_primary.clone();
    let activate_filter_trailing_for_set = activate_filter_trailing.clone();
    let activate_input_unselected_primary_row = activate_input_unselected_primary.clone();
    let activate_input_unselected_trailing_row = activate_input_unselected_trailing.clone();
    let activate_input_unselected_primary_for_set = activate_input_unselected_primary.clone();
    let activate_input_unselected_trailing_for_set = activate_input_unselected_trailing.clone();

    vec![
        cx.text("Material 3 AssistChip: token-driven shape/colors + state layer + bounded ripple."),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::AssistChip::new("Flat")
                        .on_activate(activate_row1.clone())
                        .test_id("ui-gallery-material3-chip-flat")
                        .into_element(cx),
                    material3::AssistChip::new("Override")
                        .on_activate(activate_row1.clone())
                        .style(override_style_row1.clone())
                        .test_id("ui-gallery-material3-chip-flat-override")
                        .into_element(cx),
                    material3::AssistChip::new("Disabled")
                        .disabled(true)
                        .test_id("ui-gallery-material3-chip-flat-disabled")
                        .into_element(cx),
                    material3::AssistChip::new("Hover Override")
                        .on_activate(activate_row1.clone())
                        .style(hover_style_row1.clone())
                        .test_id("ui-gallery-material3-chip-flat-hover-override")
                        .into_element(cx),
                ]
            },
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::AssistChip::new("Elevated")
                        .variant(material3::AssistChipVariant::Elevated)
                        .leading_icon(ids::ui::SETTINGS)
                        .on_activate(activate_row2.clone())
                        .test_id("ui-gallery-material3-chip-elevated")
                        .into_element(cx),
                    material3::AssistChip::new("Override")
                        .variant(material3::AssistChipVariant::Elevated)
                        .leading_icon(ids::ui::SETTINGS)
                        .on_activate(activate_row2.clone())
                        .style(override_style_row2.clone())
                        .test_id("ui-gallery-material3-chip-elevated-override")
                        .into_element(cx),
                    material3::AssistChip::new("Disabled")
                        .variant(material3::AssistChipVariant::Elevated)
                        .leading_icon(ids::ui::SLASH)
                        .disabled(true)
                        .test_id("ui-gallery-material3-chip-elevated-disabled")
                        .into_element(cx),
                    material3::AssistChip::new("Hover Override")
                        .variant(material3::AssistChipVariant::Elevated)
                        .leading_icon(ids::ui::SETTINGS)
                        .on_activate(activate_row2.clone())
                        .style(hover_style_row2.clone())
                        .test_id("ui-gallery-material3-chip-elevated-hover-override")
                        .into_element(cx),
                ]
            },
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::SuggestionChip::new("Suggestion")
                        .on_activate(activate_row3.clone())
                        .test_id("ui-gallery-material3-suggestion-chip-flat")
                        .into_element(cx),
                    material3::SuggestionChip::new("Suggestion (icon)")
                        .leading_icon(ids::ui::SEARCH)
                        .variant(material3::SuggestionChipVariant::Elevated)
                        .on_activate(activate_row3.clone())
                        .test_id("ui-gallery-material3-suggestion-chip-elevated")
                        .into_element(cx),
                    material3::SuggestionChip::new("Disabled")
                        .disabled(true)
                        .test_id("ui-gallery-material3-suggestion-chip-disabled")
                        .into_element(cx),
                ]
            },
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::FilterChip::new(filter_selected_row1.clone(), "Filter")
                        .trailing_icon(ids::ui::CLOSE)
                        .on_activate(activate_filter_primary_row.clone())
                        .on_trailing_icon_activate(activate_filter_trailing_row.clone())
                        .test_id("ui-gallery-material3-filter-chip-selected")
                        .into_element(cx),
                    material3::FilterChip::new(filter_unselected_row1.clone(), "Filter")
                        .on_activate(activate_filter_primary_row.clone())
                        .test_id("ui-gallery-material3-filter-chip-unselected")
                        .into_element(cx),
                    material3::FilterChip::new(filter_selected_row2.clone(), "Override")
                        .variant(material3::FilterChipVariant::Elevated)
                        .style(filter_override_style_row.clone())
                        .on_activate(activate_filter_primary_row.clone())
                        .test_id("ui-gallery-material3-filter-chip-override")
                        .into_element(cx),
                    material3::FilterChip::new(filter_unselected_row2.clone(), "Disabled")
                        .disabled(true)
                        .test_id("ui-gallery-material3-filter-chip-disabled")
                        .into_element(cx),
                ]
            },
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::InputChip::new(input_selected_row1.clone(), "Input")
                        .leading_icon(ids::ui::SETTINGS)
                        .on_activate(activate_input_selected_primary.clone())
                        .test_id("ui-gallery-material3-input-chip-selected")
                        .into_element(cx),
                    material3::InputChip::new(input_unselected_row1.clone(), "Input")
                        .trailing_icon(ids::ui::CLOSE)
                        .on_activate(activate_input_unselected_primary_row.clone())
                        .on_trailing_icon_activate(activate_input_unselected_trailing_row.clone())
                        .test_id("ui-gallery-material3-input-chip-unselected")
                        .into_element(cx),
                    material3::InputChip::new(input_unselected_row2.clone(), "Disabled")
                        .disabled(true)
                        .test_id("ui-gallery-material3-input-chip-disabled")
                        .into_element(cx),
                ]
            },
        ),
        cx.text(
            "Material 3 ChipSet: roving focus (ArrowLeft/Right + Home/End). Multi-action chips use ArrowLeft/Right to move focus between primary/trailing actions, then roving continues to the next chip.",
        ),
        material3::ChipSet::new(vec![
            material3::ChipSetItem::from(
                material3::AssistChip::new("Assist")
                    .leading_icon(ids::ui::SETTINGS)
                    .on_activate(activate_row4.clone())
                    .test_id("ui-gallery-material3-chip-set-assist"),
            ),
            material3::ChipSetItem::from(
                material3::SuggestionChip::new("Suggestion")
                    .leading_icon(ids::ui::SEARCH)
                    .on_activate(activate_row4.clone())
                    .test_id("ui-gallery-material3-chip-set-suggestion"),
            ),
            material3::ChipSetItem::from(
                material3::FilterChip::new(filter_selected.clone(), "Filter")
                    .trailing_icon(ids::ui::CLOSE)
                    .on_activate(activate_filter_primary_for_set.clone())
                    .on_trailing_icon_activate(activate_filter_trailing_for_set.clone())
                    .test_id("ui-gallery-material3-chip-set-filter"),
            ),
            material3::ChipSetItem::from(
                material3::InputChip::new(input_unselected.clone(), "Input")
                    .trailing_icon(ids::ui::CLOSE)
                    .on_activate(activate_input_unselected_primary_for_set.clone())
                    .on_trailing_icon_activate(activate_input_unselected_trailing_for_set.clone())
                    .test_id("ui-gallery-material3-chip-set-input"),
            ),
        ])
        .a11y_label("chip set")
        .test_id("ui-gallery-material3-chip-set")
        .into_element(cx),
    ]
}
