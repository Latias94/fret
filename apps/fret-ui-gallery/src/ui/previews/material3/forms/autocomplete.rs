use super::super::super::super::*;

pub(in crate::ui) fn preview_material3_autocomplete(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
    disabled: Model<bool>,
    error: Model<bool>,
    dialog_open: Model<bool>,
) -> Vec<AnyElement> {
    let disabled_now = cx
        .get_model_copied(&disabled, Invalidation::Layout)
        .unwrap_or(false);
    let error_now = cx
        .get_model_copied(&error, Invalidation::Layout)
        .unwrap_or(false);

    #[derive(Default)]
    struct LocalState {
        selected_value: Option<Model<Option<Arc<str>>>>,
        exposed_selected_value: Option<Model<Option<Arc<str>>>>,
        exposed_query: Option<Model<String>>,
    }

    let selected_value = cx.with_state(LocalState::default, |st| st.selected_value.clone());
    let selected_value = if let Some(model) = selected_value {
        model
    } else {
        let model = cx.app.models_mut().insert(None::<Arc<str>>);
        cx.with_state(LocalState::default, |st| {
            st.selected_value = Some(model.clone())
        });
        model
    };

    let exposed_selected_value =
        cx.with_state(LocalState::default, |st| st.exposed_selected_value.clone());
    let exposed_selected_value = if let Some(model) = exposed_selected_value {
        model
    } else {
        let model = cx
            .app
            .models_mut()
            .insert(Some(Arc::<str>::from("beta")) as Option<Arc<str>>);
        cx.with_state(LocalState::default, |st| {
            st.exposed_selected_value = Some(model.clone())
        });
        model
    };

    let exposed_query = cx.with_state(LocalState::default, |st| st.exposed_query.clone());
    let exposed_query = if let Some(model) = exposed_query {
        model
    } else {
        let model = cx.app.models_mut().insert(String::new());
        cx.with_state(LocalState::default, |st| {
            st.exposed_query = Some(model.clone())
        });
        model
    };

    let query_now = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_default();
    let selected_now = cx
        .get_model_cloned(&selected_value, Invalidation::Layout)
        .unwrap_or(None);
    let selected_label = selected_now.as_deref().unwrap_or("<none>");

    let exposed_selected_now = cx
        .get_model_cloned(&exposed_selected_value, Invalidation::Layout)
        .unwrap_or(None);
    let exposed_selected_label = exposed_selected_now.as_deref().unwrap_or("<none>");

    let toggles = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                cx.text("disabled"),
                material3::Switch::new(disabled.clone())
                    .a11y_label("Disable autocomplete")
                    .test_id("ui-gallery-material3-autocomplete-disabled")
                    .into_element(cx),
                cx.text("error"),
                material3::Switch::new(error.clone())
                    .a11y_label("Toggle autocomplete error state")
                    .test_id("ui-gallery-material3-autocomplete-error")
                    .into_element(cx),
            ]
        },
    );

    let items: Arc<[material3::AutocompleteItem]> = Arc::from(vec![
        material3::AutocompleteItem::new("alpha", "Alpha"),
        material3::AutocompleteItem::new("beta", "Beta"),
        material3::AutocompleteItem::new("gamma", "Gamma"),
        material3::AutocompleteItem::new("delta", "Delta"),
        material3::AutocompleteItem::new("epsilon", "Epsilon"),
        material3::AutocompleteItem::new("zeta", "Zeta"),
    ]);

    let supporting = if error_now {
        "Error: required"
    } else {
        "Supporting text"
    };

    let outlined = material3::Autocomplete::new(value.clone())
        .selected_value(selected_value.clone())
        .variant(material3::AutocompleteVariant::Outlined)
        .label("Search")
        .placeholder("Type to filter")
        .supporting_text(supporting)
        .items(items.clone())
        .disabled(disabled_now)
        .error(error_now)
        .a11y_label("autocomplete outlined")
        .test_id("ui-gallery-material3-autocomplete")
        .into_element(cx);

    let outlined_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Outlined").into_element(cx),
            shadcn::CardDescription::new(
                "Combobox-style: focus stays on the input; the active option is exposed via active-descendant.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![outlined]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let filled = material3::Autocomplete::new(value.clone())
        .selected_value(selected_value.clone())
        .variant(material3::AutocompleteVariant::Filled)
        .label("Search (filled)")
        .placeholder("Type to filter")
        .supporting_text(supporting)
        .items(items.clone())
        .disabled(disabled_now)
        .error(error_now)
        .a11y_label("autocomplete filled")
        .test_id("ui-gallery-material3-autocomplete-filled")
        .into_element(cx);

    let filled_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Filled").into_element(cx),
            shadcn::CardDescription::new(
                "Filled container + active indicator outcomes (token-driven).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![filled]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let exposed = material3::ExposedDropdown::new(exposed_selected_value.clone())
        .query(exposed_query.clone())
        .variant(material3::AutocompleteVariant::Outlined)
        .label("Searchable select")
        .placeholder("Type to filter")
        .supporting_text(
            "Policy: when the input blurs, the query reverts to the committed selection.",
        )
        .items(items.clone())
        .disabled(disabled_now)
        .error(error_now)
        .a11y_label("exposed dropdown")
        .test_id("ui-gallery-material3-exposed-dropdown")
        .into_element(cx);

    let exposed_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Exposed dropdown (composition)").into_element(cx),
            shadcn::CardDescription::new(
                "Compose-style: a committed selection model drives the closed display, while the query stays editable while focused.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![exposed]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let open_action: fret_ui::action::OnActivate = {
        let dialog_open = dialog_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&dialog_open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };
    let close_action: fret_ui::action::OnActivate = {
        let dialog_open = dialog_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&dialog_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };

    let dialog = material3::Dialog::new(dialog_open.clone())
        .headline("Autocomplete (Dialog probe)")
        .supporting_text("Overlay should anchor correctly inside a modal dialog without clipping.")
        .actions(vec![material3::DialogAction::new("Close").on_activate(close_action)])
        .test_id("ui-gallery-material3-autocomplete-dialog")
        .into_element(
            cx,
            move |cx| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N4),
                    move |cx| {
                        vec![
                            material3::Button::new("Open dialog probe")
                                .variant(material3::ButtonVariant::Filled)
                                .on_activate(open_action.clone())
                                .test_id("ui-gallery-material3-autocomplete-dialog-open")
                                .into_element(cx),
                            cx.text("Tip: focus the autocomplete and press ArrowDown; keep typing while the menu is open."),
                        ]
                    },
                )
            },
            {
                let items = items.clone();
                let value = value.clone();
                move |cx| {
                    let spacer = cx.container(
                        fret_ui::element::ContainerProps {
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.size.width = fret_ui::element::Length::Fill;
                                l.size.height = fret_ui::element::Length::Px(Px(360.0));
                                l
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::<AnyElement>::new(),
                    );

                    vec![stack::vstack(
                        cx,
                        stack::VStackProps::default()
                                    .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N4),
                        move |cx| {
                            vec![
                                material3::Autocomplete::new(value.clone())
                                    .selected_value(selected_value.clone())
                                    .variant(material3::AutocompleteVariant::Outlined)
                                    .label("Dialog autocomplete")
                                    .placeholder("Type to filter")
                                    .supporting_text("Bottom-edge clamping probe: open near the dialog bottom.")
                                    .items(items.clone())
                                    .a11y_label("autocomplete dialog")
                                    .test_id("ui-gallery-material3-autocomplete-dialog-field")
                                    .into_element(cx),
                                spacer,
                                material3::Autocomplete::new(value.clone())
                                    .selected_value(selected_value.clone())
                                    .variant(material3::AutocompleteVariant::Outlined)
                                    .label("Dialog autocomplete (bottom)")
                                    .placeholder("Type to filter")
                                    .supporting_text("Open menu near the dialog bottom edge.")
                                    .items(items.clone())
                                    .a11y_label("autocomplete dialog bottom")
                                    .test_id("ui-gallery-material3-autocomplete-dialog-field-bottom")
                                    .into_element(cx),
                            ]
                        },
                    )]
                }
            },
        );

    vec![
        cx.text("Material 3 Autocomplete: editable combobox input with a listbox popover menu."),
        toggles,
        cx.text(Arc::from(format!(
            "Query: \"{}\" | Selected value: {}",
            query_now, selected_label
        ))),
        cx.text(Arc::from(format!(
            "Exposed dropdown committed value: {}",
            exposed_selected_label
        ))),
        exposed_card,
        outlined_card,
        filled_card,
        dialog,
    ]
}
