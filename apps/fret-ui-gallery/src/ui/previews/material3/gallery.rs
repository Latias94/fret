use super::super::super::*;
use super::*;

pub(in crate::ui) fn preview_material3_gallery(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_list_value: Model<Arc<str>>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let disabled = cx
        .get_model_copied(&material3_text_field_disabled, Invalidation::Layout)
        .unwrap_or(false);
    let error = cx
        .get_model_copied(&material3_text_field_error, Invalidation::Layout)
        .unwrap_or(false);

    let mut out: Vec<AnyElement> = Vec::new();
    out.push(cx.text("Material 3 Gallery: compact outcomes-first surface."));

    out.push(cx.text("— Buttons —"));
    out.push(stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                material3::Button::new("Filled")
                    .variant(material3::ButtonVariant::Filled)
                    .into_element(cx),
                material3::Button::new("Tonal")
                    .variant(material3::ButtonVariant::Tonal)
                    .into_element(cx),
                material3::Button::new("Outlined")
                    .variant(material3::ButtonVariant::Outlined)
                    .into_element(cx),
                material3::Button::new("Text")
                    .variant(material3::ButtonVariant::Text)
                    .into_element(cx),
            ]
        },
    ));

    out.push(cx.text("— Icon Buttons —"));
    out.push(stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            let (hover_icon, hover_container) = cx.with_theme(|theme| {
                (
                    fret_ui_shadcn::ColorRef::Color(
                        theme.color_required("md.sys.color.on-tertiary-container"),
                    ),
                    fret_ui_shadcn::ColorRef::Color(
                        theme.color_required("md.sys.color.tertiary-container"),
                    ),
                )
            });
            let hover_style = material3::IconButtonStyle::default()
                .container_background(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_container)),
                )
                .icon_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_icon)),
                );

            vec![
                material3::IconButton::new(ids::ui::SEARCH)
                    .a11y_label("Search")
                    .into_element(cx),
                material3::IconButton::new(ids::ui::SETTINGS)
                    .a11y_label("Settings")
                    .into_element(cx),
                material3::IconButton::new(ids::ui::CLOSE)
                    .a11y_label("Close")
                    .into_element(cx),
                material3::IconButton::new(ids::ui::SEARCH)
                    .a11y_label("Override")
                    .style(hover_style)
                    .into_element(cx),
            ]
        },
    ));

    out.push(cx.text("— FAB —"));
    out.push(stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                material3::Fab::new(ids::ui::SEARCH)
                    .a11y_label("Search")
                    .into_element(cx),
                material3::Fab::new(ids::ui::SEARCH)
                    .a11y_label("Search (small)")
                    .size(material3::FabSize::Small)
                    .into_element(cx),
                material3::Fab::new(ids::ui::SEARCH)
                    .a11y_label("Search (large)")
                    .size(material3::FabSize::Large)
                    .into_element(cx),
                material3::Fab::new(ids::ui::SEARCH)
                    .a11y_label("Search (primary)")
                    .variant(material3::FabVariant::Primary)
                    .into_element(cx),
            ]
        },
    ));

    out.push(cx.text("— Selection —"));
    out.push(stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N3).items_center(),
        |cx| {
            let (hover_container, hover_icon, hover_outline) = cx.with_theme(|theme| {
                (
                    fret_ui_shadcn::ColorRef::Color(
                        theme.color_required("md.sys.color.tertiary-container"),
                    ),
                    fret_ui_shadcn::ColorRef::Color(
                        theme.color_required("md.sys.color.on-tertiary-container"),
                    ),
                    fret_ui_shadcn::ColorRef::Color(theme.color_required("md.sys.color.tertiary")),
                )
            });
            let hover_style = material3::CheckboxStyle::default()
                .container_background(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_container)),
                )
                .icon_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_icon)),
                )
                .outline_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_outline)),
                );

            vec![
                material3::Checkbox::new(material3_checkbox.clone())
                    .a11y_label("Checkbox")
                    .into_element(cx),
                material3::Checkbox::new(material3_checkbox.clone())
                    .a11y_label("Checkbox Override")
                    .style(hover_style)
                    .into_element(cx),
                material3::Switch::new(material3_switch.clone())
                    .a11y_label("Switch")
                    .into_element(cx),
                material3::Switch::new(material3_switch.clone())
                    .a11y_label("Switch Override")
                    .style({
                        let (hover_track, hover_handle) = cx.with_theme(|theme| {
                            (
                                fret_ui_shadcn::ColorRef::Color(
                                    theme.color_required("md.sys.color.tertiary-container"),
                                ),
                                fret_ui_shadcn::ColorRef::Color(
                                    theme.color_required("md.sys.color.on-tertiary-container"),
                                ),
                            )
                        });
                        material3::SwitchStyle::default()
                            .track_color(
                                fret_ui_kit::WidgetStateProperty::new(None)
                                    .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_track)),
                            )
                            .handle_color(
                                fret_ui_kit::WidgetStateProperty::new(None)
                                    .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_handle)),
                            )
                    })
                    .into_element(cx),
                stack::vstack(
                    cx,
                    stack::VStackProps::default().gap(Space::N1).items_start(),
                    |cx| {
                        let items = vec![
                            material3::RadioGroupItem::new("Alpha").a11y_label("Radio Alpha"),
                            material3::RadioGroupItem::new("Beta").a11y_label("Radio Beta"),
                            material3::RadioGroupItem::new("Charlie")
                                .a11y_label("Radio Charlie")
                                .disabled(true),
                        ];

                        let hover_color = cx.with_theme(|theme| {
                            fret_ui_shadcn::ColorRef::Color(
                                theme.color_required("md.sys.color.tertiary"),
                            )
                        });
                        let hover_style = material3::RadioStyle::default()
                            .icon_color(fret_ui_kit::WidgetStateProperty::new(None).when(
                                fret_ui_kit::WidgetStates::HOVERED,
                                Some(hover_color.clone()),
                            ))
                            .state_layer_color(
                                fret_ui_kit::WidgetStateProperty::new(None)
                                    .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_color)),
                            );

                        vec![
                            cx.text("Radio Group"),
                            material3::RadioGroup::new(material3_radio_value.clone())
                                .a11y_label("Radio Group")
                                .items(items.clone())
                                .into_element(cx),
                            cx.text("Radio Group Override"),
                            material3::RadioGroup::new(material3_radio_value.clone())
                                .a11y_label("Radio Group Override")
                                .style(hover_style)
                                .items(items)
                                .into_element(cx),
                        ]
                    },
                ),
            ]
        },
    ));

    out.push(cx.text("— Text Field —"));
    out.push(stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Switch::new(material3_text_field_disabled.clone())
                    .a11y_label("Disable Text Field")
                    .into_element(cx),
                cx.text("Disabled"),
                shadcn::Switch::new(material3_text_field_error.clone())
                    .a11y_label("Text Field Error")
                    .into_element(cx),
                cx.text("Error"),
            ]
        },
    ));
    out.push(stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N1).items_start(),
        |cx| {
            let hover = cx.with_theme(|theme| {
                fret_ui_shadcn::ColorRef::Color(theme.color_required("md.sys.color.tertiary"))
            });
            let hover_style = material3::TextFieldStyle::default()
                .outline_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover.clone())),
                )
                .label_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover)),
                );

            vec![
                cx.text("Text Field"),
                material3::TextField::new(material3_text_field_value.clone())
                    .label("Label")
                    .placeholder("Placeholder")
                    .disabled(disabled)
                    .error(error)
                    .into_element(cx),
                cx.text("Text Field Override"),
                material3::TextField::new(material3_text_field_value)
                    .label("Label")
                    .placeholder("Placeholder")
                    .style(hover_style)
                    .disabled(disabled)
                    .error(error)
                    .into_element(cx),
            ]
        },
    ));

    out.push(cx.text("— Tabs —"));
    out.push(stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N1).items_start(),
        |cx| {
            let items = vec![
                material3::TabItem::new("overview", "Overview"),
                material3::TabItem::new("security", "Security"),
                material3::TabItem::new("settings", "Settings"),
            ];

            let hover_color = cx.with_theme(|theme| {
                fret_ui_shadcn::ColorRef::Color(theme.color_required("md.sys.color.tertiary"))
            });
            let hover_style = material3::TabsStyle::default()
                .label_color(fret_ui_kit::WidgetStateProperty::new(None).when(
                    fret_ui_kit::WidgetStates::HOVERED,
                    Some(hover_color.clone()),
                ))
                .state_layer_color(fret_ui_kit::WidgetStateProperty::new(None).when(
                    fret_ui_kit::WidgetStates::HOVERED,
                    Some(hover_color.clone()),
                ))
                .active_indicator_color(fret_ui_kit::WidgetStateProperty::new(Some(hover_color)));

            vec![
                cx.text("Tabs"),
                material3::Tabs::new(material3_tabs_value.clone())
                    .a11y_label("Tabs")
                    .items(items.clone())
                    .into_element(cx),
                cx.text("Tabs Override"),
                material3::Tabs::new(material3_tabs_value)
                    .a11y_label("Tabs Override")
                    .style(hover_style)
                    .items(items)
                    .into_element(cx),
            ]
        },
    ));

    out.push(cx.text("— Navigation Bar —"));
    out.push(
        material3::NavigationBar::new(material3_navigation_bar_value)
            .a11y_label("Navigation bar")
            .items(vec![
                material3::NavigationBarItem::new("search", "Search", ids::ui::SEARCH),
                material3::NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS),
                material3::NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL),
            ])
            .into_element(cx),
    );

    out.push(cx.text("— List —"));
    out.push(
        material3::List::new(material3_list_value)
            .a11y_label("List")
            .items(vec![
                material3::ListItem::new("alpha", "Alpha").leading_icon(ids::ui::SEARCH),
                material3::ListItem::new("beta", "Beta").leading_icon(ids::ui::SETTINGS),
                material3::ListItem::new("disabled", "Disabled")
                    .leading_icon(ids::ui::SLASH)
                    .disabled(true),
            ])
            .into_element(cx),
    );

    out
}

pub(in crate::ui) fn preview_material3_state_matrix(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    material3_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let mut out: Vec<AnyElement> = Vec::new();

    out.push(cx.text(
        "Material 3 State Matrix: exercise hover/focus/press/disabled/selected states across multiple components.",
    ));
    out.push(cx.text(
        "Tip: use keyboard Tab/Arrow/Home/End on Tabs/Radio/Menu; use Esc/outside press to close Menu.",
    ));

    out.extend(material3_state_matrix_content(
        cx,
        material3_checkbox,
        material3_switch,
        material3_radio_value,
        material3_tabs_value,
        material3_navigation_bar_value,
        material3_text_field_value,
        material3_text_field_disabled,
        material3_text_field_error,
        material3_menu_open,
        last_action,
    ));
    out
}

pub(in crate::ui) fn material3_state_matrix_content(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    material3_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let mut out: Vec<AnyElement> = Vec::new();

    out.push(cx.text("— Buttons —"));
    out.extend(preview_material3_button(cx));

    out.push(cx.text("— Chips —"));
    out.extend(preview_material3_chip(cx, last_action.clone()));

    out.push(cx.text("— Cards —"));
    out.extend(preview_material3_card(cx, last_action.clone()));

    out.push(cx.text("— Icon Buttons —"));
    out.extend(preview_material3_icon_button(cx));

    out.push(cx.text("— FAB —"));
    out.extend(preview_material3_fab(cx, last_action.clone()));

    out.push(cx.text("— Checkbox —"));
    out.extend(preview_material3_checkbox(cx, material3_checkbox));

    out.push(cx.text("— Switch —"));
    out.extend(preview_material3_switch(cx, material3_switch));

    out.push(cx.text("— Radio —"));
    out.extend(preview_material3_radio(cx, material3_radio_value));

    out.push(cx.text("— Text Field —"));
    out.extend(preview_material3_text_field(
        cx,
        material3_text_field_value,
        material3_text_field_disabled,
        material3_text_field_error,
    ));

    out.push(cx.text("— Search View —"));
    out.extend(preview_material3_search_view(cx));

    out.push(cx.text("— Tabs —"));
    out.extend(preview_material3_tabs(cx, material3_tabs_value));

    out.push(cx.text("— Navigation Bar —"));
    out.extend(preview_material3_navigation_bar(
        cx,
        material3_navigation_bar_value,
    ));

    out.push(cx.text("— Menu —"));
    out.extend(preview_material3_menu(cx, material3_menu_open, last_action));

    out
}

pub(in crate::ui) fn preview_material3_search_view(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    use fret_icons::ids::ui as ui_icons;

    #[derive(Default)]
    struct SearchViewPageModels {
        open: Option<Model<bool>>,
        query: Option<Model<String>>,
        selected: Option<Model<Arc<str>>>,
    }

    let open = cx.with_state(SearchViewPageModels::default, |st| st.open.clone());
    let open = match open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SearchViewPageModels::default, |st| {
                st.open = Some(model.clone())
            });
            model
        }
    };

    let query = cx.with_state(SearchViewPageModels::default, |st| st.query.clone());
    let query = match query {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(SearchViewPageModels::default, |st| {
                st.query = Some(model.clone())
            });
            model
        }
    };

    let selected = cx.with_state(SearchViewPageModels::default, |st| st.selected.clone());
    let selected = match selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("alpha"));
            cx.with_state(SearchViewPageModels::default, |st| {
                st.selected = Some(model.clone())
            });
            model
        }
    };

    let suggestions = material3::List::new(selected)
        .a11y_label("Suggestions")
        .test_id("ui-gallery-material3-search-view-suggestions")
        .items(vec![
            material3::ListItem::new("alpha", "Alpha")
                .leading_icon(ui_icons::SEARCH)
                .test_id("ui-gallery-material3-search-view-option-alpha"),
            material3::ListItem::new("bravo", "Bravo")
                .leading_icon(ui_icons::SEARCH)
                .test_id("ui-gallery-material3-search-view-option-bravo"),
            material3::ListItem::new("charlie", "Charlie")
                .leading_icon(ui_icons::SEARCH)
                .test_id("ui-gallery-material3-search-view-option-charlie"),
        ])
        .into_element(cx);

    let view = material3::SearchView::new(open, query)
        .leading_icon(ui_icons::SEARCH)
        .trailing_icon(ui_icons::CLOSE)
        .placeholder("Search")
        .a11y_label("Search")
        .test_id("ui-gallery-material3-search-view")
        .overlay_test_id("ui-gallery-material3-search-view-panel")
        .into_element(cx, |_cx| vec![suggestions]);

    vec![view]
}

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

pub(in crate::ui) fn preview_material3_card(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui::element::{ContainerProps, Length, TextProps};
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let activate: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host.models_mut().update(&last_action, |v| {
            *v = Arc::<str>::from("material3.card.activated");
        });
    });

    let (body_style, body_color, hover_container, hover_outline) = cx.with_theme(|theme| {
        let body_style = theme
            .text_style_by_key("md.sys.typescale.body-medium")
            .unwrap_or_else(|| fret_core::TextStyle::default());
        let body_color = theme.color_required("md.sys.color.on-surface");
        let hover_container = theme.color_required("md.sys.color.tertiary-container");
        let hover_outline = theme.color_required("md.sys.color.tertiary");
        (body_style, body_color, hover_container, hover_outline)
    });

    let override_style = material3::CardStyle::default()
        .container_background(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(hover_container)),
        ))
        .outline_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_outline))),
        );

    let activate_row1 = activate.clone();
    let activate_row2 = activate.clone();
    let override_style_row1 = override_style.clone();
    let override_style_row2 = override_style.clone();

    let card_content_row1 = {
        let body_style = body_style.clone();
        let body_color = body_color;
        move |cx: &mut ElementContext<'_, App>, label: &'static str| {
            let mut container = ContainerProps::default();
            container.layout.size.width = Length::Px(Px(280.0));
            container.layout.size.height = Length::Px(Px(72.0));
            container.padding = Edges::all(Px(12.0));

            let mut text = TextProps::new(Arc::<str>::from(label));
            text.style = Some(body_style.clone());
            text.color = Some(body_color);
            cx.container(container, move |cx| vec![cx.text_props(text)])
        }
    };

    let card_content_row2 = {
        let body_style = body_style.clone();
        let body_color = body_color;
        move |cx: &mut ElementContext<'_, App>, label: &'static str| {
            let mut container = ContainerProps::default();
            container.layout.size.width = Length::Px(Px(280.0));
            container.layout.size.height = Length::Px(Px(72.0));
            container.padding = Edges::all(Px(12.0));

            let mut text = TextProps::new(Arc::<str>::from(label));
            text.style = Some(body_style.clone());
            text.color = Some(body_color);
            cx.container(container, move |cx| vec![cx.text_props(text)])
        }
    };

    vec![
        cx.text("Material 3 Card: token-driven surface + outline + ink (interactive only when on_activate is set)."),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::Card::new()
                        .variant(material3::CardVariant::Filled)
                        .on_activate(activate_row1.clone())
                        .test_id("ui-gallery-material3-card-filled")
                        .into_element(cx, |cx| vec![card_content_row1(cx, "Filled")]),
                    material3::Card::new()
                        .variant(material3::CardVariant::Filled)
                        .on_activate(activate_row1.clone())
                        .style(override_style_row1.clone())
                        .test_id("ui-gallery-material3-card-filled-override")
                        .into_element(cx, |cx| vec![card_content_row1(cx, "Override")]),
                    material3::Card::new()
                        .variant(material3::CardVariant::Filled)
                        .on_activate(activate_row1.clone())
                        .disabled(true)
                        .test_id("ui-gallery-material3-card-filled-disabled")
                        .into_element(cx, |cx| vec![card_content_row1(cx, "Disabled")]),
                ]
            },
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::Card::new()
                        .variant(material3::CardVariant::Elevated)
                        .on_activate(activate_row2.clone())
                        .test_id("ui-gallery-material3-card-elevated")
                        .into_element(cx, |cx| vec![card_content_row2(cx, "Elevated")]),
                    material3::Card::new()
                        .variant(material3::CardVariant::Outlined)
                        .on_activate(activate_row2.clone())
                        .test_id("ui-gallery-material3-card-outlined")
                        .into_element(cx, |cx| vec![card_content_row2(cx, "Outlined")]),
                    material3::Card::new()
                        .variant(material3::CardVariant::Outlined)
                        .on_activate(activate_row2.clone())
                        .style(override_style_row2.clone())
                        .test_id("ui-gallery-material3-card-outlined-override")
                        .into_element(cx, |cx| vec![card_content_row2(cx, "Outline override")]),
                ]
            },
        ),
    ]
}

pub(in crate::ui) fn preview_material3_touch_targets(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let min = cx.with_theme(|theme| {
        theme
            .metric_by_key("md.sys.layout.minimum-touch-target.size")
            .unwrap_or(Px(48.0))
    });

    let target_overlay = |cx: &mut ElementContext<'_, App>,
                          label: &'static str,
                          chrome: Option<Size>,
                          child: AnyElement| {
        let min = min;

        let stack = cx.stack_props(
            StackProps {
                layout: {
                    let mut l = fret_ui::element::LayoutStyle::default();
                    l.overflow = fret_ui::element::Overflow::Visible;
                    l
                },
            },
            move |cx| {
                let mut canvas = CanvasProps::default();
                canvas.layout.position = fret_ui::element::PositionStyle::Absolute;
                canvas.layout.inset.top = Some(Px(0.0));
                canvas.layout.inset.right = Some(Px(0.0));
                canvas.layout.inset.bottom = Some(Px(0.0));
                canvas.layout.inset.left = Some(Px(0.0));

                let overlay = cx.canvas(canvas, move |p| {
                    let bounds = p.bounds();
                    let center = Point::new(
                        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
                        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
                    );

                    let min_rect = Rect::new(
                        Point::new(Px(center.x.0 - min.0 * 0.5), Px(center.y.0 - min.0 * 0.5)),
                        Size::new(min, min),
                    );

                    let chrome_rect = chrome.map(|chrome| {
                        Rect::new(
                            Point::new(
                                Px(center.x.0 - chrome.width.0 * 0.5),
                                Px(center.y.0 - chrome.height.0 * 0.5),
                            ),
                            chrome,
                        )
                    });

                    fn outline(
                        p: &mut fret_ui::canvas::CanvasPainter<'_>,
                        order: u32,
                        rect: Rect,
                        color: CoreColor,
                    ) {
                        p.scene().push(SceneOp::Quad {
                            order: DrawOrder(order),
                            rect,
                            background: fret_core::Paint::TRANSPARENT,

                            border: Edges::all(Px(1.0)),
                            border_paint: fret_core::Paint::Solid(color),

                            corner_radii: Corners::all(Px(0.0)),
                        });
                    }

                    outline(
                        p,
                        0,
                        bounds,
                        CoreColor {
                            r: 0.1,
                            g: 0.8,
                            b: 0.2,
                            a: 0.8,
                        },
                    );
                    outline(
                        p,
                        1,
                        min_rect,
                        CoreColor {
                            r: 0.95,
                            g: 0.75,
                            b: 0.2,
                            a: 0.9,
                        },
                    );
                    if let Some(chrome_rect) = chrome_rect {
                        outline(
                            p,
                            2,
                            chrome_rect,
                            CoreColor {
                                r: 0.2,
                                g: 0.75,
                                b: 0.95,
                                a: 0.9,
                            },
                        );
                    }
                });

                vec![child, overlay]
            },
        );

        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new(label).into_element(cx),
                shadcn::CardDescription::new(match chrome {
                    Some(chrome) => format!(
                        "min={}px, chrome={}x{}px",
                        min.0, chrome.width.0, chrome.height.0
                    ),
                    None => format!("min={}px", min.0),
                })
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![stack]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_px(Px(280.0)).min_w_0())
        .into_element(cx)
    };

    let checkbox_chrome = {
        let size = cx.with_theme(|theme| {
            theme
                .metric_by_key("md.comp.checkbox.state-layer.size")
                .unwrap_or(Px(40.0))
        });
        Size::new(size, size)
    };
    let radio_chrome = {
        let size = cx.with_theme(|theme| {
            theme
                .metric_by_key("md.comp.radio-button.state-layer.size")
                .unwrap_or(Px(40.0))
        });
        Size::new(size, size)
    };
    let switch_chrome = {
        let (width, height) = cx.with_theme(|theme| {
            (
                theme
                    .metric_by_key("md.comp.switch.track.width")
                    .unwrap_or(Px(52.0)),
                theme
                    .metric_by_key("md.comp.switch.state-layer.size")
                    .unwrap_or(Px(40.0)),
            )
        });
        Size::new(width, height)
    };
    let icon_button_chrome = {
        let size = cx.with_theme(|theme| {
            theme
                .metric_by_key("md.comp.icon-button.small.container.height")
                .unwrap_or(Px(40.0))
        });
        Size::new(size, size)
    };

    let grid = {
        let mut props = fret_ui::element::FlexProps::default();
        props.layout = fret_ui::element::LayoutStyle::default();
        props.layout.size.width = fret_ui::element::Length::Fill;
        props.direction = fret_core::Axis::Horizontal;
        props.wrap = true;
        props.gap = Px(16.0);
        props.align = fret_ui::element::CrossAlign::Start;
        props.justify = fret_ui::element::MainAlign::Start;

        cx.flex(props, move |cx| {
            let checkbox = material3::Checkbox::new(material3_checkbox.clone())
                .a11y_label("Material3 checkbox")
                .test_id("ui-gallery-material3-touch-target-checkbox")
                .into_element(cx);
            let radio = material3::Radio::new_value("alpha", material3_radio_value.clone())
                .a11y_label("Material3 radio")
                .test_id("ui-gallery-material3-touch-target-radio")
                .into_element(cx);
            let switch = material3::Switch::new(material3_switch.clone())
                .a11y_label("Material3 switch")
                .test_id("ui-gallery-material3-touch-target-switch")
                .into_element(cx);
            let icon_button = material3::IconButton::new(ids::ui::SETTINGS)
                .a11y_label("Material3 icon button")
                .test_id("ui-gallery-material3-touch-target-icon-button")
                .into_element(cx);
            let tabs = material3::Tabs::new(material3_tabs_value.clone())
                .a11y_label("Material3 tabs (touch targets)")
                .test_id("ui-gallery-material3-touch-target-tabs")
                .scrollable(true)
                .items(vec![
                    material3::TabItem::new("overview", "A")
                        .a11y_label("Material3 tab")
                        .test_id("ui-gallery-material3-touch-target-tab"),
                ])
                .into_element(cx);

            vec![
                target_overlay(cx, "Checkbox", Some(checkbox_chrome), checkbox),
                target_overlay(cx, "Radio", Some(radio_chrome), radio),
                target_overlay(cx, "Switch", Some(switch_chrome), switch),
                target_overlay(cx, "Icon Button", Some(icon_button_chrome), icon_button),
                target_overlay(cx, "Tabs (scrollable, 1 item)", None, tabs),
            ]
        })
    };

    vec![
        cx.text("Touch target overlay legend: green=bounds, yellow=min 48x48, cyan=token chrome (if shown)."),
        grid,
    ]
}
