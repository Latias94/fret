use super::super::super::*;

pub(in crate::ui) fn preview_intro(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    let card = |cx: &mut ElementContext<'_, App>, title: &str, desc: &str| -> AnyElement {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                .into_element(cx),
            shadcn::CardContent::new(vec![ui::text_block(cx, desc).into_element(cx)])
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
        .into_element(cx)
    };

    let grid = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_stretch(),
        |cx| {
            vec![
                card(
                    cx,
                    "Core",
                    "Window / event / UiTree / renderer contracts (mechanisms & boundaries)",
                ),
                card(
                    cx,
                    "UI Kit",
                    "Headless interaction policies: focus trap, dismiss, hover intent, etc.",
                ),
                card(
                    cx,
                    "Shadcn",
                    "Visual recipes: composed defaults built on the Kit layer",
                ),
            ]
        },
    );
    let grid = grid.attach_semantics(
        SemanticsDecoration::default()
            .label("Debug:ui-gallery:intro:preview-grid")
            .test_id("ui-gallery-intro-preview-grid"),
    );

    let note = {
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("muted")))
                .rounded(Radius::Md)
                .p(Space::N4),
            LayoutRefinement::default().w_full().min_w_0(),
        );
        cx.container(props, |cx| {
            vec![ui::text_block(cx, "Phase 1: fixed two-pane layout + hardcoded docs strings (focus on validating component usability). Docking/multi-window views will come later.").into_element(cx)]
        })
    };
    let note = note.attach_semantics(
        SemanticsDecoration::default()
            .label("Debug:ui-gallery:intro:preview-note")
            .test_id("ui-gallery-intro-preview-note"),
    );

    vec![grid, note]
}

pub(in crate::ui) fn preview_hit_test_only_paint_cache_probe(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui::element::SemanticsProps;

    fn with_alpha(mut color: CoreColor, alpha: f32) -> CoreColor {
        color.a = alpha;
        color
    }

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: deterministically trigger HitTestOnly invalidation on a cache-eligible subtree."),
                cx.text("Pointer moves over the probe region call `host.invalidate(Invalidation::HitTestOnly)` while layout and painted content remain stable."),
                cx.text("Use this page to validate `paint_cache_hit_test_only_replay_*` counters."),
            ]
        },
    );

    let panel = cx
        .semantics_with_id(
            SemanticsProps {
                role: fret_core::SemanticsRole::Panel,
                label: Some(Arc::from("ui-gallery-hit-test-only-probe-region")),
                ..Default::default()
            },
            move |cx, _id| {
                let on_move: fret_ui::action::OnPointerMove =
                    Arc::new(move |host, action_cx, _mv| {
                        host.invalidate(fret_ui::Invalidation::HitTestOnly);
                        host.request_redraw(action_cx.window);
                        true
                    });

                let mut pointer = fret_ui::element::PointerRegionProps::default();
                pointer.layout.size.width = fret_ui::element::Length::Fill;
                pointer.layout.size.height = fret_ui::element::Length::Fill;
                pointer.layout.overflow = fret_ui::element::Overflow::Clip;

                let mut canvas = CanvasProps::default();
                canvas.layout.size.width = fret_ui::element::Length::Fill;
                canvas.layout.size.height = fret_ui::element::Length::Fill;
                canvas.layout.overflow = fret_ui::element::Overflow::Clip;
                canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

                let region = cx.pointer_region(pointer, move |cx| {
                    cx.pointer_region_on_pointer_move(on_move.clone());

                    vec![
                        cx.container(
                            decl_style::container_props(
                                theme,
                                ChromeRefinement::default()
                                    .border_1()
                                    .rounded(Radius::Md)
                                    .bg(ColorRef::Color(theme.color_required("background"))),
                                LayoutRefinement::default()
                                    .w_full()
                                    .h_px(MetricRef::Px(Px(320.0))),
                            ),
                            move |cx| {
                                vec![
                                    cx.canvas(canvas, move |p| {
                                        let bounds = p.bounds();
                                        let accent_bg =
                                            with_alpha(p.theme().color_required("accent"), 0.10);
                                        let border_color = p.theme().color_required("border");
                                        let secondary_bg =
                                            with_alpha(p.theme().color_required("secondary"), 0.16);
                                        let muted_border = with_alpha(
                                            p.theme().color_required("muted-foreground"),
                                            0.35,
                                        );

                                        p.scene().push(SceneOp::Quad {
                                            order: DrawOrder(0),
                                            rect: bounds,
                                            background: fret_core::Paint::Solid(accent_bg),

                                            border: Edges::all(Px(1.0)),
                                            border_paint: fret_core::Paint::Solid(border_color),
                                            corner_radii: Corners::all(Px(8.0)),
                                        });

                                        let guide = Rect::new(
                                            Point::new(
                                                Px(bounds.origin.x.0 + 48.0),
                                                Px(bounds.origin.y.0 + 36.0),
                                            ),
                                            Size::new(
                                                Px((bounds.size.width.0 - 96.0).max(0.0)),
                                                Px((bounds.size.height.0 - 72.0).max(0.0)),
                                            ),
                                        );
                                        p.scene().push(SceneOp::Quad {
                                            order: DrawOrder(0),
                                            rect: guide,
                                            background: fret_core::Paint::Solid(secondary_bg),

                                            border: Edges::all(Px(1.0)),
                                            border_paint: fret_core::Paint::Solid(muted_border),

                                            corner_radii: Corners::all(Px(6.0)),
                                        });
                                    })
                                    .test_id("ui-gallery-hit-test-only-probe-canvas"),
                                ]
                            },
                        )
                        .test_id("ui-gallery-hit-test-only-probe-region"),
                    ]
                });

                vec![region]
            },
        )
        .test_id("ui-gallery-hit-test-only-probe-region");

    vec![header, panel]
}
pub(in crate::ui) fn preview_view_cache(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    view_cache_enabled: Model<bool>,
    view_cache_cache_shell: Model<bool>,
    view_cache_inner_enabled: Model<bool>,
    view_cache_popover_open: Model<bool>,
    view_cache_continuous: Model<bool>,
    view_cache_counter: Model<u64>,
    text_input: Model<String>,
    text_area: Model<String>,
) -> Vec<AnyElement> {
    let enabled = cx
        .get_model_copied(&view_cache_enabled, Invalidation::Layout)
        .unwrap_or(false);
    let cache_shell = cx
        .get_model_copied(&view_cache_cache_shell, Invalidation::Layout)
        .unwrap_or(false);
    let cache_inner = cx
        .get_model_copied(&view_cache_inner_enabled, Invalidation::Layout)
        .unwrap_or(true);
    let continuous = cx
        .get_model_copied(&view_cache_continuous, Invalidation::Layout)
        .unwrap_or(false);

    let toggles = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2).items_start(),
        |cx| {
            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Switch::new(view_cache_enabled.clone())
                                .a11y_label("Enable view-cache mode")
                                .test_id("ui-gallery-view-cache-enabled")
                                .into_element(cx),
                            cx.text("Enable view-cache mode (global UiTree flag)"),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Switch::new(view_cache_cache_shell.clone())
                                .a11y_label("Cache the gallery shell")
                                .test_id("ui-gallery-view-cache-cache-shell")
                                .into_element(cx),
                            cx.text("Cache shell (sidebar/content wrappers)"),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Switch::new(view_cache_inner_enabled.clone())
                                .a11y_label("Enable inner ViewCache boundary")
                                .test_id("ui-gallery-view-cache-inner-cache")
                                .into_element(cx),
                            cx.text("Enable inner ViewCache boundary (torture subtree)"),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Switch::new(view_cache_continuous.clone())
                                .a11y_label("Request continuous frames")
                                .test_id("ui-gallery-view-cache-continuous")
                                .into_element(cx),
                            cx.text("Continuous frames (cache-hit should still keep state alive)"),
                        ]
                    },
                ),
            ]
        },
    );

    let actions = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Bump counter")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-view-cache-bump-counter")
                    .on_click(CMD_VIEW_CACHE_BUMP)
                    .into_element(cx),
                shadcn::Button::new("Reset counter")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-view-cache-reset-counter")
                    .on_click(CMD_VIEW_CACHE_RESET)
                    .into_element(cx),
            ]
        },
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3),
        |cx| {
            vec![
                cx.text("Goal: validate cached-subtree correctness under real interaction."),
                cx.text(format!(
                    "Current settings: view_cache={} shell_cache={} inner_cache={} continuous={}",
                    enabled as u8, cache_shell as u8, cache_inner as u8, continuous as u8
                )),
                toggles,
                actions,
            ]
        },
    );

    let subtree_body = |cx: &mut ElementContext<'_, App>| -> Vec<AnyElement> {
        let render_count = cx.with_state(
            || 0u64,
            |v| {
                *v = v.saturating_add(1);
                *v
            },
        );

        let mut needs_lease = false;
        cx.with_state(
            || None::<ContinuousFrames>,
            |lease| {
                if continuous {
                    if lease.is_none() {
                        needs_lease = true;
                    }
                } else {
                    *lease = None;
                }
            },
        );
        if needs_lease {
            let lease = cx.begin_continuous_frames();
            cx.with_state(
                || None::<ContinuousFrames>,
                |slot| {
                    *slot = Some(lease);
                },
            );
        }

        let counter = cx
            .get_model_copied(&view_cache_counter, Invalidation::Layout)
            .unwrap_or(0);

        let input = shadcn::Input::new(text_input.clone())
            .a11y_label("Cached input")
            .placeholder("Type to invalidate the cache root")
            .into_element(cx);
        let textarea = shadcn::Textarea::new(text_area.clone())
            .a11y_label("Cached textarea")
            .into_element(cx);

        let popover = shadcn::Popover::new(view_cache_popover_open.clone())
            .auto_focus(true)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Popover (cached trigger)")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-view-cache-popover-trigger")
                        .toggle_model(view_cache_popover_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new([
                        cx.text("Popover content"),
                        shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .test_id("ui-gallery-view-cache-popover-close")
                            .toggle_model(view_cache_popover_open.clone())
                            .into_element(cx),
                    ])
                    .into_element(cx)
                },
            );

        let mut rows: Vec<AnyElement> = Vec::new();
        rows.reserve(240);
        for i in 0..240u32 {
            rows.push(cx.keyed(i, |cx| {
                shadcn::Button::new(format!("Row {i}"))
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
            }));
        }

        let list = shadcn::ScrollArea::new([stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N1),
            |_cx| rows,
        )])
        .refine_layout(LayoutRefinement::default().w_full().h_px(Px(280.0)))
        .into_element(cx);

        vec![
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![
                    shadcn::CardTitle::new("Cached subtree").into_element(cx),
                    shadcn::CardDescription::new(format!(
                        "render_count={} counter={}",
                        render_count, counter
                    ))
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::CardContent::new(vec![input, textarea, popover, list]).into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        ]
    };

    let subtree = if cache_inner {
        cx.cached_subtree(subtree_body)
    } else {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Uncached subtree").into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(subtree_body(cx)).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
    };

    let root = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3),
        move |cx| {
            vec![
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("View Cache Torture").into_element(cx),
                        shadcn::CardDescription::new(
                            "Compare cached vs uncached subtree execution and state retention.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![header]).into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
                subtree,
                cx.text_props(TextProps {
                    layout: Default::default(),
                    text: Arc::from(
                        "Tip: keep 'Cache shell' off while iterating so the status bar updates every frame.",
                    ),
                    style: None,
                    color: Some(theme.color_required("muted-foreground")),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                }),
            ]
        },
    )
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Generic)
            .test_id("ui-gallery-view-cache-root"),
    );

    vec![root]
}

pub(in crate::ui) fn preview_layout(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    let boxy = |cx: &mut ElementContext<'_, App>, label: &str, color: fret_core::Color| {
        cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(color))
                    .rounded(Radius::Md)
                    .p(Space::N3),
                // In a horizontal flex row, we want "equal columns" semantics (`flex-1`), not
                // `w-full` (percent sizing). Percent sizing is fragile under intrinsic sizing
                // probes and can cause transient wrap widths (0px) to leak into final layout.
                LayoutRefinement::default().flex_1().min_w_0(),
            ),
            |cx| [ui::label(cx, label).w_full().into_element(cx)],
        )
    };

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_stretch(),
        |cx| {
            vec![
                boxy(cx, "Left (fill)", theme.color_required("accent")),
                boxy(cx, "Center (fill)", theme.color_required("muted")),
                boxy(cx, "Right (fill)", theme.color_required("card")),
            ]
        },
    );

    vec![
        ui::text_block(
            cx,
            "Layout mental model: LayoutRefinement (constraints) + stack (composition) + Theme tokens (color/spacing).",
        )
        .into_element(cx),
        row,
    ]
}

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

#[derive(Default)]
struct UiKitListTortureModels {
    selection: Option<Model<Option<usize>>>,
}

pub(in crate::ui) fn preview_ui_kit_list_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let selection = cx.with_state(UiKitListTortureModels::default, |st| st.selection.clone());
    let selection = match selection {
        Some(selection) => selection,
        None => {
            let selection = cx.app.models_mut().insert(Option::<usize>::None);
            cx.with_state(UiKitListTortureModels::default, |st| {
                st.selection = Some(selection.clone());
            });
            selection
        }
    };

    let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text(
                    "Goal: validate fret-ui-kit list virtualization under view-cache + shell reuse (ADR 0177).",
                ),
                cx.text("Expect: scroll boundary shifts reconcile without scroll-window dirty views."),
            ]
        },
    );

    let len: usize = std::env::var("FRET_UI_GALLERY_UI_KIT_LIST_LEN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10_000)
        .clamp(16, 200_000);
    let overscan: usize = 6;

    let list = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        vec![
            fret_ui_kit::declarative::list::list_virtualized_copyable_retained_v0(
                cx,
                selection,
                fret_ui_kit::Size::Medium,
                None,
                len,
                overscan,
                &scroll_handle,
                0,
                |i| i as u64,
                Arc::new(|_models, i| Some(format!("Item {i}"))),
                |_i| None,
                |cx, i| {
                    let mut out = Vec::new();
                    let label = cx.text(format!("Item {i}"));
                    let label = if i == 0 {
                        label.attach_semantics(
                            SemanticsDecoration::default()
                                .test_id("ui-gallery-ui-kit-list-row-0-label"),
                        )
                    } else {
                        label
                    };
                    out.push(label);
                    out.push(cx.spacer(fret_ui::element::SpacerProps {
                        min: Px(0.0),
                        ..Default::default()
                    }));
                    out
                },
            ),
        ]
    });

    let list = list.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::List)
            .test_id("ui-gallery-ui-kit-list-root"),
    );

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
            .test_id("ui-gallery-ui-kit-list-torture-root"),
    );

    vec![root]
}
