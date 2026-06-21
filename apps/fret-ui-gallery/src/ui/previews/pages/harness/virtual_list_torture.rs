use super::super::super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use fret::AppComponentCx;

fn virtual_list_row_label_test_id(index: usize) -> Arc<str> {
    Arc::<str>::from(format!("ui-gallery-virtual-list-row-{index}-label"))
}

fn virtual_list_row_label_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement
where
    T: Into<Arc<str>>,
{
    fret_ui_kit::declarative::text::text_list_row_label(cx, text)
}

fn with_alpha(mut color: CoreColor, alpha: f32) -> CoreColor {
    color.a = alpha;
    color
}

fn virtual_list_row_detail_text<T>(cx: &mut AppComponentCx<'_>, text: T) -> AnyElement
where
    T: Into<Arc<str>>,
{
    doc_layout::control_readout_text(cx, text)
}

fn virtual_list_row_semantics(index: usize, len: usize) -> SemanticsDecoration {
    let mut decoration = SemanticsDecoration::default()
        .role(fret_core::SemanticsRole::ListItem)
        .label(format!("Row {index}"))
        .test_id(Arc::<str>::from(format!(
            "ui-gallery-virtual-list-row-{index}"
        )));

    if let (Ok(pos_in_set), Ok(set_size)) =
        (u32::try_from(index.saturating_add(1)), u32::try_from(len))
    {
        decoration = decoration.collection_position(pos_in_set, set_size);
    }

    decoration
}

fn virtual_list_selected_row_semantics(
    index: usize,
    len: usize,
    selected: bool,
) -> SemanticsDecoration {
    virtual_list_row_semantics(index, len)
        .selected(selected)
        .invokable(true)
}

fn virtual_list_row_content(
    cx: &mut AppComponentCx<'_>,
    theme: &Theme,
    row_label: AnyElement,
    right: AnyElement,
) -> AnyElement {
    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = fret_ui::element::Length::Fill;
    layout.size.height = fret_ui::element::Length::Fill;

    cx.flex(
        fret_ui::element::FlexProps {
            layout,
            direction: fret_core::Axis::Horizontal,
            gap: fret_ui::element::SpacingLength::Px(
                fret_ui_kit::MetricRef::space(Space::N2).resolve(theme),
            ),
            padding: fret_core::Edges::all(Px(0.0)).into(),
            justify: fret_ui::element::MainAlign::Start,
            align: fret_ui::element::CrossAlign::Center,
            wrap: false,
        },
        |_cx| [row_label, right],
    )
}

// Keeps row action chrome button-like without paying for the full shadcn Button slot tree.
fn virtual_list_row_action_button<T, I>(
    cx: &mut AppComponentCx<'_>,
    theme: &Theme,
    label: T,
    test_id: I,
    variant: shadcn::ButtonVariant,
    layout: LayoutRefinement,
    text_fill: bool,
    on_activate: fret_ui::action::OnActivate,
) -> AnyElement
where
    T: Into<Arc<str>>,
    I: Into<Arc<str>>,
{
    let label = label.into();
    let test_id = test_id.into();
    let variants = shadcn::button_variants(&theme.snapshot(), variant, shadcn::ButtonSize::Sm);
    let mut chrome = variants.chrome.px(Space::N3).py(Space::N1);
    if variant == shadcn::ButtonVariant::Outline {
        chrome = chrome.shadow_xs();
    }

    let pressable_layout = decl_style::layout_style(theme, variants.layout.merge(layout));
    let chrome_props = decl_style::container_props(theme, chrome, LayoutRefinement::default());
    let focus_ring = decl_style::focus_ring(theme, decl_style::radius(theme, Radius::Md));

    fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props(
        cx,
        move |cx, _state, _id| {
            cx.pressable_on_activate(on_activate.clone());
            let text = if text_fill {
                fret_ui_kit::declarative::text::text_button_label_fill(cx, label.clone())
            } else {
                fret_ui_kit::declarative::text::text_button_label(cx, label.clone())
            };

            (
                fret_ui::element::PressableProps {
                    layout: pressable_layout,
                    enabled: true,
                    focusable: true,
                    focus_ring: Some(focus_ring),
                    key_activation: fret_ui::element::PressableKeyActivation::EnterAndSpace,
                    a11y: fret_ui::element::PressableA11y {
                        role: Some(fret_core::SemanticsRole::Button),
                        label: Some(label.clone()),
                        test_id: Some(test_id),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                chrome_props,
                move |_cx| [text],
            )
        },
    )
}

pub(in crate::ui) fn preview_virtual_list_torture(
    cx: &mut AppComponentCx<'_>,
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
        let jump_input = shadcn::Input::new(virtual_list_torture_jump.clone())
            .a11y_label("Jump to row")
            .test_id("ui-gallery-virtual-list-jump-input")
            .placeholder("Row index (e.g. 9000)")
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

        doc_layout::wrap_controls_row(cx, theme, Space::N2, |cx| {
            let jump_model = virtual_list_torture_jump.clone();
            let scroll_for_jump = virtual_list_torture_scroll.clone();
            let on_jump: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let raw = host
                    .models_mut()
                    .get_cloned(&jump_model)
                    .unwrap_or_default();
                let index = raw.trim().parse::<usize>().unwrap_or(0);
                scroll_for_jump.scroll_to_item(index, fret_ui::scroll::ScrollStrategy::Start);
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
        })
        .into_element(cx)
    });

    let editing_indicator = (!minimal_harness).then(|| {
        let label = if let Some(row) = header_editing_row {
            Arc::<str>::from(format!("editing_row={row}"))
        } else {
            Arc::<str>::from("editing_row=<none>")
        };

        let text = if let Some(row) = header_editing_row {
            virtual_list_row_detail_text(cx, format!("Editing row: {row}"))
        } else {
            virtual_list_row_detail_text(cx, "Editing row: <none>")
        };
        text.attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Text)
                .label(label)
                .test_id("ui-gallery-virtual-list-editing"),
        )
    });

    let header = ui::v_flex(|cx| {
            let mut out = vec![
                doc_layout::paragraph_text(cx, "Goal: deterministic virtualization torture surface (10k rows + scroll-to-item + inline edit)."),
                doc_layout::control_readout_text(cx, if retained_host {
                    "Mode: retained host (virt-003 prototype; item subtrees can reattach without rerendering the parent cache root)."
                } else {
                    "Mode: render-driven (baseline; visible items update requires rerender when the window changes)."
                }),
                doc_layout::control_readout_text(cx, if known_heights {
                    "Mode: known row heights (no measure pass; better for perf baselines)."
                } else {
                    "Mode: measured row heights (baseline)."
                }),
                doc_layout::control_readout_text(cx, if keep_alive > 0 {
                    format!("Mode: keep-alive enabled (budget={keep_alive}).")
                } else {
                    "Mode: keep-alive disabled (budget=0).".to_string()
                }),
            ];

            if minimal_harness {
                out.push(doc_layout::paragraph_text(cx, "Harness: minimal (no focusable controls; reduces RAF/notify noise in perf bundles)."));
            } else {
                if let Some(controls) = controls {
                    out.push(controls);
                }
                if let Some(editing_indicator) = editing_indicator {
                    out.push(editing_indicator);
                }
            }

            out
        })
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2).into_element(cx);

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

    let list = cx.cached_subtree_with(
        CachedSubtreeProps::default()
            .layout(list_layout)
            .contain_layout_when_bounds_known(true),
        |cx| {
            let list = if minimal_harness {
                if retained_host {
                    let theme = theme.clone();
                    let key_at = Arc::new(|i| i as fret_ui::ItemKey);
                    let row = Arc::new(move |cx: &mut AppComponentCx<'_>, index: usize| {
                        let zebra = (index % 2) == 0;
                        let background = if zebra {
                            theme.color_token("muted")
                        } else {
                            theme.color_token("background")
                        };

                        let height_hint = if index % 15 == 0 { Px(44.0) } else { Px(28.0) };
                        let row_label = virtual_list_row_label_text(cx, format!("Row {index}"))
                            .test_id(virtual_list_row_label_test_id(index));
                        let extra_line = virtual_list_row_detail_text(
                            cx,
                            format!(
                                "Details: index={index} seed={} repeat={}",
                                index.wrapping_mul(2654435761),
                                (index % 7) + 1
                            ),
                        );

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

                        cx.container(container_props, |_cx| {
                            if variable_height && index % 15 == 0 {
                                vec![row_label, extra_line]
                            } else {
                                vec![row_label]
                            }
                        })
                        .attach_semantics(virtual_list_row_semantics(index, len))
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
                                theme.color_token("muted")
                            } else {
                                theme.color_token("background")
                            };

                            let height_hint = if index % 15 == 0 { Px(44.0) } else { Px(28.0) };
                            let row_label = virtual_list_row_label_text(cx, format!("Row {index}"))
                                .test_id(virtual_list_row_label_test_id(index));
                            let extra_line = virtual_list_row_detail_text(
                                cx,
                                format!(
                                    "Details: index={index} seed={} repeat={}",
                                    index.wrapping_mul(2654435761),
                                    (index % 7) + 1
                                ),
                            );

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

                            cx.container(container_props, |_cx| {
                                if variable_height && index % 15 == 0 {
                                    vec![row_label, extra_line]
                                } else {
                                    vec![row_label]
                                }
                            })
                            .attach_semantics(virtual_list_row_semantics(index, len))
                        },
                    )
                }
            } else if retained_host {
                let theme = theme.clone();
                let edit_row = virtual_list_torture_edit_row.clone();
                let edit_text = virtual_list_torture_edit_text.clone();
                let row_cache = row_cache;

                let key_at = Arc::new(|i| i as fret_ui::ItemKey);
                let row = Arc::new(move |cx: &mut AppComponentCx<'_>, index: usize| {
                    let index_u64 = index as u64;
                    let row = |cx: &mut AppComponentCx<'_>| {
                        let editing_row = cx
                            .get_model_copied(&edit_row, Invalidation::Layout)
                            .flatten();
                        let is_editing = editing_row == Some(index_u64);

                        let zebra = (index % 2) == 0;
                        let background = if is_editing {
                            theme.color_token("accent")
                        } else if zebra {
                            theme.color_token("muted")
                        } else {
                            theme.color_token("background")
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

                        let row_label = virtual_list_row_action_button(
                            cx,
                            &theme,
                            format!("Row {index}"),
                            format!("ui-gallery-virtual-list-row-{index}-label"),
                            shadcn::ButtonVariant::Ghost,
                            LayoutRefinement::default().flex_1().min_w_0(),
                            true,
                            on_select_row.clone(),
                        );

                        let right = if is_editing {
                            shadcn::Input::new(edit_text.clone())
                                .a11y_label("Inline edit")
                                .test_id("ui-gallery-virtual-list-edit-input")
                                .placeholder("Type to edit…")
                                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                                .into_element(cx)
                        } else {
                            virtual_list_row_action_button(
                                cx,
                                &theme,
                                "Edit",
                                format!("ui-gallery-virtual-list-row-{index}-edit"),
                                shadcn::ButtonVariant::Outline,
                                LayoutRefinement::default(),
                                false,
                                on_select_row,
                            )
                        };

                        let border_color = is_editing
                            .then(|| theme.color_token("ring"))
                            .unwrap_or_else(|| with_alpha(theme.color_token("border"), 0.55));
                        let mut chrome = ChromeRefinement::default()
                            .bg(ColorRef::Color(background))
                            .border_1()
                            .border_color(ColorRef::Color(border_color))
                            .p(Space::N2);
                        if is_editing {
                            chrome = chrome.text_color(ColorRef::Token {
                                key: "accent-foreground",
                                fallback: fret_ui_kit::ColorFallback::ThemeTextPrimary,
                            });
                        }

                        let mut container_props = decl_style::container_props(
                            &theme,
                            chrome,
                            LayoutRefinement::default().w_full().h_px(height_hint),
                        );
                        container_props.layout.overflow = fret_ui::element::Overflow::Clip;

                        let row = cx.container(container_props, |cx| {
                            [virtual_list_row_content(cx, &theme, row_label, right)]
                        });

                        row.attach_semantics(virtual_list_selected_row_semantics(
                            index, len, is_editing,
                        ))
                    };

                    if row_cache {
                        let selected_for_key =
                            cx.app.models().get_copied(&edit_row).unwrap_or_default()
                                == Some(index_u64);
                        cx.cached_subtree_with(
                            CachedSubtreeProps::default()
                                .contain_layout_when_bounds_known(true)
                                .cache_key(fret_ui::cache_key::mix(
                                    index_u64,
                                    u64::from(selected_for_key),
                                )),
                            |cx| {
                                let row = row(cx);
                                [row]
                            },
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
                        let row = |cx: &mut AppComponentCx<'_>| {
                            let editing_row = cx
                                .get_model_copied(
                                    &virtual_list_torture_edit_row,
                                    Invalidation::Layout,
                                )
                                .flatten();
                            let is_editing = editing_row == Some(index_u64);

                            let zebra = (index % 2) == 0;
                            let background = if is_editing {
                                theme.color_token("accent")
                            } else if zebra {
                                theme.color_token("muted")
                            } else {
                                theme.color_token("background")
                            };

                            let height_hint = if index % 15 == 0 { Px(44.0) } else { Px(28.0) };

                            let edit_row_for_activate = virtual_list_torture_edit_row.clone();
                            let edit_text_for_activate = virtual_list_torture_edit_text.clone();
                            let on_select_row: fret_ui::action::OnActivate =
                                Arc::new(move |host, action_cx, _reason| {
                                    let _ = host
                                        .models_mut()
                                        .update(&edit_row_for_activate, |v| *v = Some(index_u64));
                                    let _ =
                                        host.models_mut().update(&edit_text_for_activate, |v| {
                                            *v = format!("Row {index_u64}");
                                        });
                                    host.request_redraw(action_cx.window);
                                });
                            let row_label = virtual_list_row_action_button(
                                cx,
                                theme,
                                format!("Row {index}"),
                                format!("ui-gallery-virtual-list-row-{index}-label"),
                                shadcn::ButtonVariant::Ghost,
                                LayoutRefinement::default().flex_1().min_w_0(),
                                true,
                                on_select_row.clone(),
                            );

                            let right = if is_editing {
                                shadcn::Input::new(virtual_list_torture_edit_text.clone())
                                    .a11y_label("Inline edit")
                                    .test_id("ui-gallery-virtual-list-edit-input")
                                    .placeholder("Type to edit…")
                                    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                                    .into_element(cx)
                            } else {
                                virtual_list_row_action_button(
                                    cx,
                                    theme,
                                    "Edit",
                                    format!("ui-gallery-virtual-list-row-{index}-edit"),
                                    shadcn::ButtonVariant::Outline,
                                    LayoutRefinement::default(),
                                    false,
                                    on_select_row,
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
                                vec![virtual_list_row_content(cx, theme, row_label, right)]
                            })
                            .attach_semantics(virtual_list_row_semantics(index, len))
                        };

                        if row_cache {
                            cx.cached_subtree_with(
                                CachedSubtreeProps::default()
                                    .contain_layout_when_bounds_known(true)
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
        },
    );

    let root = ui::v_flex(|_cx| vec![header, list])
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N3)
        .into_element(cx);

    let root = root.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-virtual-list-torture-root"),
    );

    let harness = DocSection::build(cx, "Harness", root)
        .no_shell()
        .max_w(Px(980.0));

    let page = doc_layout::render_doc_page(
        cx,
        Some(
            "Deterministic virtualization torture surface (10k rows + scroll-to-item + inline edit).",
        ),
        vec![harness],
    );

    vec![page.into_element(cx)]
}
