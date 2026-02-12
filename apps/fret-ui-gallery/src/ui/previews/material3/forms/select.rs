use super::super::super::super::*;

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
