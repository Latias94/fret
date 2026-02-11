use super::super::super::*;

pub(in crate::ui) fn preview_material3_bottom_sheet(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui_material3::{
        Button, ButtonVariant, DockedBottomSheet, DockedBottomSheetVariant, ModalBottomSheet,
    };

    let open_sheet: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };
    let close_sheet: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };

    let underlay = move |cx: &mut ElementContext<'_, App>| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N4),
            move |cx| {
                let docked =
                    DockedBottomSheet::new()
                        .variant(DockedBottomSheetVariant::Standard)
                        .test_id("ui-gallery-material3-bottom-sheet-docked")
                        .into_element(cx, |cx| {
                            vec![
                        cx.text("Docked (standard) sheet: token-driven container + drag handle."),
                        Button::new("Primary action")
                            .variant(ButtonVariant::Filled)
                            .test_id("ui-gallery-material3-bottom-sheet-docked-primary")
                            .into_element(cx),
                        Button::new("Secondary action")
                            .variant(ButtonVariant::Outlined)
                            .test_id("ui-gallery-material3-bottom-sheet-docked-secondary")
                            .into_element(cx),
                    ]
                        });

                vec![
                cx.text(
                    "Material 3 Bottom Sheet: primitives driven by md.comp.sheet.bottom.* tokens.",
                ),
                Button::new("Open modal bottom sheet")
                    .variant(ButtonVariant::Filled)
                    .on_activate(open_sheet.clone())
                    .test_id("ui-gallery-material3-bottom-sheet-open")
                    .into_element(cx),
                Button::new("Underlay focus probe")
                    .variant(ButtonVariant::Outlined)
                    .test_id("ui-gallery-material3-bottom-sheet-underlay-probe")
                    .into_element(cx),
                cx.text(
                    "Tip: click the scrim to dismiss; Tab should stay inside the sheet while open.",
                ),
                docked,
            ]
            },
        )
    };

    let sheet = ModalBottomSheet::new(open)
        .test_id("ui-gallery-material3-bottom-sheet")
        .into_element(cx, underlay, move |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N4),
                move |cx| {
                    vec![
                        cx.text("Modal bottom sheet content."),
                        Button::new("Close")
                            .variant(ButtonVariant::Filled)
                            .on_activate(close_sheet.clone())
                            .test_id("ui-gallery-material3-bottom-sheet-close")
                            .into_element(cx),
                    ]
                },
            )]
        });

    vec![sheet]
}

pub(in crate::ui) fn preview_material3_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<time::Date>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui_material3::{Button, ButtonVariant, DatePickerDialog, DockedDatePicker};

    let open_dialog: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };

    let selected_value = cx
        .get_model_cloned(&selected, Invalidation::Layout)
        .unwrap_or(None);
    let selected_label: Arc<str> = match selected_value {
        Some(date) => Arc::from(format!("Selected: {date}")),
        None => Arc::<str>::from("Selected: <none>"),
    };

    let dialog = DatePickerDialog::new(open.clone(), month.clone(), selected.clone())
        .test_id("ui-gallery-material3-date-picker")
        .into_element(cx, move |cx| {
            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N4),
                move |cx| {
                    let docked = DockedDatePicker::new(month.clone(), selected.clone())
                        .test_id("ui-gallery-material3-date-picker-docked")
                        .into_element(cx);

                    vec![
                        cx.text(
                            "Material 3 Date Picker: primitives driven by md.comp.date-picker.* tokens.",
                        ),
                        cx.text(selected_label.clone()),
                        Button::new("Open date picker dialog")
                            .variant(ButtonVariant::Filled)
                            .on_activate(open_dialog.clone())
                            .test_id("ui-gallery-material3-date-picker-open")
                            .into_element(cx),
                        Button::new("Underlay focus probe")
                            .variant(ButtonVariant::Outlined)
                            .test_id("ui-gallery-material3-date-picker-underlay-probe")
                            .into_element(cx),
                        docked,
                    ]
                },
            )
        });

    vec![dialog]
}

pub(in crate::ui) fn preview_material3_time_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    selected: Model<time::Time>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui_material3::{Button, ButtonVariant, DockedTimePicker, TimePickerDialog};

    let open_dialog: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };

    let selected_value = cx
        .get_model_copied(&selected, Invalidation::Layout)
        .unwrap_or_else(|| time::Time::from_hms(9, 41, 0).expect("valid time"));
    let selected_label: Arc<str> = Arc::from(format!(
        "Selected: {:02}:{:02}",
        selected_value.hour(),
        selected_value.minute()
    ));

    let dialog = TimePickerDialog::new(open.clone(), selected.clone())
        .test_id("ui-gallery-material3-time-picker")
        .into_element(cx, move |cx| {
            stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N4),
                move |cx| {
                    let docked = DockedTimePicker::new(selected.clone())
                        .test_id("ui-gallery-material3-time-picker-docked")
                        .into_element(cx);

                    vec![
                        cx.text(
                            "Material 3 Time Picker: primitives driven by md.comp.time-picker.* tokens.",
                        ),
                        cx.text(selected_label.clone()),
                        Button::new("Open time picker dialog")
                            .variant(ButtonVariant::Filled)
                            .on_activate(open_dialog.clone())
                            .test_id("ui-gallery-material3-time-picker-open")
                            .into_element(cx),
                        Button::new("Underlay focus probe")
                            .variant(ButtonVariant::Outlined)
                            .test_id("ui-gallery-material3-time-picker-underlay-probe")
                            .into_element(cx),
                        docked,
                    ]
                },
            )
        });

    vec![dialog]
}

pub(in crate::ui) fn preview_material3_segmented_button(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    use std::collections::BTreeSet;

    use fret_ui_material3::{SegmentedButtonItem, SegmentedButtonSet};

    #[derive(Default)]
    struct SegmentedButtonPageModels {
        single_value: Option<Model<Arc<str>>>,
        multi_value: Option<Model<BTreeSet<Arc<str>>>>,
    }

    let single_value = cx.with_state(SegmentedButtonPageModels::default, |st| {
        st.single_value.clone()
    });
    let single_value = match single_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("alpha"));
            cx.with_state(SegmentedButtonPageModels::default, |st| {
                st.single_value = Some(model.clone())
            });
            model
        }
    };

    let multi_value = cx.with_state(SegmentedButtonPageModels::default, |st| {
        st.multi_value.clone()
    });
    let multi_value = match multi_value {
        Some(model) => model,
        None => {
            let initial: BTreeSet<Arc<str>> = [Arc::<str>::from("alpha")].into_iter().collect();
            let model = cx.app.models_mut().insert(initial);
            cx.with_state(SegmentedButtonPageModels::default, |st| {
                st.multi_value = Some(model.clone())
            });
            model
        }
    };

    let single_current = cx
        .get_model_cloned(&single_value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));
    let multi_current_len = cx
        .get_model_cloned(&multi_value, Invalidation::Layout)
        .map(|set| set.len())
        .unwrap_or(0);

    let content = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N4).items_start(),
        |cx| {
            vec![
                SegmentedButtonSet::single(single_value.clone())
                    .items(vec![
                        SegmentedButtonItem::new("alpha", "Alpha")
                            .icon(fret_icons::ids::ui::SEARCH)
                            .test_id("ui-gallery-material3-segmented-single-alpha"),
                        SegmentedButtonItem::new("beta", "Beta")
                            .icon(fret_icons::ids::ui::SETTINGS)
                            .test_id("ui-gallery-material3-segmented-single-beta"),
                        SegmentedButtonItem::new("gamma", "Gamma (disabled)")
                            .disabled(true)
                            .icon(fret_icons::ids::ui::MORE_HORIZONTAL)
                            .test_id("ui-gallery-material3-segmented-single-gamma-disabled"),
                    ])
                    .a11y_label("Material 3 Segmented Button (single)")
                    .test_id("ui-gallery-material3-segmented-single")
                    .into_element(cx),
                cx.text(format!("single={}", single_current.as_ref())),
                SegmentedButtonSet::multi(multi_value.clone())
                    .items(vec![
                        SegmentedButtonItem::new("alpha", "Alpha")
                            .test_id("ui-gallery-material3-segmented-multi-alpha"),
                        SegmentedButtonItem::new("beta", "Beta")
                            .test_id("ui-gallery-material3-segmented-multi-beta"),
                        SegmentedButtonItem::new("gamma", "Gamma (disabled)")
                            .disabled(true)
                            .test_id("ui-gallery-material3-segmented-multi-gamma-disabled"),
                    ])
                    .a11y_label("Material 3 Segmented Button (multi)")
                    .test_id("ui-gallery-material3-segmented-multi")
                    .into_element(cx),
                cx.text(format!("multi_count={multi_current_len}")),
            ]
        },
    );

    vec![
        cx.text("Material 3 Segmented Buttons: token-driven outcomes + roving focus + selection."),
        content,
    ]
}

pub(in crate::ui) fn preview_material3_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    #[derive(Default)]
    struct SelectPageModels {
        selected: Option<Model<Option<Arc<str>>>>,
        selected_unclamped: Option<Model<Option<Arc<str>>>>,
        selected_typeahead: Option<Model<Option<Arc<str>>>>,
        selected_rich: Option<Model<Option<Arc<str>>>>,
        selected_transformed: Option<Model<Option<Arc<str>>>>,
        menu_width_floor_enabled: Option<Model<bool>>,
        typeahead_delay_ms: Option<Model<u32>>,
    }

    let selected = cx.with_state(SelectPageModels::default, |st| st.selected.clone());
    let selected = match selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(SelectPageModels::default, |st| {
                st.selected = Some(model.clone())
            });
            model
        }
    };

    let selected_unclamped = cx.with_state(SelectPageModels::default, |st| {
        st.selected_unclamped.clone()
    });
    let selected_unclamped = match selected_unclamped {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(SelectPageModels::default, |st| {
                st.selected_unclamped = Some(model.clone())
            });
            model
        }
    };

    let selected_typeahead = cx.with_state(SelectPageModels::default, |st| {
        st.selected_typeahead.clone()
    });
    let selected_typeahead = match selected_typeahead {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(SelectPageModels::default, |st| {
                st.selected_typeahead = Some(model.clone())
            });
            model
        }
    };

    let selected_rich = cx.with_state(SelectPageModels::default, |st| st.selected_rich.clone());
    let selected_rich = match selected_rich {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(SelectPageModels::default, |st| {
                st.selected_rich = Some(model.clone())
            });
            model
        }
    };

    let selected_transformed = cx.with_state(SelectPageModels::default, |st| {
        st.selected_transformed.clone()
    });
    let selected_transformed = match selected_transformed {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(SelectPageModels::default, |st| {
                st.selected_transformed = Some(model.clone())
            });
            model
        }
    };

    let menu_width_floor_enabled = cx.with_state(SelectPageModels::default, |st| {
        st.menu_width_floor_enabled.clone()
    });
    let menu_width_floor_enabled = match menu_width_floor_enabled {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(SelectPageModels::default, |st| {
                st.menu_width_floor_enabled = Some(model.clone())
            });
            model
        }
    };
    let menu_width_floor_enabled_now = cx
        .get_model_copied(&menu_width_floor_enabled, Invalidation::Layout)
        .unwrap_or(true);

    let typeahead_delay_ms = cx.with_state(SelectPageModels::default, |st| {
        st.typeahead_delay_ms.clone()
    });
    let typeahead_delay_ms = match typeahead_delay_ms {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(200u32);
            cx.with_state(SelectPageModels::default, |st| {
                st.typeahead_delay_ms = Some(model.clone())
            });
            model
        }
    };
    let typeahead_delay_ms_now = cx
        .get_model_copied(&typeahead_delay_ms, Invalidation::Layout)
        .unwrap_or(200);

    let items: Arc<[material3::SelectItem]> = vec![
        material3::SelectItem::new("alpha", "Alpha").test_id("ui-gallery-material3-select-a"),
        material3::SelectItem::new("beta", "Beta").test_id("ui-gallery-material3-select-b"),
        material3::SelectItem::new("charlie", "Charlie (disabled)")
            .disabled(true)
            .test_id("ui-gallery-material3-select-c-disabled"),
    ]
    .into();

    let default = material3::Select::new(selected.clone())
        .a11y_label("Material 3 Select")
        .placeholder("Pick one")
        .items(items.clone())
        .test_id("ui-gallery-material3-select")
        .into_element(cx);

    let (primary, primary_container, secondary_container) = cx.with_theme(|theme| {
        (
            theme.color_required("md.sys.color.primary"),
            theme.color_required("md.sys.color.primary-container"),
            theme.color_required("md.sys.color.secondary-container"),
        )
    });

    let override_style = material3::SelectStyle::default()
        .container_background(
            WidgetStateProperty::new(None)
                .when(WidgetStates::OPEN, Some(ColorRef::Color(primary_container))),
        )
        .outline_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::FOCUS_VISIBLE, Some(ColorRef::Color(primary))),
        )
        .trailing_icon_color(
            WidgetStateProperty::new(None).when(WidgetStates::OPEN, Some(ColorRef::Color(primary))),
        )
        .menu_selected_container_color(WidgetStateProperty::new(Some(ColorRef::Color(
            secondary_container,
        ))));

    let overridden = material3::Select::new(selected.clone())
        .a11y_label("Material 3 Select (override)")
        .placeholder("Pick one")
        .items(items)
        .style(override_style)
        .test_id("ui-gallery-material3-select-overridden")
        .into_element(cx);

    let unclamped_items: Arc<[material3::SelectItem]> = vec![
        material3::SelectItem::new("short", "Short")
            .test_id("ui-gallery-material3-select-unclamped-item-short"),
        material3::SelectItem::new("medium", "Medium option")
            .test_id("ui-gallery-material3-select-unclamped-item-medium"),
        material3::SelectItem::new(
            "long",
            "A very long option label that should expand the menu beyond the anchor width",
        )
        .test_id("ui-gallery-material3-select-unclamped-item-long"),
        material3::SelectItem::new("long2", "Another long-ish label for measuring menu width")
            .test_id("ui-gallery-material3-select-unclamped-item-long2"),
        material3::SelectItem::new(
            "xl",
            "Extra long: The quick brown fox jumps over the lazy dog",
        )
        .test_id("ui-gallery-material3-select-unclamped-item-xl"),
    ]
    .into();

    let unclamped = material3::Select::new(selected_unclamped.clone())
        .a11y_label("Material 3 Select (unclamped menu width)")
        .placeholder("Unclamped")
        .items(unclamped_items)
        .match_anchor_width(false)
        .menu_width_floor(if menu_width_floor_enabled_now {
            Px(210.0)
        } else {
            Px(0.0)
        })
        .typeahead_delay_ms(typeahead_delay_ms_now)
        .test_id("ui-gallery-material3-select-unclamped")
        .into_element(cx);

    let floor_toggle = material3::Switch::new(menu_width_floor_enabled.clone())
        .a11y_label("Select menu width floor (210px)")
        .test_id("ui-gallery-material3-select-menu-width-floor-toggle")
        .into_element(cx);

    let typeahead_items: Arc<[material3::SelectItem]> = vec![
        material3::SelectItem::new("beta", "Beta")
            .test_id("ui-gallery-material3-select-typeahead-item-beta"),
        material3::SelectItem::new("charlie", "Charlie (disabled)")
            .disabled(true)
            .test_id("ui-gallery-material3-select-typeahead-item-charlie-disabled"),
        material3::SelectItem::new("delta", "Delta")
            .test_id("ui-gallery-material3-select-typeahead-item-delta"),
        material3::SelectItem::new("echo", "Echo")
            .test_id("ui-gallery-material3-select-typeahead-item-echo"),
    ]
    .into();

    let set_delay_button = |cx: &mut ElementContext<'_, App>, ms: u32| -> AnyElement {
        use fret_ui::action::OnActivate;

        let delay_model = typeahead_delay_ms.clone();
        let on_activate: OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&delay_model, |v| *v = ms);
            host.request_redraw(action_cx.window);
        });

        material3::Button::new(format!("{ms}ms"))
            .variant(if typeahead_delay_ms_now == ms {
                material3::ButtonVariant::Filled
            } else {
                material3::ButtonVariant::Outlined
            })
            .test_id(format!("ui-gallery-material3-select-typeahead-delay-{ms}"))
            .on_activate(on_activate)
            .into_element(cx)
    };

    let typeahead_select = material3::Select::new(selected_typeahead.clone())
        .a11y_label("Material 3 Select (typeahead delay)")
        .placeholder("Typeahead probe")
        .items(typeahead_items)
        .typeahead_delay_ms(typeahead_delay_ms_now)
        .test_id("ui-gallery-material3-select-typeahead")
        .into_element(cx);

    let rich_items: Arc<[material3::SelectItem]> = vec![
        material3::SelectItem::new("alpha", "Alpha")
            .supporting_text("Supporting: quick summary")
            .trailing_supporting_text("⌘A")
            .leading_icon(fret_icons::ids::ui::SEARCH)
            .test_id("ui-gallery-material3-select-rich-item-alpha"),
        material3::SelectItem::new("beta", "Beta")
            .supporting_text("Supporting: secondary line")
            .trailing_supporting_text("⌘B")
            .leading_icon(fret_icons::ids::ui::SETTINGS)
            .test_id("ui-gallery-material3-select-rich-item-beta"),
        material3::SelectItem::new("charlie", "Charlie (disabled)")
            .supporting_text("Disabled items are skipped by typeahead/roving")
            .disabled(true)
            .leading_icon(fret_icons::ids::ui::SLASH)
            .test_id("ui-gallery-material3-select-rich-item-charlie-disabled"),
        material3::SelectItem::new("delta", "Delta")
            .supporting_text("Trailing-only still aligns")
            .trailing_supporting_text("⌘D")
            .test_id("ui-gallery-material3-select-rich-item-delta"),
    ]
    .into();

    let rich_select = material3::Select::new(selected_rich.clone())
        .a11y_label("Material 3 Select (supporting text options)")
        .placeholder("Option richness probe")
        .items(rich_items)
        .typeahead_delay_ms(typeahead_delay_ms_now)
        .test_id("ui-gallery-material3-select-rich")
        .into_element(cx);

    let transformed_items: Arc<[material3::SelectItem]> = vec![
        material3::SelectItem::new("alpha", "Alpha")
            .test_id("ui-gallery-material3-select-transformed-item-alpha"),
        material3::SelectItem::new("beta", "Beta")
            .test_id("ui-gallery-material3-select-transformed-item-beta"),
        material3::SelectItem::new("gamma", "Gamma")
            .test_id("ui-gallery-material3-select-transformed-item-gamma"),
    ]
    .into();

    let transformed_select = material3::Select::new(selected_transformed.clone())
        .a11y_label("Material 3 Select (transformed)")
        .placeholder("Transformed")
        .items(transformed_items)
        .test_id("ui-gallery-material3-select-transformed")
        .into_element(cx);

    let (probe_bg, probe_border) = cx.with_theme(|theme| {
        let bg = theme
            .color_by_key("md.sys.color.surface-container")
            .or_else(|| theme.color_by_key("md.sys.color.surface"))
            .unwrap_or(fret_core::Color::TRANSPARENT);
        let border = theme
            .color_by_key("md.sys.color.outline-variant")
            .unwrap_or(fret_core::Color::TRANSPARENT);
        (bg, border)
    });
    let transformed_probe = cx.container(
        fret_ui::element::ContainerProps {
            layout: {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.size.width = fret_ui::element::Length::Fill;
                l.size.height = fret_ui::element::Length::Px(Px(88.0));
                l.overflow = fret_ui::element::Overflow::Clip;
                l
            },
            background: Some(probe_bg),
            border: fret_core::Edges::all(Px(1.0)),
            border_color: Some(probe_border),
            corner_radii: fret_core::Corners::all(Px(12.0)),
            padding: fret_core::Edges::all(Px(12.0)),
            ..Default::default()
        },
        move |cx| {
            let transform =
                fret_core::Transform2D::translation(fret_core::Point::new(Px(12.0), Px(6.0)))
                    * fret_core::Transform2D::scale_uniform(0.92);
            vec![cx.visual_transform(transform, |_cx| vec![transformed_select.clone()])]
        },
    );

    vec![
        cx.text(
            "Material 3 Select: token-driven trigger + listbox overlay + ADR 0220 style overrides.",
        ),
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N4).items_start(),
            move |cx| {
                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N4).items_center(),
                        move |_cx| vec![default, overridden],
                    ),
                    cx.text("Option richness probe (Material Web select-option supporting slots):"),
                    rich_select,
                    cx.text("Menu width probe (Material Web min-width behavior + optional 210px floor):"),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |cx| {
                            vec![
                                cx.text("menu_width_floor=210px"),
                                floor_toggle,
                                cx.text(if menu_width_floor_enabled_now { "on" } else { "off" }),
                            ]
                        },
                    ),
                    unclamped,
                    cx.text(format!(
                        "Typeahead delay probe (Material Web typeaheadDelay): current={}ms",
                        typeahead_delay_ms_now
                    )),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |cx| vec![
                            set_delay_button(cx, 200),
                            set_delay_button(cx, 500),
                            set_delay_button(cx, 1000),
                        ],
                    ),
                    typeahead_select,
                    cx.text(
                        "Menu positioning probe (Material Web menuPositioning): select is render-transformed + clipped; overlay should still align and avoid clipping.",
                    ),
                    transformed_probe,
                ]
            },
        ),
    ]
}

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

pub(in crate::ui) fn preview_material3_text_field(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
    disabled: Model<bool>,
    error: Model<bool>,
) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

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

    let override_style = material3::TextFieldStyle::default()
        .outline_color(WidgetStateProperty::new(None).when(
            WidgetStates::FOCUS_VISIBLE,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.2,
                g: 0.8,
                b: 0.4,
                a: 1.0,
            })),
        ))
        .caret_color(WidgetStateProperty::new(Some(ColorRef::Color(
            fret_core::Color {
                r: 0.2,
                g: 0.8,
                b: 0.4,
                a: 1.0,
            },
        ))))
        .placeholder_color(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.9,
                g: 0.2,
                b: 0.9,
                a: 1.0,
            })),
        ));
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

    vec![
        cx.text(
            "Material 3 Text Field: outlined + filled variants (token-driven chrome + label/placeholder outcomes).",
        ),
        toggles,
        outlined_card,
        filled_card,
        override_card,
    ]
}
