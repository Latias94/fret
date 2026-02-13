use super::super::super::super::*;

pub(in crate::ui) fn preview_virtual_list_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    virtual_list_torture_jump: Model<String>,
    virtual_list_torture_edit_row: Model<Option<u64>>,
    virtual_list_torture_edit_text: Model<String>,
    virtual_list_torture_scroll: VirtualListScrollHandle,
) -> Vec<AnyElement> {
    let len: usize = 10_000;

    let minimal_harness =
        match std::env::var_os("FRET_UI_GALLERY_VLIST_MINIMAL").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => false,
        };

    let known_heights =
        match std::env::var_os("FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => false,
        };

    let variable_height =
        match std::env::var_os("FRET_UI_GALLERY_VLIST_VARIABLE_HEIGHT").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => false,
        };

    let retained_host =
        match std::env::var_os("FRET_UI_GALLERY_VLIST_RETAINED").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => true,
        };

    let row_cache =
        match std::env::var_os("FRET_UI_GALLERY_VLIST_ROW_CACHE").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => false,
        };

    let keep_alive: usize = std::env::var("FRET_UI_GALLERY_VLIST_KEEP_ALIVE")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(0);

    let header_editing_row = (!minimal_harness)
        .then(|| {
            cx.get_model_copied(&virtual_list_torture_edit_row, Invalidation::Layout)
                .flatten()
        })
        .flatten();

    let controls = (!minimal_harness).then(|| {
        let jump_input = {
            let mut props =
                fret_ui::element::TextInputProps::new(virtual_list_torture_jump.clone());
            props.a11y_label = Some(Arc::<str>::from("Jump to row"));
            props.test_id = Some(Arc::<str>::from("ui-gallery-virtual-list-jump-input"));
            props.placeholder = Some(Arc::<str>::from("Row index (e.g. 9000)"));
            props.layout.size.width = fret_ui::element::Length::Fill;
            cx.text_input(props)
        };

        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2)
                .items_center(),
            |cx| {
                let jump_model = virtual_list_torture_jump.clone();
                let scroll_for_jump = virtual_list_torture_scroll.clone();
                let on_jump: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let raw = host
                            .models_mut()
                            .get_cloned(&jump_model)
                            .unwrap_or_default();
                        let index = raw.trim().parse::<usize>().unwrap_or(0);
                        scroll_for_jump
                            .scroll_to_item(index, fret_ui::scroll::ScrollStrategy::Start);
                        host.request_redraw(action_cx.window);
                    });

                let scroll_for_bottom = virtual_list_torture_scroll.clone();
                let on_bottom: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        scroll_for_bottom.scroll_to_bottom();
                        host.request_redraw(action_cx.window);
                    });

                let edit_row_for_clear = virtual_list_torture_edit_row.clone();
                let edit_text_for_clear = virtual_list_torture_edit_text.clone();
                let on_clear_edit: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let _ = host.models_mut().update(&edit_row_for_clear, |v| *v = None);
                        let _ = host
                            .models_mut()
                            .update(&edit_text_for_clear, |v| v.clear());
                        host.request_redraw(action_cx.window);
                    });

                vec![
                    jump_input,
                    shadcn::Button::new("Jump")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id("ui-gallery-virtual-list-jump-button")
                        .on_activate(on_jump)
                        .into_element(cx),
                    shadcn::Button::new("Bottom")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id("ui-gallery-virtual-list-bottom-button")
                        .on_activate(on_bottom)
                        .into_element(cx),
                    shadcn::Button::new("Clear edit")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id("ui-gallery-virtual-list-clear-edit-button")
                        .on_activate(on_clear_edit)
                        .into_element(cx),
                ]
            },
        )
    });

    let editing_indicator = (!minimal_harness).then(|| {
        let label = if let Some(row) = header_editing_row {
            Arc::<str>::from(format!("editing_row={row}"))
        } else {
            Arc::<str>::from("editing_row=<none>")
        };

        let text = if let Some(row) = header_editing_row {
            cx.text(format!("Editing row: {row}"))
        } else {
            cx.text("Editing row: <none>")
        };
        text.attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Text)
                .label(label)
                .test_id("ui-gallery-virtual-list-editing"),
        )
    });

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            let mut out = vec![
                cx.text("Goal: deterministic virtualization torture surface (10k rows + scroll-to-item + inline edit)."),
                cx.text(if retained_host {
                    "Mode: retained host (virt-003 prototype; item subtrees can reattach without rerendering the parent cache root)."
                } else {
                    "Mode: render-driven (baseline; visible items update requires rerender when the window changes)."
                }),
                cx.text(if known_heights {
                    "Mode: known row heights (no measure pass; better for perf baselines)."
                } else {
                    "Mode: measured row heights (baseline)."
                }),
                cx.text(if keep_alive > 0 {
                    format!("Mode: keep-alive enabled (budget={keep_alive}).")
                } else {
                    "Mode: keep-alive disabled (budget=0).".to_string()
                }),
            ];

            if minimal_harness {
                out.push(cx.text("Harness: minimal (no focusable controls; reduces RAF/notify noise in perf bundles)."));
            } else {
                if let Some(controls) = controls {
                    out.push(controls);
                }
                if let Some(editing_indicator) = editing_indicator {
                    out.push(editing_indicator);
                }
            }

            out
        },
    );

    let list_layout = fret_ui::element::LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: fret_ui::element::Length::Fill,
            height: fret_ui::element::Length::Px(Px(420.0)),
            ..Default::default()
        },
        overflow: fret_ui::element::Overflow::Clip,
        ..Default::default()
    };

    let options = if known_heights {
        fret_ui::element::VirtualListOptions::known(Px(28.0), 10, |index| {
            if index % 15 == 0 { Px(44.0) } else { Px(28.0) }
        })
    } else {
        fret_ui::element::VirtualListOptions::new(Px(28.0), 10)
    };

    let options = if retained_host && keep_alive > 0 {
        options.keep_alive(keep_alive)
    } else {
        options
    };

    let list = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        let list = if minimal_harness {
            if retained_host {
                let theme = theme.clone();
                let key_at = Arc::new(|i| i as fret_ui::ItemKey);
                let row = Arc::new(move |cx: &mut ElementContext<'_, App>, index: usize| {
                    let zebra = (index % 2) == 0;
                    let background = if zebra {
                        theme.color_required("muted")
                    } else {
                        theme.color_required("background")
                    };

                    let height_hint = if index % 15 == 0 { Px(44.0) } else { Px(28.0) };
                    let row_label = cx.text(format!("Row {index}"));
                    let extra_line = cx.text(format!(
                        "Details: index={index} seed={} repeat={}",
                        index.wrapping_mul(2654435761),
                        (index % 7) + 1
                    ));

                    let mut container_props = decl_style::container_props(
                        &theme,
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(background))
                            .p(Space::N2),
                        {
                            let mut layout = LayoutRefinement::default().w_full();
                            if !variable_height {
                                layout = layout.h_px(MetricRef::Px(height_hint));
                            }
                            layout
                        },
                    );
                    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

                    let container = cx.container(container_props, |_cx| {
                        if variable_height && index % 15 == 0 {
                            vec![row_label, extra_line]
                        } else {
                            vec![row_label]
                        }
                    });
                    container.test_id(Arc::<str>::from(format!(
                        "ui-gallery-virtual-list-row-{index}-label"
                    )))
                });

                cx.virtual_list_keyed_retained_with_layout(
                    list_layout,
                    len,
                    options,
                    &virtual_list_torture_scroll,
                    key_at,
                    row,
                )
            } else {
                cx.virtual_list_keyed_with_layout(
                    list_layout,
                    len,
                    options,
                    &virtual_list_torture_scroll,
                    |i| i as fret_ui::ItemKey,
                    |cx, index| {
                        let zebra = (index % 2) == 0;
                        let background = if zebra {
                            theme.color_required("muted")
                        } else {
                            theme.color_required("background")
                        };

                        let height_hint = if index % 15 == 0 { Px(44.0) } else { Px(28.0) };
                        let row_label = cx.text(format!("Row {index}"));
                        let extra_line = cx.text(format!(
                            "Details: index={index} seed={} repeat={}",
                            index.wrapping_mul(2654435761),
                            (index % 7) + 1
                        ));

                        let mut container_props = decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .bg(ColorRef::Color(background))
                                .p(Space::N2),
                            {
                                let mut layout = LayoutRefinement::default().w_full();
                                if !variable_height {
                                    layout = layout.h_px(MetricRef::Px(height_hint));
                                }
                                layout
                            },
                        );
                        container_props.layout.overflow = fret_ui::element::Overflow::Clip;

                        let container = cx.container(container_props, |_cx| {
                            if variable_height && index % 15 == 0 {
                                vec![row_label, extra_line]
                            } else {
                                vec![row_label]
                            }
                        });
                        container.test_id(Arc::<str>::from(format!(
                            "ui-gallery-virtual-list-row-{index}-label"
                        )))
                    },
                )
            }
        } else if retained_host {
            let theme = theme.clone();
            let edit_row = virtual_list_torture_edit_row.clone();
            let edit_text = virtual_list_torture_edit_text.clone();
            let row_cache = row_cache;

            let key_at = Arc::new(|i| i as fret_ui::ItemKey);
            let row = Arc::new(move |cx: &mut ElementContext<'_, App>, index: usize| {
                let index_u64 = index as u64;
                let row = |cx: &mut ElementContext<'_, App>| {
                    let editing_row = cx
                        .get_model_copied(&edit_row, Invalidation::Layout)
                        .flatten();
                    let is_editing = editing_row == Some(index_u64);

                    let zebra = (index % 2) == 0;
                    let background = if is_editing {
                        theme.color_required("accent")
                    } else if zebra {
                        theme.color_required("muted")
                    } else {
                        theme.color_required("background")
                    };

                    let height_hint = if index % 15 == 0 { Px(44.0) } else { Px(28.0) };

                    let edit_row_for_activate = edit_row.clone();
                    let edit_text_for_activate = edit_text.clone();
                    let on_select_row: fret_ui::action::OnActivate =
                        Arc::new(move |host, action_cx, _reason| {
                            let _ = host
                                .models_mut()
                                .update(&edit_row_for_activate, |v| *v = Some(index_u64));
                            let _ = host.models_mut().update(&edit_text_for_activate, |v| {
                                *v = format!("Row {index_u64}");
                            });
                            host.request_redraw(action_cx.window);
                        });

                    let row_label = shadcn::Button::new(format!("Row {index}"))
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id(format!("ui-gallery-virtual-list-row-{index}-label"))
                        .on_activate(on_select_row.clone())
                        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                        .into_element(cx);

                    let right = if is_editing {
                        let mut props = fret_ui::element::TextInputProps::new(edit_text.clone());
                        props.a11y_label = Some(Arc::<str>::from("Inline edit"));
                        props.test_id =
                            Some(Arc::<str>::from("ui-gallery-virtual-list-edit-input"));
                        props.placeholder = Some(Arc::<str>::from("Type to edit…"));
                        props.layout.size.width = fret_ui::element::Length::Fill;

                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full())
                                .gap(Space::N2)
                                .items_center(),
                            |cx| [cx.text_input(props)],
                        )
                    } else {
                        let edit_button = shadcn::Button::new("Edit")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .test_id(format!("ui-gallery-virtual-list-row-{index}-edit"))
                            .on_activate(on_select_row)
                            .into_element(cx);

                        stack::hstack(
                            cx,
                            stack::HStackProps::default().gap(Space::N2).items_center(),
                            |_cx| [edit_button],
                        )
                    };

                    let mut container_props = decl_style::container_props(
                        &theme,
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(background))
                            .p(Space::N2),
                        LayoutRefinement::default().w_full().h_px(height_hint),
                    );
                    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

                    cx.container(container_props, |cx| {
                        [stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full().h_full())
                                .gap(Space::N2)
                                .items_center(),
                            |_cx| [row_label, right],
                        )]
                    })
                };

                if row_cache {
                    cx.cached_subtree_with(
                        CachedSubtreeProps::default()
                            .contained_layout(true)
                            .cache_key(index_u64),
                        |cx| [row(cx)],
                    )
                } else {
                    row(cx)
                }
            });

            cx.virtual_list_keyed_retained_with_layout(
                list_layout,
                len,
                options,
                &virtual_list_torture_scroll,
                key_at,
                row,
            )
        } else {
            cx.virtual_list_keyed_with_layout(
                list_layout,
                len,
                options,
                &virtual_list_torture_scroll,
                |i| i as fret_ui::ItemKey,
                |cx, index| {
                    let index_u64 = index as u64;
                    let row = |cx: &mut ElementContext<'_, App>| {
                        let editing_row = cx
                            .get_model_copied(&virtual_list_torture_edit_row, Invalidation::Layout)
                            .flatten();
                        let is_editing = editing_row == Some(index_u64);

                        let zebra = (index % 2) == 0;
                        let background = if is_editing {
                            theme.color_required("accent")
                        } else if zebra {
                            theme.color_required("muted")
                        } else {
                            theme.color_required("background")
                        };

                        let height_hint = if index % 15 == 0 { Px(44.0) } else { Px(28.0) };

                        let edit_row_for_activate = virtual_list_torture_edit_row.clone();
                        let edit_text_for_activate = virtual_list_torture_edit_text.clone();
                        let on_select_row: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                let _ = host
                                    .models_mut()
                                    .update(&edit_row_for_activate, |v| *v = Some(index_u64));
                                let _ = host.models_mut().update(&edit_text_for_activate, |v| {
                                    *v = format!("Row {index_u64}");
                                });
                                host.request_redraw(action_cx.window);
                            });
                        let row_label = shadcn::Button::new(format!("Row {index}"))
                            .variant(shadcn::ButtonVariant::Ghost)
                            .size(shadcn::ButtonSize::Sm)
                            .test_id(format!("ui-gallery-virtual-list-row-{index}-label"))
                            .on_activate(on_select_row.clone())
                            .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                            .into_element(cx);

                        let right = if is_editing {
                            let mut props = fret_ui::element::TextInputProps::new(
                                virtual_list_torture_edit_text.clone(),
                            );
                            props.a11y_label = Some(Arc::<str>::from("Inline edit"));
                            props.test_id =
                                Some(Arc::<str>::from("ui-gallery-virtual-list-edit-input"));
                            props.placeholder = Some(Arc::<str>::from("Type to edit…"));
                            props.layout.size.width = fret_ui::element::Length::Fill;

                            stack::hstack(
                                cx,
                                stack::HStackProps::default()
                                    .layout(LayoutRefinement::default().w_full())
                                    .gap(Space::N2)
                                    .items_center(),
                                |cx| vec![cx.text_input(props)],
                            )
                        } else {
                            let edit_button = shadcn::Button::new("Edit")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id(format!("ui-gallery-virtual-list-row-{index}-edit"))
                                .on_activate(on_select_row)
                                .into_element(cx);

                            stack::hstack(
                                cx,
                                stack::HStackProps::default().gap(Space::N2).items_center(),
                                |_cx| vec![edit_button],
                            )
                        };

                        let mut container_props = decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .bg(ColorRef::Color(background))
                                .p(Space::N2),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(height_hint)),
                        );
                        container_props.layout.overflow = fret_ui::element::Overflow::Clip;

                        cx.container(container_props, |cx| {
                            vec![stack::hstack(
                                cx,
                                stack::HStackProps::default()
                                    .layout(LayoutRefinement::default().w_full().h_full())
                                    .gap(Space::N2)
                                    .items_center(),
                                |_cx| vec![row_label, right],
                            )]
                        })
                    };

                    if row_cache {
                        cx.cached_subtree_with(
                            CachedSubtreeProps::default()
                                .contained_layout(true)
                                .cache_key(index_u64),
                            |cx| vec![row(cx)],
                        )
                    } else {
                        row(cx)
                    }
                },
            )
        };

        let list = list.attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::List)
                .test_id("ui-gallery-virtual-list-root"),
        );

        vec![list]
    });

    let root = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3),
        |_cx| vec![header, list],
    );

    let root = root.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-virtual-list-torture-root"),
    );

    vec![root]
}
