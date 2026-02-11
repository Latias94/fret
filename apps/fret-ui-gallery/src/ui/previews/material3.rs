use super::super::*;

pub(in crate::ui) fn material3_scoped_page<I, F>(
    cx: &mut ElementContext<'_, App>,
    material3_expressive: Model<bool>,
    content: F,
) -> Vec<AnyElement>
where
    F: FnOnce(&mut ElementContext<'_, App>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    let enabled = cx
        .get_model_copied(&material3_expressive, Invalidation::Layout)
        .unwrap_or(false);

    let mut out: Vec<AnyElement> = Vec::new();
    out.push(material3_variant_toggle_row(cx, material3_expressive));

    let body = if enabled {
        material3::context::with_material_design_variant(
            cx,
            material3::MaterialDesignVariant::Expressive,
            content,
        )
    } else {
        content(cx)
    };

    out.extend(body);
    out
}

pub(in crate::ui) fn material3_variant_toggle_row(
    cx: &mut ElementContext<'_, App>,
    material3_expressive: Model<bool>,
) -> AnyElement {
    let enabled = cx
        .get_model_copied(&material3_expressive, Invalidation::Layout)
        .unwrap_or(false);

    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            vec![
                shadcn::Switch::new(material3_expressive.clone())
                    .a11y_label("Enable Material 3 Expressive variant")
                    .into_element(cx),
                ui::label(
                    cx,
                    if enabled {
                        "Variant: Expressive"
                    } else {
                        "Variant: Standard"
                    },
                )
                .into_element(cx),
            ]
        },
    )
}

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

pub(in crate::ui) fn code_view_torture_source() -> Arc<str> {
    static SOURCE: OnceLock<Arc<str>> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            let mut out = String::new();
            out.push_str("// Code View Torture Harness\n");
            out.push_str("// Generated content: large line count + long lines\n\n");
            for i in 0..8_000 {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "{i:05}: fn example_{i}() {{ let x = {i}; let y = x.wrapping_mul(31); }}\n"
                    ),
                );
            }
            Arc::<str>::from(out)
        })
        .clone()
}

pub(in crate::ui) fn preview_code_view_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress large scrollable code/text surfaces (candidate for prepaint-windowed lines)."),
                cx.text("Use scripted wheel steps + stale-paint checks to validate scroll stability."),
            ]
        },
    );

    let code = code_view_torture_source();

    let windowed =
        match std::env::var_os("FRET_UI_GALLERY_CODE_VIEW_WINDOWED").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => true,
        };

    let block = code_view::CodeBlock::new(code)
        .language("rust")
        .show_line_numbers(true)
        .windowed_lines(windowed)
        .show_scrollbar_y(true)
        .max_height(Px(420.0));
    let block = block.into_element(cx);

    let block = block.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-code-view-root"),
    );

    vec![header, block]
}

pub(in crate::ui) fn code_editor_mvp_source() -> String {
    [
        "// Code Editor MVP\n",
        "// Goals:\n",
        "// - Validate TextInputRegion focus + TextInput/Ime events\n",
        "// - Validate nested scrolling (editor owns its own scroll)\n",
        "// - Provide a base surface for code-editor-ecosystem-v1 workstream\n",
        "\n",
        "fn main() {\n",
        "    let mut sum = 0u64;\n",
        "    for i in 0..10_000 {\n",
        "        sum = sum.wrapping_add(i);\n",
        "    }\n",
        "    println!(\"sum={}\", sum);\n",
        "}\n",
        "\n",
        "struct Point { x: f32, y: f32 }\n",
        "\n",
        "impl Point {\n",
        "    fn len(&self) -> f32 {\n",
        "        (self.x * self.x + self.y * self.y).sqrt()\n",
        "    }\n",
        "}\n",
        "\n",
        "// Try: mouse drag selection, Ctrl+C/Ctrl+V, arrows, Backspace/Delete, IME.\n",
    ]
    .concat()
}

pub(in crate::ui) fn code_editor_torture_source() -> String {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            let mut out = String::new();
            out.push_str("// Code Editor Torture Harness\n");
            out.push_str("// Generated content: many lines + deterministic prefixes\n\n");
            for i in 0..20_000usize {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "{i:05}: let value_{i} = {i}; // scrolling should never show stale lines\n"
                    ),
                );
            }
            out
        })
        .clone()
}

pub(in crate::ui) fn code_editor_word_boundary_fixture() -> String {
    [
        "// Word boundary fixture (UI Gallery)\n",
        "\n",
        "世界 hello 😀 foo123_bar baz foo.bar\n",
        "a_b c\t  hello   world\n",
        "αβγ δ\n",
    ]
    .concat()
}

pub(in crate::ui) fn format_word_boundary_debug(text: &str, idx: usize) -> String {
    let idx = code_editor_view::clamp_to_char_boundary(text, idx).min(text.len());
    fn move_n_chars_left(text: &str, mut idx: usize, n: usize) -> usize {
        for _ in 0..n {
            let prev = code_editor_view::prev_char_boundary(text, idx);
            if prev == idx {
                break;
            }
            idx = prev;
        }
        idx
    }

    fn move_n_chars_right(text: &str, mut idx: usize, n: usize) -> usize {
        for _ in 0..n {
            let next = code_editor_view::next_char_boundary(text, idx);
            if next == idx {
                break;
            }
            idx = next;
        }
        idx
    }

    fn sanitize_inline(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for ch in s.chars() {
            match ch {
                '\n' => out.push('⏎'),
                '\t' => out.push('⇥'),
                '\r' => out.push('␍'),
                _ => out.push(ch),
            }
        }
        out
    }

    let ctx_start = move_n_chars_left(text, idx, 16);
    let ctx_end = move_n_chars_right(text, idx, 16);
    let ctx_start = code_editor_view::clamp_to_char_boundary(text, ctx_start).min(text.len());
    let ctx_end = code_editor_view::clamp_to_char_boundary(text, ctx_end).min(text.len());
    let ctx_before = sanitize_inline(text.get(ctx_start..idx).unwrap_or(""));
    let ctx_after = sanitize_inline(text.get(idx..ctx_end).unwrap_or(""));
    let caret_ch = text.get(idx..).and_then(|s| s.chars().next());
    let caret_ch = caret_ch.map(|c| sanitize_inline(&c.to_string()));

    let unicode = fret_runtime::TextBoundaryMode::UnicodeWord;
    let ident = fret_runtime::TextBoundaryMode::Identifier;

    let (u_a, u_b) = code_editor_view::select_word_range(text, idx, unicode);
    let (i_a, i_b) = code_editor_view::select_word_range(text, idx, ident);

    let u_l = code_editor_view::move_word_left(text, idx, unicode);
    let u_r = code_editor_view::move_word_right(text, idx, unicode);
    let i_l = code_editor_view::move_word_left(text, idx, ident);
    let i_r = code_editor_view::move_word_right(text, idx, ident);

    [
        format!(
            "idx={idx} caret_char={}",
            caret_ch.as_deref().unwrap_or("<eof>")
        ),
        format!("context: {ctx_before}|{ctx_after}"),
        format!("UnicodeWord: select={u_a}..{u_b} left={u_l} right={u_r}"),
        format!("Identifier: select={i_a}..{i_b} left={i_l} right={i_r}"),
    ]
    .join("\n")
}

pub(in crate::ui) fn preview_code_editor_mvp(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    syntax_rust: Model<bool>,
    boundary_identifier: Model<bool>,
    soft_wrap: Model<bool>,
) -> Vec<AnyElement> {
    let syntax_enabled = cx
        .get_model_copied(&syntax_rust, Invalidation::Layout)
        .unwrap_or(false);
    let boundary_identifier_enabled = cx
        .get_model_copied(&boundary_identifier, Invalidation::Layout)
        .unwrap_or(true);
    let soft_wrap_enabled = cx
        .get_model_copied(&soft_wrap, Invalidation::Layout)
        .unwrap_or(false);

    #[derive(Clone)]
    struct CodeEditorMvpHandles {
        main: code_editor::CodeEditorHandle,
        word_fixture: code_editor::CodeEditorHandle,
        word_gate: code_editor::CodeEditorHandle,
        word_gate_soft_wrap: code_editor::CodeEditorHandle,
        a11y_selection_gate: code_editor::CodeEditorHandle,
        a11y_composition_gate: code_editor::CodeEditorHandle,
        a11y_selection_wrap_gate: code_editor::CodeEditorHandle,
        a11y_composition_wrap_gate: code_editor::CodeEditorHandle,
        a11y_composition_drag_gate: code_editor::CodeEditorHandle,
    }

    fn code_editor_wrap_gate_fixture() -> String {
        let mut s = String::new();
        for _ in 0..20 {
            s.push_str("0123456789");
        }
        s
    }

    let handles = cx.with_state(
        || CodeEditorMvpHandles {
            main: code_editor::CodeEditorHandle::new(code_editor_mvp_source()),
            word_fixture: code_editor::CodeEditorHandle::new(code_editor_word_boundary_fixture()),
            word_gate: code_editor::CodeEditorHandle::new("can't"),
            word_gate_soft_wrap: code_editor::CodeEditorHandle::new("can't"),
            a11y_selection_gate: code_editor::CodeEditorHandle::new("hello world"),
            a11y_composition_gate: {
                let handle = code_editor::CodeEditorHandle::new("hello world");
                handle.set_caret(2);
                handle
            },
            a11y_selection_wrap_gate: {
                let handle = code_editor::CodeEditorHandle::new(code_editor_wrap_gate_fixture());
                handle
            },
            a11y_composition_wrap_gate: {
                let handle = code_editor::CodeEditorHandle::new(code_editor_wrap_gate_fixture());
                handle.set_caret(78);
                handle
            },
            a11y_composition_drag_gate: {
                let handle = code_editor::CodeEditorHandle::new(code_editor_wrap_gate_fixture());
                handle.set_caret(78);
                handle
            },
        },
        |h| h.clone(),
    );
    let handle = handles.main;
    let word_handle = handles.word_fixture;
    let word_gate_handle = handles.word_gate;
    let word_gate_soft_wrap_handle = handles.word_gate_soft_wrap;
    let a11y_selection_gate_handle = handles.a11y_selection_gate;
    let a11y_composition_gate_handle = handles.a11y_composition_gate;
    let a11y_selection_wrap_gate_handle = handles.a11y_selection_wrap_gate;
    let a11y_composition_wrap_gate_handle = handles.a11y_composition_wrap_gate;
    let a11y_composition_drag_gate_handle = handles.a11y_composition_drag_gate;

    #[derive(Debug, Default, Clone, Copy)]
    struct CodeEditorMvpAppliedFlags {
        syntax_enabled: Option<bool>,
        boundary_identifier_enabled: Option<bool>,
    }

    let applied = cx.with_state(
        || Rc::new(Cell::new(CodeEditorMvpAppliedFlags::default())),
        |v| v.clone(),
    );
    let mut applied_flags = applied.get();
    if applied_flags.syntax_enabled != Some(syntax_enabled) {
        handle.set_language(if syntax_enabled { Some("rust") } else { None });
        applied_flags.syntax_enabled = Some(syntax_enabled);
        applied.set(applied_flags);
    }
    if applied_flags.boundary_identifier_enabled != Some(boundary_identifier_enabled) {
        let mode = if boundary_identifier_enabled {
            fret_runtime::TextBoundaryMode::Identifier
        } else {
            fret_runtime::TextBoundaryMode::UnicodeWord
        };
        handle.set_text_boundary_mode(mode);
        word_handle.set_text_boundary_mode(mode);
        word_gate_handle.set_text_boundary_mode(mode);
        word_gate_soft_wrap_handle.set_text_boundary_mode(mode);
        a11y_selection_gate_handle.set_text_boundary_mode(mode);
        a11y_composition_gate_handle.set_text_boundary_mode(mode);
        a11y_selection_wrap_gate_handle.set_text_boundary_mode(mode);
        a11y_composition_wrap_gate_handle.set_text_boundary_mode(mode);
        a11y_composition_drag_gate_handle.set_text_boundary_mode(mode);
        applied_flags.boundary_identifier_enabled = Some(boundary_identifier_enabled);
        applied.set(applied_flags);
    }

    let word_fixture_loaded = cx.with_state(|| Rc::new(Cell::new(true)), |v| v.clone());
    let word_idx = cx.with_state(|| Rc::new(Cell::new(0usize)), |v| v.clone());
    let word_debug = cx.with_state(
        || Rc::new(std::cell::RefCell::new(String::new())),
        |v| v.clone(),
    );

    let syntax_rust_switch = syntax_rust.clone();
    let boundary_identifier_switch = boundary_identifier.clone();
    let boundary_identifier_for_harness = boundary_identifier.clone();
    let soft_wrap_switch = soft_wrap.clone();
    let boundary_identifier_set_identifier = boundary_identifier_for_harness.clone();
    let set_identifier_mode: fret_ui::action::OnActivate =
        Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&boundary_identifier_set_identifier, |v| *v = true);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        });
    let boundary_identifier_set_unicode = boundary_identifier_for_harness.clone();
    let set_unicode_mode: fret_ui::action::OnActivate =
        Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&boundary_identifier_set_unicode, |v| *v = false);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        });
    let word_handle_for_harness = word_handle.clone();
    let word_gate_handle_for_harness = word_gate_handle.clone();
    let word_gate_soft_wrap_handle_for_harness = word_gate_soft_wrap_handle.clone();
    let word_debug_for_harness = word_debug.clone();
    let word_debug_for_render = word_debug.clone();
    let a11y_selection_gate_handle_for_harness = a11y_selection_gate_handle.clone();
    let a11y_composition_gate_handle_for_harness = a11y_composition_gate_handle.clone();
    let a11y_selection_wrap_gate_handle_for_harness = a11y_selection_wrap_gate_handle.clone();
    let a11y_composition_wrap_gate_handle_for_harness = a11y_composition_wrap_gate_handle.clone();
    let a11y_composition_drag_gate_handle_for_harness = a11y_composition_drag_gate_handle.clone();
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        move |cx| {
            vec![
                cx.text("Goal: validate a paint-driven editable surface using TextInputRegion (focus + IME)."),
                cx.text("Try: drag selection, Ctrl+C/Ctrl+V, arrows, Backspace/Delete, Enter/Tab, IME preedit."),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Switch::new(syntax_rust_switch.clone())
                                .a11y_label("Toggle Rust syntax highlighting")
                                .into_element(cx),
                            cx.text(if syntax_enabled {
                                "Syntax: Rust (tree-sitter)"
                            } else {
                                "Syntax: disabled"
                            }),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Switch::new(boundary_identifier_switch.clone())
                                .a11y_label("Toggle identifier word boundaries")
                                .test_id("ui-gallery-code-editor-boundary-identifier-switch")
                                .into_element(cx),
                            cx.text(if boundary_identifier_enabled {
                                "Word boundaries: Identifier"
                            } else {
                                "Word boundaries: UnicodeWord"
                            }),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Button::new("Set Identifier")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-boundary-set-identifier")
                                .on_activate(set_identifier_mode.clone())
                                .into_element(cx),
                            shadcn::Button::new("Set Unicode")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-boundary-set-unicode")
                                .on_activate(set_unicode_mode.clone())
                                .into_element(cx),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Button::new("Load fonts…")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .on_click(CMD_CODE_EDITOR_LOAD_FONTS)
                                .into_element(cx),
                            shadcn::Button::new("Dump layout…")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .on_click(CMD_CODE_EDITOR_DUMP_TAFFY)
                                .into_element(cx),
                            shadcn::Switch::new(soft_wrap_switch.clone())
                                .test_id("ui-gallery-code-editor-mvp-soft-wrap")
                                .a11y_label("Toggle soft wrap at 80 columns")
                                .into_element(cx),
                            cx.text(if soft_wrap_enabled {
                                "Soft wrap: 80 cols"
                            } else {
                                "Soft wrap: off"
                            }),
                        ]
                    },
                ),
                cx.keyed("word-boundary-gate", |cx| {
                    let gate_editor = code_editor::CodeEditor::new(word_gate_handle_for_harness.clone())
                        .key(2)
                        .overscan(8)
                        .soft_wrap_cols(None)
                        .a11y_label("Code editor word gate")
                        .viewport_test_id("ui-gallery-code-editor-word-gate-viewport")
                        .into_element(cx);
                    cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    )
                }),
                cx.keyed("word-boundary-soft-wrap-gate", |cx| {
                    let gate_editor =
                        code_editor::CodeEditor::new(word_gate_soft_wrap_handle_for_harness.clone())
                            .key(9)
                            .overscan(8)
                            .soft_wrap_cols(Some(4))
                            .a11y_label("Code editor word gate soft wrap")
                            .viewport_test_id(
                                "ui-gallery-code-editor-word-gate-soft-wrap-viewport",
                            )
                            .into_element(cx);
                    cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    )
                }),
                cx.keyed("a11y-selection-gate", |cx| {
                    let gate_editor = code_editor::CodeEditor::new(
                        a11y_selection_gate_handle_for_harness.clone(),
                    )
                    .key(3)
                    .overscan(8)
                    .soft_wrap_cols(None)
                    .a11y_label("Code editor a11y selection gate")
                    .viewport_test_id("ui-gallery-code-editor-a11y-selection-gate-viewport")
                    .into_element(cx);
                    cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    )
                }),
                cx.keyed("a11y-composition-gate", |cx| {
                    let gate_editor = code_editor::CodeEditor::new(
                        a11y_composition_gate_handle_for_harness.clone(),
                    )
                    .key(4)
                    .overscan(8)
                    .soft_wrap_cols(None)
                    .a11y_label("Code editor a11y composition gate")
                    .viewport_test_id("ui-gallery-code-editor-a11y-composition-gate-viewport")
                    .into_element(cx);

                    const COMPOSITION_CARET: usize = 2;

                    let inject = {
                        let handle = a11y_composition_gate_handle_for_harness.clone();
                        Arc::new(move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      action_cx: fret_ui::action::ActionCx,
                                      _up: fret_ui::action::PointerUpCx| {
                            handle.set_caret(COMPOSITION_CARET);
                            handle.set_preedit_debug("ab", None);
                            if let Some(region_id) = handle.region_id() {
                                host.request_focus(region_id);
                            }
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                            true
                        })
                    };

                    let clear = {
                        let handle = a11y_composition_gate_handle_for_harness.clone();
                        Arc::new(move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      action_cx: fret_ui::action::ActionCx,
                                      _up: fret_ui::action::PointerUpCx| {
                            handle.set_caret(COMPOSITION_CARET);
                            handle.set_preedit_debug("", None);
                            if let Some(region_id) = handle.region_id() {
                                host.request_focus(region_id);
                            }
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                            true
                        })
                    };

                    let inject = cx
                        .pointer_region(
                            fret_ui::element::PointerRegionProps::default(),
                            move |cx| {
                                cx.pointer_region_on_pointer_down(Arc::new(
                                    |host, _cx, _down| {
                                        host.prevent_default(
                                            fret_runtime::DefaultAction::FocusOnPointerDown,
                                        );
                                        true
                                    },
                                ));
                                cx.pointer_region_on_pointer_up(inject.clone());
                                vec![cx.text("Inject preedit")]
                            },
                        )
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id("ui-gallery-code-editor-a11y-composition-inject-preedit")
                                .label("Inject preedit"),
                        );

                    let clear = cx
                        .pointer_region(
                            fret_ui::element::PointerRegionProps::default(),
                            move |cx| {
                                cx.pointer_region_on_pointer_down(Arc::new(
                                    |host, _cx, _down| {
                                        host.prevent_default(
                                            fret_runtime::DefaultAction::FocusOnPointerDown,
                                        );
                                        true
                                    },
                                ));
                                cx.pointer_region_on_pointer_up(clear.clone());
                                vec![cx.text("Clear preedit")]
                            },
                        )
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id("ui-gallery-code-editor-a11y-composition-clear-preedit")
                                .label("Clear preedit"),
                        );

                    let controls = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |_cx| vec![inject.clone(), clear.clone()],
                    );

                    let panel = cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    );

                    stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        |_cx| vec![controls, panel],
                    )
                }),
                cx.keyed("a11y-selection-wrap-gate", |cx| {
                    let gate_editor = code_editor::CodeEditor::new(
                        a11y_selection_wrap_gate_handle_for_harness.clone(),
                    )
                    .key(5)
                    .overscan(8)
                    .soft_wrap_cols(Some(80))
                    .a11y_label("Code editor a11y selection wrap gate")
                    .viewport_test_id("ui-gallery-code-editor-a11y-selection-wrap-gate-viewport")
                    .into_element(cx);
                    cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    )
                }),
                cx.keyed("a11y-composition-wrap-gate", |cx| {
                    let gate_editor = code_editor::CodeEditor::new(
                        a11y_composition_wrap_gate_handle_for_harness.clone(),
                    )
                    .key(6)
                    .overscan(8)
                    .soft_wrap_cols(Some(80))
                    .a11y_label("Code editor a11y composition wrap gate")
                    .viewport_test_id("ui-gallery-code-editor-a11y-composition-wrap-gate-viewport")
                    .into_element(cx);

                    const WRAP_CARET: usize = 78;

                    let inject = {
                        let handle = a11y_composition_wrap_gate_handle_for_harness.clone();
                        Arc::new(
                            move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                  action_cx: fret_ui::action::ActionCx,
                                  _up: fret_ui::action::PointerUpCx| {
                                handle.set_caret(WRAP_CARET);
                                handle.set_preedit_debug("ab", None);
                                if let Some(region_id) = handle.region_id() {
                                    host.request_focus(region_id);
                                }
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                                true
                            },
                        )
                    };

                    let clear = {
                        let handle = a11y_composition_wrap_gate_handle_for_harness.clone();
                        Arc::new(
                            move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                  action_cx: fret_ui::action::ActionCx,
                                  _up: fret_ui::action::PointerUpCx| {
                                handle.set_caret(WRAP_CARET);
                                handle.set_preedit_debug("", None);
                                if let Some(region_id) = handle.region_id() {
                                    host.request_focus(region_id);
                                }
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                                true
                            },
                        )
                    };

                    let inject = cx
                        .pointer_region(
                            fret_ui::element::PointerRegionProps::default(),
                            move |cx| {
                                cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                                    host.prevent_default(
                                        fret_runtime::DefaultAction::FocusOnPointerDown,
                                    );
                                    true
                                }));
                                cx.pointer_region_on_pointer_up(inject.clone());
                                vec![cx.text("Inject preedit (wrap)")]
                            },
                        )
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id(
                                    "ui-gallery-code-editor-a11y-composition-wrap-inject-preedit",
                                )
                                .label("Inject preedit (wrap)"),
                        );

                    let clear = cx
                        .pointer_region(
                            fret_ui::element::PointerRegionProps::default(),
                            move |cx| {
                                cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                                    host.prevent_default(
                                        fret_runtime::DefaultAction::FocusOnPointerDown,
                                    );
                                    true
                                }));
                                cx.pointer_region_on_pointer_up(clear.clone());
                                vec![cx.text("Clear preedit (wrap)")]
                            },
                        )
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id(
                                    "ui-gallery-code-editor-a11y-composition-wrap-clear-preedit",
                                )
                                .label("Clear preedit (wrap)"),
                        );

                    let controls = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |_cx| vec![inject.clone(), clear.clone()],
                    );

                    let panel = cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    );

                    stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        |_cx| vec![controls, panel],
                    )
                }),
                cx.keyed("a11y-composition-drag-gate", |cx| {
                    let gate_editor = code_editor::CodeEditor::new(
                        a11y_composition_drag_gate_handle_for_harness.clone(),
                    )
                    .key(7)
                    .overscan(8)
                    .soft_wrap_cols(Some(80))
                    .a11y_label("Code editor a11y composition drag gate")
                    .viewport_test_id("ui-gallery-code-editor-a11y-composition-drag-gate-viewport")
                    .into_element(cx);

                    const WRAP_CARET: usize = 78;

                    let inject = {
                        let handle = a11y_composition_drag_gate_handle_for_harness.clone();
                        Arc::new(
                            move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                  action_cx: fret_ui::action::ActionCx,
                                  _up: fret_ui::action::PointerUpCx| {
                                handle.set_caret(WRAP_CARET);
                                handle.set_preedit_debug("ab", None);
                                if let Some(region_id) = handle.region_id() {
                                    host.request_focus(region_id);
                                }
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                                true
                            },
                        )
                    };

                    let clear = {
                        let handle = a11y_composition_drag_gate_handle_for_harness.clone();
                        Arc::new(
                            move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                  action_cx: fret_ui::action::ActionCx,
                                  _up: fret_ui::action::PointerUpCx| {
                                handle.set_caret(WRAP_CARET);
                                handle.set_preedit_debug("", None);
                                if let Some(region_id) = handle.region_id() {
                                    host.request_focus(region_id);
                                }
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                                true
                            },
                        )
                    };

                    let inject = cx
                        .pointer_region(
                            fret_ui::element::PointerRegionProps::default(),
                            move |cx| {
                                cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                                    host.prevent_default(
                                        fret_runtime::DefaultAction::FocusOnPointerDown,
                                    );
                                    true
                                }));
                                cx.pointer_region_on_pointer_up(inject.clone());
                                vec![cx.text("Inject preedit (drag)")]
                            },
                        )
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id("ui-gallery-code-editor-a11y-composition-drag-inject-preedit")
                                .label("Inject preedit (drag)"),
                        );

                    let clear = cx
                        .pointer_region(
                            fret_ui::element::PointerRegionProps::default(),
                            move |cx| {
                                cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                                    host.prevent_default(
                                        fret_runtime::DefaultAction::FocusOnPointerDown,
                                    );
                                    true
                                }));
                                cx.pointer_region_on_pointer_up(clear.clone());
                                vec![cx.text("Clear preedit (drag)")]
                            },
                        )
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id("ui-gallery-code-editor-a11y-composition-drag-clear-preedit")
                                .label("Clear preedit (drag)"),
                        );

                    let controls = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |_cx| vec![inject.clone(), clear.clone()],
                    );

                    let panel = cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    );

                    stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        |_cx| vec![controls, panel],
                    )
                }),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        let text = word_handle_for_harness.with_buffer(|b| b.text_string());
                        let caret = word_handle_for_harness.selection().caret().min(text.len());
                        if word_idx.get() != caret {
                            word_idx.set(caret);
                        }
                        *word_debug_for_harness.borrow_mut() =
                            format_word_boundary_debug(text.as_str(), caret);

                        let apply_fixture_handle = word_handle_for_harness.clone();
                        let apply_fixture_loaded = word_fixture_loaded.clone();
                        let apply_fixture_idx = word_idx.clone();
                        let apply_fixture_debug = word_debug_for_harness.clone();
                        let apply_fixture: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                let fixture = code_editor_word_boundary_fixture();
                                apply_fixture_handle.set_text(fixture.clone());
                                apply_fixture_handle.set_caret(0);
                                apply_fixture_loaded.set(true);
                                apply_fixture_idx.set(0);
                                *apply_fixture_debug.borrow_mut() =
                                    format_word_boundary_debug(&fixture, 0);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        let prev_char_loaded = word_fixture_loaded.clone();
                        let prev_char_idx = word_idx.clone();
                        let prev_char_handle = word_handle_for_harness.clone();
                        let prev_char_debug = word_debug_for_harness.clone();
                        let prev_char: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                if !prev_char_loaded.get() {
                                    return;
                                }
                                let text = prev_char_handle.with_buffer(|b| b.text_string());
                                let cur = prev_char_idx.get().min(text.len());
                                let next = code_editor_view::prev_char_boundary(text.as_str(), cur);
                                prev_char_idx.set(next);
                                prev_char_handle.set_caret(next);
                                *prev_char_debug.borrow_mut() =
                                    format_word_boundary_debug(text.as_str(), next);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        let next_char_loaded = word_fixture_loaded.clone();
                        let next_char_idx = word_idx.clone();
                        let next_char_handle = word_handle_for_harness.clone();
                        let next_char_debug = word_debug_for_harness.clone();
                        let next_char: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                if !next_char_loaded.get() {
                                    return;
                                }
                                let text = next_char_handle.with_buffer(|b| b.text_string());
                                let cur = next_char_idx.get().min(text.len());
                                let next = code_editor_view::next_char_boundary(text.as_str(), cur);
                                next_char_idx.set(next);
                                next_char_handle.set_caret(next);
                                *next_char_debug.borrow_mut() =
                                    format_word_boundary_debug(text.as_str(), next);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        let prev_word_loaded = word_fixture_loaded.clone();
                        let prev_word_idx = word_idx.clone();
                        let prev_word_handle = word_handle_for_harness.clone();
                        let prev_word_debug = word_debug_for_harness.clone();
                        let prev_word_mode = boundary_identifier_for_harness.clone();
                        let prev_word: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                if !prev_word_loaded.get() {
                                    return;
                                }
                                let text = prev_word_handle.with_buffer(|b| b.text_string());
                                let cur = prev_word_idx.get().min(text.len());
                                let identifier = host
                                    .models_mut()
                                    .read(&prev_word_mode, |v| *v)
                                    .unwrap_or(true);
                                let mode = if identifier {
                                    fret_runtime::TextBoundaryMode::Identifier
                                } else {
                                    fret_runtime::TextBoundaryMode::UnicodeWord
                                };
                                let next = code_editor_view::move_word_left(text.as_str(), cur, mode);
                                prev_word_idx.set(next);
                                prev_word_handle.set_caret(next);
                                *prev_word_debug.borrow_mut() =
                                    format_word_boundary_debug(text.as_str(), next);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        let next_word_loaded = word_fixture_loaded.clone();
                        let next_word_idx = word_idx.clone();
                        let next_word_handle = word_handle_for_harness.clone();
                        let next_word_debug = word_debug_for_harness.clone();
                        let next_word_mode = boundary_identifier_for_harness.clone();
                        let next_word: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                if !next_word_loaded.get() {
                                    return;
                                }
                                let text = next_word_handle.with_buffer(|b| b.text_string());
                                let cur = next_word_idx.get().min(text.len());
                                let identifier = host
                                    .models_mut()
                                    .read(&next_word_mode, |v| *v)
                                    .unwrap_or(true);
                                let mode = if identifier {
                                    fret_runtime::TextBoundaryMode::Identifier
                                } else {
                                    fret_runtime::TextBoundaryMode::UnicodeWord
                                };
                                let next = code_editor_view::move_word_right(text.as_str(), cur, mode);
                                next_word_idx.set(next);
                                next_word_handle.set_caret(next);
                                *next_word_debug.borrow_mut() =
                                    format_word_boundary_debug(text.as_str(), next);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        let apply_caret_loaded = word_fixture_loaded.clone();
                        let apply_caret_idx = word_idx.clone();
                        let apply_caret_handle = word_handle_for_harness.clone();
                        let apply_caret: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                if !apply_caret_loaded.get() {
                                    return;
                                }
                                let text = apply_caret_handle.with_buffer(|b| b.text_string());
                                let idx = apply_caret_idx.get().min(text.len());
                                apply_caret_handle.set_caret(idx);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        let apply_word_loaded = word_fixture_loaded.clone();
                        let apply_word_idx = word_idx.clone();
                        let apply_word_handle = word_handle_for_harness.clone();
                        let apply_word_mode = boundary_identifier_for_harness.clone();
                        let apply_word: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                if !apply_word_loaded.get() {
                                    return;
                                }
                                let text = apply_word_handle.with_buffer(|b| b.text_string());
                                let idx = apply_word_idx.get().min(text.len());
                                let identifier = host
                                    .models_mut()
                                    .read(&apply_word_mode, |v| *v)
                                    .unwrap_or(true);
                                let mode = if identifier {
                                    fret_runtime::TextBoundaryMode::Identifier
                                } else {
                                    fret_runtime::TextBoundaryMode::UnicodeWord
                                };
                                let (a, b) = code_editor_view::select_word_range(text.as_str(), idx, mode);
                                apply_word_handle.set_selection(code_editor::Selection {
                                    anchor: a,
                                    focus: b,
                                });
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        vec![
                            shadcn::Button::new("Load word-boundary fixture")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(apply_fixture)
                                .into_element(cx),
                            shadcn::Button::new("Prev char")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(prev_char)
                                .disabled(!word_fixture_loaded.get())
                                .into_element(cx),
                            shadcn::Button::new("Next char")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(next_char)
                                .disabled(!word_fixture_loaded.get())
                                .into_element(cx),
                            shadcn::Button::new("Prev word")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(prev_word)
                                .disabled(!word_fixture_loaded.get())
                                .into_element(cx),
                            shadcn::Button::new("Next word")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(next_word)
                                .disabled(!word_fixture_loaded.get())
                                .into_element(cx),
                            shadcn::Button::new("Apply caret")
                                .variant(shadcn::ButtonVariant::Ghost)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(apply_caret)
                                .disabled(!word_fixture_loaded.get())
                                .into_element(cx),
                            shadcn::Button::new("Apply selection")
                                .variant(shadcn::ButtonVariant::Ghost)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(apply_word)
                                .disabled(!word_fixture_loaded.get())
                                .into_element(cx),
                        ]
                    },
                ),
                cx.keyed("word-boundary-debug", |cx| {
                    stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        move |cx| {
                            let fixture_editor = code_editor::CodeEditor::new(word_handle.clone())
                                .key(1)
                                .overscan(8)
                                .soft_wrap_cols(None)
                                .viewport_test_id("ui-gallery-code-editor-word-fixture-viewport")
                                .into_element(cx);
                            let fixture_panel = cx.container(
                                decl_style::container_props(
                                    theme,
                                    ChromeRefinement::default()
                                        .border_1()
                                        .rounded(Radius::Md)
                                        .bg(ColorRef::Color(theme.color_required("background"))),
                                    LayoutRefinement::default()
                                        .w_full()
                                        .h_px(MetricRef::Px(Px(150.0))),
                                ),
                                |_cx| vec![fixture_editor],
                            );

                            let debug = word_debug_for_render.borrow().clone();
                            let lines: Vec<Arc<str>> = debug
                                .lines()
                                .map(|line| Arc::<str>::from(line.to_string()))
                                .collect();
                            let debug_lines = stack::vstack(
                                cx,
                                stack::VStackProps::default()
                                    .layout(LayoutRefinement::default().w_full())
                                    .gap(Space::N0),
                                move |cx| {
                                    lines
                                        .iter()
                                        .cloned()
                                        .map(|line| {
                                            let mut props = fret_ui::element::TextProps::new(line);
                                            props.style = Some(TextStyle {
                                                font: FontId::monospace(),
                                                size: Px(12.0),
                                                ..Default::default()
                                            });
                                            props.wrap = TextWrap::None;
                                            props.overflow = TextOverflow::Clip;
                                            cx.text_props(props)
                                        })
                                        .collect::<Vec<_>>()
                                },
                            );

                            vec![fixture_panel, debug_lines]
                        },
                    )
                }),
            ]
        },
    );

    let editor = code_editor::CodeEditor::new(handle)
        .key(0)
        .overscan(32)
        .soft_wrap_cols(soft_wrap_enabled.then_some(80))
        .viewport_test_id("ui-gallery-code-editor-mvp-viewport")
        .into_element(cx);

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(520.0))),
        ),
        |_cx| vec![editor],
    );

    let panel = panel.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-code-editor-root"),
    );

    vec![header, panel]
}

pub(in crate::ui) fn preview_code_editor_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    syntax_rust: Model<bool>,
    boundary_identifier: Model<bool>,
    soft_wrap: Model<bool>,
    folds: Model<bool>,
    inlays: Model<bool>,
) -> Vec<AnyElement> {
    let syntax_enabled = cx
        .get_model_copied(&syntax_rust, Invalidation::Layout)
        .unwrap_or(false);
    let boundary_identifier_enabled = cx
        .get_model_copied(&boundary_identifier, Invalidation::Layout)
        .unwrap_or(true);
    let soft_wrap_enabled = cx
        .get_model_copied(&soft_wrap, Invalidation::Layout)
        .unwrap_or(false);
    let folds_enabled = cx
        .get_model_copied(&folds, Invalidation::Layout)
        .unwrap_or(false);
    let inlays_enabled = cx
        .get_model_copied(&inlays, Invalidation::Layout)
        .unwrap_or(false);

    let soft_wrap_set_on = soft_wrap.clone();
    let set_soft_wrap_on: fret_ui::action::OnActivate =
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&soft_wrap_set_on, |v| *v = true);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        });
    let soft_wrap_set_off = soft_wrap.clone();
    let set_soft_wrap_off: fret_ui::action::OnActivate =
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&soft_wrap_set_off, |v| *v = false);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        });

    let folds_set_on = folds.clone();
    let set_folds_on: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&folds_set_on, |v| *v = true);
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    });
    let folds_set_off = folds.clone();
    let set_folds_off: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&folds_set_off, |v| *v = false);
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    });

    let inlays_set_on = inlays.clone();
    let set_inlays_on: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&inlays_set_on, |v| *v = true);
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    });
    let inlays_set_off = inlays.clone();
    let set_inlays_off: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&inlays_set_off, |v| *v = false);
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    });

    let handle = cx.with_state(
        || code_editor::CodeEditorHandle::new(code_editor_torture_source()),
        |h| h.clone(),
    );
    let last_applied = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_applied.get() != Some(syntax_enabled) {
        handle.set_language(if syntax_enabled { Some("rust") } else { None });
        last_applied.set(Some(syntax_enabled));
    }
    let last_boundaries = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_boundaries.get() != Some(boundary_identifier_enabled) {
        handle.set_text_boundary_mode(if boundary_identifier_enabled {
            fret_runtime::TextBoundaryMode::Identifier
        } else {
            fret_runtime::TextBoundaryMode::UnicodeWord
        });
        last_boundaries.set(Some(boundary_identifier_enabled));
    }

    let last_folds = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_folds.get() != Some(folds_enabled) {
        if folds_enabled {
            let span = handle.with_buffer(|b| b.line_text(0)).and_then(|line| {
                let prefix_end = line.find(": ").map(|i| i + 2).unwrap_or(0);
                let comment_start = line.find("//").unwrap_or_else(|| line.len());
                let start = prefix_end.min(line.len());
                let end = comment_start.min(line.len());
                if start < end {
                    Some(code_editor_view::FoldSpan {
                        range: start..end,
                        placeholder: Arc::<str>::from("…"),
                    })
                } else {
                    None
                }
            });
            if let Some(span) = span {
                handle.set_line_folds(0, vec![span]);
            } else {
                handle.clear_all_folds();
            }
        } else {
            handle.clear_all_folds();
        }
        last_folds.set(Some(folds_enabled));
    }

    let last_inlays = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_inlays.get() != Some(inlays_enabled) {
        if inlays_enabled {
            let byte = handle
                .with_buffer(|b| b.line_text(0))
                .map(|line| line.find(": ").map(|i| i + 2).unwrap_or(0).min(line.len()))
                .unwrap_or(0);
            handle.set_line_inlays(
                0,
                vec![code_editor_view::InlaySpan {
                    byte,
                    text: Arc::<str>::from("<inlay>"),
                }],
            );
        } else {
            handle.clear_all_inlays();
        }
        last_inlays.set(Some(inlays_enabled));
    }

    let allow_decorations_under_preedit =
        cx.with_state(|| Rc::new(Cell::new(false)), |v| v.clone());
    let allow_decorations_under_preedit_enabled = allow_decorations_under_preedit.get();
    if handle.allow_decorations_under_inline_preedit() != allow_decorations_under_preedit_enabled {
        handle.set_allow_decorations_under_inline_preedit(allow_decorations_under_preedit_enabled);
    }

    let compose_inline_preedit = cx.with_state(|| Rc::new(Cell::new(false)), |v| v.clone());
    let compose_inline_preedit_enabled = compose_inline_preedit.get();
    if handle.compose_inline_preedit() != compose_inline_preedit_enabled {
        handle.set_compose_inline_preedit(compose_inline_preedit_enabled);
    }

    let header_handle = handle.clone();
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        move |cx| {
            let header_handle_controls = header_handle.clone();
            let header_handle_mode = header_handle.clone();
            vec![
                cx.text("Goal: stress scroll stability + bounded text caching for the windowed code editor."),
                cx.text("Expect: auto-scroll bounce; line prefixes must stay consistent (no stale paint)."),
                cx.text("Note: with soft wrap enabled, continuation rows may start mid-token (the numeric prefix does not repeat)."),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Switch::new(syntax_rust.clone())
                                .a11y_label("Toggle Rust syntax highlighting")
                                .into_element(cx),
                            cx.text(if syntax_enabled {
                                "Syntax: Rust (tree-sitter)"
                            } else {
                                "Syntax: disabled"
                            }),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Switch::new(boundary_identifier.clone())
                                .a11y_label("Toggle identifier word boundaries")
                                .into_element(cx),
                            cx.text(if boundary_identifier_enabled {
                                "Word boundaries: Identifier"
                            } else {
                                "Word boundaries: UnicodeWord"
                            }),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        let reset_handle = header_handle_controls.clone();
                        let preedit_handle = header_handle_controls.clone();
                        let clear_preedit_handle = header_handle_controls.clone();
                        let allow_decorations_under_preedit_off =
                            allow_decorations_under_preedit.clone();
                        let allow_decorations_under_preedit_on =
                            allow_decorations_under_preedit.clone();
                        let header_handle_controls_off = header_handle_controls.clone();
                        let header_handle_controls_on = header_handle_controls.clone();
                        vec![
                            shadcn::Button::new("Load fonts…")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .on_click(CMD_CODE_EDITOR_LOAD_FONTS)
                                .into_element(cx),
                            shadcn::Button::new("Reset editor stats")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-reset-stats")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    reset_handle.reset_cache_stats();
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Switch::new(soft_wrap.clone())
                                .test_id("ui-gallery-code-editor-torture-soft-wrap")
                                .a11y_label("Toggle soft wrap at 80 columns")
                                .into_element(cx),
                            shadcn::Button::new("Wrap: off")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-soft-wrap-set-off")
                                .on_activate(set_soft_wrap_off.clone())
                                .into_element(cx),
                            shadcn::Button::new("Wrap: 80")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-soft-wrap-set-on")
                                .on_activate(set_soft_wrap_on.clone())
                                .into_element(cx),
                            cx.text(if soft_wrap_enabled {
                                "Soft wrap: 80 cols"
                            } else {
                                "Soft wrap: off"
                            }),
                            shadcn::Button::new("Preedit: inject")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-inject-preedit")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    preedit_handle.set_preedit_debug("ab", None);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Preedit: clear")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-clear-preedit")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    clear_preedit_handle.set_preedit_debug("", None);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Preedit decorations: off")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id(
                                    "ui-gallery-code-editor-torture-preedit-decorations-set-off",
                                )
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    allow_decorations_under_preedit_off.set(false);
                                    header_handle_controls_off
                                        .set_allow_decorations_under_inline_preedit(false);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Preedit decorations: on")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id(
                                    "ui-gallery-code-editor-torture-preedit-decorations-set-on",
                                )
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    allow_decorations_under_preedit_on.set(true);
                                    header_handle_controls_on
                                        .set_allow_decorations_under_inline_preedit(true);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            cx.text(if allow_decorations_under_preedit_enabled {
                                "Preedit decorations: on"
                            } else {
                                "Preedit decorations: off"
                            }),
                            shadcn::Button::new("Preedit composition: paint")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-preedit-compose-set-paint")
                                .on_activate({
                                    let compose_inline_preedit = compose_inline_preedit.clone();
                                    let header_handle_controls = header_handle_controls.clone();
                                    Arc::new(move |host, action_cx, _reason| {
                                        compose_inline_preedit.set(false);
                                        header_handle_controls.set_compose_inline_preedit(false);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    })
                                })
                                .into_element(cx),
                            shadcn::Button::new("Preedit composition: view")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-preedit-compose-set-view")
                                .on_activate({
                                    let compose_inline_preedit = compose_inline_preedit.clone();
                                    let header_handle_controls = header_handle_controls.clone();
                                    Arc::new(move |host, action_cx, _reason| {
                                        compose_inline_preedit.set(true);
                                        header_handle_controls.set_compose_inline_preedit(true);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    })
                                })
                                .into_element(cx),
                            cx.text(if compose_inline_preedit_enabled {
                                "Preedit composition: view (composed)"
                            } else {
                                "Preedit composition: paint (injected)"
                            }),
                            shadcn::Switch::new(folds.clone())
                                .test_id("ui-gallery-code-editor-torture-folds")
                                .a11y_label("Toggle fold fixture on line 0")
                                .into_element(cx),
                            shadcn::Button::new("Folds: off")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-folds-set-off")
                                .on_activate(set_folds_off.clone())
                                .into_element(cx),
                            shadcn::Button::new("Folds: on")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-folds-set-on")
                                .on_activate(set_folds_on.clone())
                                .into_element(cx),
                            cx.text(if folds_enabled {
                                "Folds: fixture"
                            } else {
                                "Folds: off"
                            }),
                            shadcn::Switch::new(inlays.clone())
                                .test_id("ui-gallery-code-editor-torture-inlays")
                                .a11y_label("Toggle inlay fixture on line 0")
                                .into_element(cx),
                            shadcn::Button::new("Inlays: off")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-inlays-set-off")
                                .on_activate(set_inlays_off.clone())
                                .into_element(cx),
                            shadcn::Button::new("Inlays: on")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-inlays-set-on")
                                .on_activate(set_inlays_on.clone())
                                .into_element(cx),
                            cx.text(if inlays_enabled {
                                "Inlays: fixture"
                            } else {
                                "Inlays: off"
                            }),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        let mode_handle = header_handle_mode.clone();
                        let edit_handle = header_handle_mode.clone();
                        let read_only_handle = header_handle_mode.clone();
                        let disabled_handle = header_handle_mode.clone();

                        let mode = mode_handle.interaction();
                        let mode_label = if !mode.enabled {
                            "disabled"
                        } else if !mode.editable {
                            "read-only"
                        } else {
                            "edit"
                        };

                        vec![
                            shadcn::Button::new("Mode: edit")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-mode-edit")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    edit_handle.set_interaction(code_editor::CodeEditorInteractionOptions::editor());
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Mode: read-only")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-mode-read-only")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    read_only_handle
                                        .set_interaction(code_editor::CodeEditorInteractionOptions::read_only());
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Mode: disabled")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-mode-disabled")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    disabled_handle
                                        .set_interaction(code_editor::CodeEditorInteractionOptions::disabled());
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            cx.text(format!("Interaction: {mode_label}")),
                        ]
                    },
                ),
            ]
        },
    );

    #[cfg(not(target_arch = "wasm32"))]
    cx.app.with_global_mut(
        crate::harness::UiGalleryCodeEditorHandlesStore::default,
        |store, _app| {
            store.per_window.insert(cx.window, handle.clone());
        },
    );

    let editor = code_editor::CodeEditor::new(handle)
        .overscan(128)
        .soft_wrap_cols(soft_wrap_enabled.then_some(80))
        .torture(code_editor::CodeEditorTorture::auto_scroll_bounce(Px(8.0)))
        .viewport_test_id("ui-gallery-code-editor-torture-viewport")
        .into_element(cx);

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(520.0))),
        ),
        |_cx| vec![editor],
    );

    let panel = panel.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-code-editor-torture-root"),
    );

    vec![header, panel]
}

pub(in crate::ui) fn markdown_editor_source_text() -> Arc<str> {
    static SOURCE: OnceLock<Arc<str>> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            Arc::<str>::from(
                "\
# Markdown Editor v0 (source mode)

This page is a contract milestone for `fret-code-editor`:

- editable vs read-only interaction control
- soft wrap stability
- Markdown syntax highlighting (best-effort)

## Fenced code block

```rust
pub(in crate::ui) fn main() {
    println!(\"hello\");
}
```

## List

- item one
- item two

## Inline code

Use `CodeEditorInteractionOptions::read_only()` for viewers.
",
            )
        })
        .clone()
}

pub(in crate::ui) fn preview_markdown_editor_source(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    soft_wrap: Model<bool>,
    folds: Model<bool>,
    inlays: Model<bool>,
) -> Vec<AnyElement> {
    let soft_wrap_enabled = cx
        .get_model_copied(&soft_wrap, Invalidation::Layout)
        .unwrap_or(false);
    let folds_enabled = cx
        .get_model_copied(&folds, Invalidation::Layout)
        .unwrap_or(false);
    let inlays_enabled = cx
        .get_model_copied(&inlays, Invalidation::Layout)
        .unwrap_or(false);

    let handle = cx.with_state(
        || code_editor::CodeEditorHandle::new(markdown_editor_source_text().as_ref().to_string()),
        |h| h.clone(),
    );
    // Best-effort: only takes effect when `fret-code-editor` is built with `syntax` features.
    handle.set_language(Some("markdown"));
    // Markdown source editing uses Unicode word boundaries (ADR 0179).
    handle.set_text_boundary_mode(fret_runtime::TextBoundaryMode::UnicodeWord);

    #[cfg(not(target_arch = "wasm32"))]
    cx.app.with_global_mut(
        crate::harness::UiGalleryMarkdownEditorHandlesStore::default,
        |store, _app| {
            store.per_window.insert(cx.window, handle.clone());
        },
    );

    let last_folds = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_folds.get() != Some(folds_enabled) {
        if folds_enabled {
            let span = handle.with_buffer(|b| b.line_text(0)).and_then(|line| {
                let start = line.find("Editor").unwrap_or(2).min(line.len());
                let end = line.len();
                if start < end {
                    Some(code_editor_view::FoldSpan {
                        range: start..end,
                        placeholder: Arc::<str>::from("…"),
                    })
                } else {
                    None
                }
            });
            if let Some(span) = span {
                handle.set_line_folds(0, vec![span]);
            } else {
                handle.clear_all_folds();
            }
        } else {
            handle.clear_all_folds();
        }
        last_folds.set(Some(folds_enabled));
    }

    let last_inlays = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_inlays.get() != Some(inlays_enabled) {
        if inlays_enabled {
            let byte = handle
                .with_buffer(|b| b.line_text(0))
                .map(|line| 2usize.min(line.len()))
                .unwrap_or(0);
            handle.set_line_inlays(
                0,
                vec![code_editor_view::InlaySpan {
                    byte,
                    text: Arc::<str>::from("<inlay>"),
                }],
            );
        } else {
            handle.clear_all_inlays();
        }
        last_inlays.set(Some(inlays_enabled));
    }

    let header_handle = handle.clone();
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        move |cx| {
            let mode_handle = header_handle.clone();
            let edit_handle = header_handle.clone();
            let read_only_handle = header_handle.clone();
            let disabled_handle = header_handle.clone();

            let mode = mode_handle.interaction();
            let mode_label = if !mode.enabled {
                "disabled"
            } else if !mode.editable {
                "read-only"
            } else {
                "edit"
            };

            vec![
                cx.text("Goal: validate a minimal Markdown source editor milestone."),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        let set_soft_wrap_on = soft_wrap.clone();
                        let set_soft_wrap_off = soft_wrap.clone();
                        vec![
                            shadcn::Switch::new(soft_wrap.clone())
                                .test_id("ui-gallery-markdown-editor-soft-wrap")
                                .a11y_label("Toggle soft wrap at 80 columns")
                                .into_element(cx),
                            shadcn::Button::new("Wrap: off")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-markdown-editor-soft-wrap-set-off")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    let _ = host
                                        .models_mut()
                                        .update(&set_soft_wrap_off, |v| *v = false);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Wrap: 80")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-markdown-editor-soft-wrap-set-on")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    let _ =
                                        host.models_mut().update(&set_soft_wrap_on, |v| *v = true);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            cx.text(if soft_wrap_enabled {
                                "Soft wrap: 80 cols"
                            } else {
                                "Soft wrap: off"
                            }),
                        ]
                    },
                ),
                {
                    let folds_caret_handle = header_handle.clone();
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |cx| {
                            let set_folds_on = folds.clone();
                            let set_folds_off = folds.clone();
                            let set_inlays_on = inlays.clone();
                            let set_inlays_off = inlays.clone();
                            let caret_handle = folds_caret_handle.clone();

                            vec![
                                shadcn::Switch::new(folds.clone())
                                    .test_id("ui-gallery-markdown-editor-folds")
                                    .a11y_label("Toggle fold fixture on line 0")
                                    .into_element(cx),
                                shadcn::Button::new("Folds: off")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-markdown-editor-folds-set-off")
                                    .on_activate(Arc::new(move |host, action_cx, _reason| {
                                        let _ = host
                                            .models_mut()
                                            .update(&set_folds_off, |v| *v = false);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    }))
                                    .into_element(cx),
                                shadcn::Button::new("Folds: on")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-markdown-editor-folds-set-on")
                                    .on_activate(Arc::new(move |host, action_cx, _reason| {
                                        let _ =
                                            host.models_mut().update(&set_folds_on, |v| *v = true);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    }))
                                    .into_element(cx),
                                shadcn::Button::new("Caret: in fold")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-markdown-editor-folds-set-caret-inside")
                                    .on_activate(Arc::new(move |host, action_cx, _reason| {
                                        if !caret_handle.interaction().enabled {
                                            return;
                                        }

                                        let Some(byte) = caret_handle.with_buffer(|b| {
                                            let line = b.line_text(0)?;
                                            let line_range = b.line_byte_range(0)?;
                                            let start =
                                                line.find("Editor").unwrap_or(2).min(line.len());
                                            let end = line.len();
                                            if start + 1 >= end {
                                                return None;
                                            }
                                            Some(line_range.start.saturating_add(start + 1))
                                        }) else {
                                            return;
                                        };

                                        caret_handle.set_caret(byte);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    }))
                                    .into_element(cx),
                                cx.text(if folds_enabled {
                                    "Folds: fixture"
                                } else {
                                    "Folds: off"
                                }),
                                shadcn::Switch::new(inlays.clone())
                                    .test_id("ui-gallery-markdown-editor-inlays")
                                    .a11y_label("Toggle inlay fixture on line 0")
                                    .into_element(cx),
                                shadcn::Button::new("Inlays: off")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-markdown-editor-inlays-set-off")
                                    .on_activate(Arc::new(move |host, action_cx, _reason| {
                                        let _ = host
                                            .models_mut()
                                            .update(&set_inlays_off, |v| *v = false);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    }))
                                    .into_element(cx),
                                shadcn::Button::new("Inlays: on")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-markdown-editor-inlays-set-on")
                                    .on_activate(Arc::new(move |host, action_cx, _reason| {
                                        let _ =
                                            host.models_mut().update(&set_inlays_on, |v| *v = true);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    }))
                                    .into_element(cx),
                                cx.text(if inlays_enabled {
                                    "Inlays: fixture"
                                } else {
                                    "Inlays: off"
                                }),
                            ]
                        },
                    )
                },
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        let inject = {
                            let handle = header_handle.clone();
                            Arc::new(
                                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      action_cx: fret_ui::action::ActionCx,
                                      _up: fret_ui::action::PointerUpCx| {
                                    if !handle.interaction().enabled {
                                        return true;
                                    }
                                    const COMPOSITION_CARET: usize = 2;
                                    handle.set_caret(COMPOSITION_CARET);
                                    handle.set_preedit_debug("ab", None);
                                    if let Some(region_id) = handle.region_id() {
                                        host.request_focus(region_id);
                                    }
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                    true
                                },
                            )
                        };

                        let clear = {
                            let handle = header_handle.clone();
                            Arc::new(
                                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      action_cx: fret_ui::action::ActionCx,
                                      _up: fret_ui::action::PointerUpCx| {
                                    if !handle.interaction().enabled {
                                        return true;
                                    }
                                    const COMPOSITION_CARET: usize = 2;
                                    handle.set_caret(COMPOSITION_CARET);
                                    handle.set_preedit_debug("", None);
                                    if let Some(region_id) = handle.region_id() {
                                        host.request_focus(region_id);
                                    }
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                    true
                                },
                            )
                        };

                        let inject = cx
                            .pointer_region(
                                fret_ui::element::PointerRegionProps::default(),
                                move |cx| {
                                    cx.pointer_region_on_pointer_down(Arc::new(
                                        |host, _cx, _down| {
                                            host.prevent_default(
                                                fret_runtime::DefaultAction::FocusOnPointerDown,
                                            );
                                            true
                                        },
                                    ));
                                    cx.pointer_region_on_pointer_up(inject.clone());
                                    vec![cx.text("Preedit: inject")]
                                },
                            )
                            .attach_semantics(
                                SemanticsDecoration::default()
                                    .role(fret_core::SemanticsRole::Button)
                                    .test_id("ui-gallery-markdown-editor-inject-preedit")
                                    .label("Inject preedit"),
                            );

                        let clear = cx
                            .pointer_region(
                                fret_ui::element::PointerRegionProps::default(),
                                move |cx| {
                                    cx.pointer_region_on_pointer_down(Arc::new(
                                        |host, _cx, _down| {
                                            host.prevent_default(
                                                fret_runtime::DefaultAction::FocusOnPointerDown,
                                            );
                                            true
                                        },
                                    ));
                                    cx.pointer_region_on_pointer_up(clear.clone());
                                    vec![cx.text("Preedit: clear")]
                                },
                            )
                            .attach_semantics(
                                SemanticsDecoration::default()
                                    .role(fret_core::SemanticsRole::Button)
                                    .test_id("ui-gallery-markdown-editor-clear-preedit")
                                    .label("Clear preedit"),
                            );

                        vec![
                            shadcn::Button::new("Mode: edit")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-markdown-editor-mode-edit")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    edit_handle.set_interaction(
                                        code_editor::CodeEditorInteractionOptions::editor(),
                                    );
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Mode: read-only")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-markdown-editor-mode-read-only")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    read_only_handle.set_interaction(
                                        code_editor::CodeEditorInteractionOptions::read_only(),
                                    );
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Mode: disabled")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-markdown-editor-mode-disabled")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    disabled_handle.set_interaction(
                                        code_editor::CodeEditorInteractionOptions::disabled(),
                                    );
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            inject,
                            clear,
                            cx.text(format!("Interaction: {mode_label}")),
                        ]
                    },
                ),
            ]
        },
    );

    let editor = code_editor::CodeEditor::new(handle.clone())
        .overscan(64)
        .soft_wrap_cols(soft_wrap_enabled.then_some(80))
        .a11y_label("Markdown editor")
        .viewport_test_id("ui-gallery-markdown-editor-viewport")
        .into_element(cx);

    let preview_cache = cx.with_state(
        || Rc::new(RefCell::new((0u64, Arc::<str>::from("")))),
        |v| v.clone(),
    );
    let rev = handle.buffer_revision().0 as u64;
    let preview_source = {
        let mut cached = preview_cache.borrow_mut();
        if cached.0 != rev {
            cached.0 = rev;
            cached.1 = handle.with_buffer(|b| Arc::<str>::from(b.text_string()));
        }
        cached.1.clone()
    };
    let preview = markdown::Markdown::new(preview_source).into_element(cx);

    let editor_panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(520.0))),
        ),
        |_cx| vec![editor],
    );

    let preview_panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(520.0))),
        ),
        |_cx| vec![preview],
    );

    let body = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4),
        |_cx| vec![editor_panel, preview_panel],
    );

    let body = body.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-markdown-editor-root"),
    );

    vec![header, body]
}

pub(in crate::ui) fn selection_perf_source() -> Arc<str> {
    static SOURCE: OnceLock<Arc<str>> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            use std::fmt::Write;

            let mut out = String::with_capacity(320_000);
            for i in 0..5000usize {
                let _ = writeln!(
                    &mut out,
                    "{i:05}: The quick brown fox jumps over the lazy dog. 0123456789 ABC xyz"
                );
            }
            Arc::<str>::from(out)
        })
        .clone()
}

pub(in crate::ui) fn preview_text_selection_perf(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    struct PreparedKey {
        max_width_bits: u32,
        scale_bits: u32,
    }

    #[derive(Default)]
    struct SelectionPerfState {
        scroll_y: Px,
        content_height: Px,
        viewport_height: Px,
        last_clipped_rects: usize,
        prepared_key: Option<PreparedKey>,
        blob: Option<fret_core::TextBlobId>,
        metrics: Option<fret_core::TextMetrics>,
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(SelectionPerfState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: track selection rect count for large selections."),
                cx.text("Expectation: rect generation scales with visible lines when clipped to the viewport (not document length)."),
                cx.text("Scroll with the mouse wheel over the demo surface."),
            ]
        },
    );

    let source = selection_perf_source();
    let source_len = source.len();

    let on_wheel_state = state.clone();
    let on_wheel: fret_ui::action::OnWheel = Arc::new(move |host, action_cx, wheel| {
        let mut st = on_wheel_state.borrow_mut();

        let max_scroll = (st.content_height.0 - st.viewport_height.0).max(0.0);
        if max_scroll <= 0.0 {
            st.scroll_y = Px(0.0);
        } else {
            st.scroll_y = Px((st.scroll_y.0 - wheel.delta.y.0).clamp(0.0, max_scroll));
        }

        host.invalidate(fret_ui::Invalidation::Paint);
        host.request_redraw(action_cx.window);
        true
    });

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(420.0))),
        ),
        move |cx| {
            let mut pointer = fret_ui::element::PointerRegionProps::default();
            pointer.layout.size.width = fret_ui::element::Length::Fill;
            pointer.layout.size.height = fret_ui::element::Length::Fill;
            pointer.layout.overflow = fret_ui::element::Overflow::Clip;

            let paint_state = state.clone();
            let paint_source = source.clone();

            let content = cx.pointer_region(pointer, move |cx| {
                cx.pointer_region_on_wheel(on_wheel.clone());

                let mut canvas = CanvasProps::default();
                canvas.layout.size.width = fret_ui::element::Length::Fill;
                canvas.layout.size.height = fret_ui::element::Length::Fill;
                canvas.layout.overflow = fret_ui::element::Overflow::Clip;
                canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

                let canvas = cx.canvas(canvas, move |p| {
                    let bounds = p.bounds();
                    let pad = Px(12.0);

                    let inner = Rect::new(
                        Point::new(
                            Px(bounds.origin.x.0 + pad.0),
                            Px(bounds.origin.y.0 + pad.0),
                        ),
                        Size::new(
                            Px((bounds.size.width.0 - 2.0 * pad.0).max(0.0)),
                            Px((bounds.size.height.0 - 2.0 * pad.0).max(0.0)),
                        ),
                    );

                    let max_width = inner.size.width;
                    if max_width.0 <= 0.0 || inner.size.height.0 <= 0.0 {
                        return;
                    }

                    let scale_factor = p.scale_factor();
                    let selection_bg = p.theme().color_required("selection.background");
                    let fg = p.theme().color_required("foreground");
                    let muted = p.theme().color_required("muted-foreground");

                    let key = PreparedKey {
                        max_width_bits: max_width.0.to_bits(),
                        scale_bits: scale_factor.to_bits(),
                    };

                    let (stats, stats_origin) = {
                        let (services, scene) = p.services_and_scene();
                        let mut st = paint_state.borrow_mut();

                        let needs_prepare = st.blob.is_none()
                            || st.metrics.is_none()
                            || st.prepared_key != Some(key);
                        if needs_prepare {
                            if let Some(blob) = st.blob.take() {
                                services.text().release(blob);
                            }

                            let style = fret_core::TextStyle {
                                font: fret_core::FontId::monospace(),
                                size: Px(12.0),
                                ..Default::default()
                            };

                            let constraints = fret_core::TextConstraints {
                                max_width: Some(max_width),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                                scale_factor,
                            };

                            let (blob, metrics) = services
                                .text()
                                .prepare_str(paint_source.as_ref(), &style, constraints);
                            st.prepared_key = Some(key);
                            st.blob = Some(blob);
                            st.metrics = Some(metrics);
                        }

                        let Some(blob) = st.blob else {
                            return;
                        };
                        let Some(metrics) = st.metrics else {
                            return;
                        };

                        st.content_height = metrics.size.height;
                        st.viewport_height = inner.size.height;
                        let max_scroll = (st.content_height.0 - st.viewport_height.0).max(0.0);
                        st.scroll_y = Px(st.scroll_y.0.clamp(0.0, max_scroll));

                        let clip = Rect::new(
                            Point::new(Px(0.0), st.scroll_y),
                            Size::new(max_width, st.viewport_height),
                        );

                        let mut rects: Vec<Rect> = Vec::new();
                        services.selection_rects_clipped(blob, (0, source_len), clip, &mut rects);
                        st.last_clipped_rects = rects.len();

                        scene.push(SceneOp::PushClipRect { rect: inner });
                        for r in rects {
                            let rect = Rect::new(
                                Point::new(
                                    Px(inner.origin.x.0 + r.origin.x.0),
                                    Px(inner.origin.y.0 + r.origin.y.0 - st.scroll_y.0),
                                ),
                                r.size,
                            );
                            scene.push(SceneOp::Quad {
                                order: DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(selection_bg),

                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT,

                                corner_radii: Corners::all(Px(0.0)),
                            });
                        }

                        let text_origin = Point::new(
                            inner.origin.x,
                            Px(inner.origin.y.0 + metrics.baseline.0 - st.scroll_y.0),
                        );
                        scene.push(SceneOp::Text {
                            order: DrawOrder(1),
                            origin: text_origin,
                            text: blob,
                            color: fg,
                        });
                        scene.push(SceneOp::PopClip);

                        let stats = format!(
                            "clipped rects: {} | scroll_y: {:.1}/{:.1} | content_h: {:.1} | viewport_h: {:.1}",
                            st.last_clipped_rects,
                            st.scroll_y.0,
                            max_scroll,
                            st.content_height.0,
                            st.viewport_height.0
                        );
                        let stats_origin = Point::new(
                            Px(bounds.origin.x.0 + 12.0),
                            Px(bounds.origin.y.0 + 10.0),
                        );
                        (stats, stats_origin)
                    };

                    let stats_style = fret_core::TextStyle {
                        font: fret_core::FontId::ui(),
                        size: Px(12.0),
                        ..Default::default()
                    };
                    let _ = p.text(
                        p.key(&"text_selection_perf_stats"),
                        DrawOrder(2),
                        stats_origin,
                        stats,
                        stats_style,
                        muted,
                        fret_ui::canvas::CanvasTextConstraints::default(),
                        scale_factor,
                    );
                });

                vec![canvas]
            });

            vec![content]
        },
    );

    let panel = panel.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-text-selection-perf-root"),
    );

    vec![header, panel]
}

pub(in crate::ui) fn preview_text_bidi_rtl_conformance(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Clone, Copy)]
    struct BidiSample {
        label: &'static str,
        text: &'static str,
    }

    const SAMPLES: &[BidiSample] = &[
        BidiSample {
            label: "LTR baseline",
            text: "The quick brown fox (123) jumps.",
        },
        BidiSample {
            label: "Hebrew (RTL)",
            text: "עברית (123) אבגדה",
        },
        BidiSample {
            label: "Arabic (RTL)",
            text: "مرحبا بالعالم (123) أهلاً",
        },
        BidiSample {
            label: "Mixed LTR + Hebrew",
            text: "abc אבג DEF 123",
        },
        BidiSample {
            label: "Mixed punctuation + numbers",
            text: "abc (אבג) - 12:34 - xyz",
        },
        BidiSample {
            label: "Mixed LTR + Arabic",
            text: "hello مرحبا (123) world",
        },
        BidiSample {
            label: "Grapheme + RTL",
            text: "emoji 😀 אבג café",
        },
        BidiSample {
            label: "Controls (RLM)",
            text: "RLM:\u{200F}abc אבג 123",
        },
    ];

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct PreparedKey {
        sample: usize,
        max_width_bits: u32,
        scale_bits: u32,
    }

    struct BidiState {
        selected_sample: usize,
        prepared_key: Option<PreparedKey>,
        blob: Option<fret_core::TextBlobId>,
        metrics: Option<fret_core::TextMetrics>,
        anchor: usize,
        caret: usize,
        affinity: CaretAffinity,
        pending_down: Option<(Point, bool)>,
        last_drag_pos: Option<Point>,
        dragging: bool,
    }

    impl Default for BidiState {
        fn default() -> Self {
            Self {
                selected_sample: 0,
                prepared_key: None,
                blob: None,
                metrics: None,
                anchor: 0,
                caret: 0,
                affinity: CaretAffinity::Downstream,
                pending_down: None,
                last_drag_pos: None,
                dragging: false,
            }
        }
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(BidiState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: sanity-check BiDi/RTL geometry queries (hit-test, caret rects, selection rects)."),
                cx.text("Use the selectable samples to validate editor-like selection behavior."),
                cx.text("Use the diagnostic panel to verify `hit_test_point` → caret/selection rendering under mixed-direction strings."),
            ]
        },
    );

    let sample_buttons = {
        let mut buttons: Vec<AnyElement> = Vec::new();
        for (i, s) in SAMPLES.iter().enumerate() {
            buttons.push(cx.keyed(format!("bidi-sample-btn-{i}"), |cx| {
                let state_for_click = state.clone();
                let is_selected = state.borrow().selected_sample == i;

                let variant = if is_selected {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Outline
                };

                let on_activate: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let mut st = state_for_click.borrow_mut();
                        st.selected_sample = i;
                        st.anchor = 0;
                        st.caret = 0;
                        st.affinity = CaretAffinity::Downstream;
                        st.pending_down = None;
                        st.last_drag_pos = None;
                        st.dragging = false;
                        host.request_redraw(action_cx.window);
                    });

                shadcn::Button::new(s.label)
                    .variant(variant)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_activate)
                    .into_element(cx)
            }));
        }

        let mut props = fret_ui::element::FlexProps::default();
        props.layout = fret_ui::element::LayoutStyle::default();
        props.layout.size.width = fret_ui::element::Length::Fill;
        props.direction = fret_core::Axis::Horizontal;
        props.wrap = true;
        props.gap = Px(8.0);
        props.align = fret_ui::element::CrossAlign::Start;
        props.justify = fret_ui::element::MainAlign::Start;

        cx.flex(props, move |_cx| buttons)
    };

    let selectable_samples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            let mut out: Vec<AnyElement> = Vec::new();
            out.push(cx.text("SelectableText samples:"));

            for (i, s) in SAMPLES.iter().enumerate() {
                out.push(cx.keyed(format!("bidi-sample-row-{i}"), |cx| {
                    let rich = AttributedText::new(
                        Arc::<str>::from(s.text),
                        Arc::<[TextSpan]>::from([TextSpan::new(s.text.len())]),
                    );

                    let mut props = fret_ui::element::SelectableTextProps::new(rich);
                    props.style = Some(TextStyle {
                        font: FontId::ui(),
                        size: Px(16.0),
                        ..Default::default()
                    });
                    props.wrap = TextWrap::None;
                    props.overflow = TextOverflow::Clip;
                    props.layout.size.width = fret_ui::element::Length::Fill;

                    let text = cx.selectable_text_props(props);

                    let row = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        |cx| {
                            vec![
                                cx.text_props(fret_ui::element::TextProps {
                                    layout: Default::default(),
                                    text: Arc::<str>::from(format!("{}:", s.label)),
                                    style: None,
                                    color: Some(theme.color_required("muted-foreground")),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                                cx.container(
                                    decl_style::container_props(
                                        theme,
                                        ChromeRefinement::default()
                                            .border_1()
                                            .rounded(Radius::Md)
                                            .p(Space::N2)
                                            .bg(ColorRef::Color(
                                                theme.color_required("background"),
                                            )),
                                        LayoutRefinement::default().w_full(),
                                    ),
                                    move |_cx| vec![text],
                                ),
                            ]
                        },
                    );

                    row
                }));
            }

            out
        },
    );

    let diagnostic = {
        let state_for_handlers = state.clone();
        let on_down: fret_ui::action::OnPointerDown = Arc::new(move |host, action_cx, down| {
            let mut st = state_for_handlers.borrow_mut();
            st.pending_down = Some((down.position, down.modifiers.shift));
            st.last_drag_pos = Some(down.position);
            st.dragging = true;
            host.invalidate(fret_ui::Invalidation::Paint);
            host.request_redraw(action_cx.window);
            true
        });

        let state_for_handlers = state.clone();
        let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, action_cx, mv| {
            let mut st = state_for_handlers.borrow_mut();
            if st.dragging && mv.buttons.left {
                st.last_drag_pos = Some(mv.position);
                host.invalidate(fret_ui::Invalidation::Paint);
                host.request_redraw(action_cx.window);
            }
            true
        });

        let state_for_handlers = state.clone();
        let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, action_cx, _up| {
            let mut st = state_for_handlers.borrow_mut();
            st.dragging = false;
            host.invalidate(fret_ui::Invalidation::Paint);
            host.request_redraw(action_cx.window);
            true
        });

        cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .bg(ColorRef::Color(theme.color_required("background"))),
                LayoutRefinement::default()
                    .w_full()
                    .h_px(MetricRef::Px(Px(220.0))),
            ),
            move |cx| {
                let mut pointer = fret_ui::element::PointerRegionProps::default();
                pointer.layout.size.width = fret_ui::element::Length::Fill;
                pointer.layout.size.height = fret_ui::element::Length::Fill;
                pointer.layout.overflow = fret_ui::element::Overflow::Clip;

                let paint_state = state.clone();

                let content = cx.pointer_region(pointer, move |cx| {
                    cx.pointer_region_on_pointer_down(on_down.clone());
                    cx.pointer_region_on_pointer_move(on_move.clone());
                    cx.pointer_region_on_pointer_up(on_up.clone());

                    let mut canvas = CanvasProps::default();
                    canvas.layout.size.width = fret_ui::element::Length::Fill;
                    canvas.layout.size.height = fret_ui::element::Length::Fill;
                    canvas.layout.overflow = fret_ui::element::Overflow::Clip;
                    canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

                    let canvas = cx.canvas(canvas, move |p| {
                        fn format_utf8_context(text: &str, index: usize) -> String {
                            let idx = index.min(text.len());
                            let mut prev = 0usize;
                            let mut next = text.len();

                            for (i, _) in text.char_indices() {
                                if i <= idx {
                                    prev = i;
                                }
                                if i >= idx {
                                    next = i;
                                    break;
                                }
                            }

                            let left = text[..prev].chars().rev().take(12).collect::<String>();
                            let left = left.chars().rev().collect::<String>();
                            let right = text[next..].chars().take(12).collect::<String>();
                            format!("{left}|{right}")
                        }

                        let bounds = p.bounds();
                        let pad = Px(12.0);

                        let inner = Rect::new(
                            Point::new(
                                Px(bounds.origin.x.0 + pad.0),
                                Px(bounds.origin.y.0 + pad.0),
                            ),
                            Size::new(
                                Px((bounds.size.width.0 - 2.0 * pad.0).max(0.0)),
                                Px((bounds.size.height.0 - 2.0 * pad.0).max(0.0)),
                            ),
                        );

                        let max_width = inner.size.width;
                        if max_width.0 <= 0.0 || inner.size.height.0 <= 0.0 {
                            return;
                        }

                        let scale_factor = p.scale_factor();
                        let selection_bg = p.theme().color_required("selection.background");
                        let fg = p.theme().color_required("foreground");
                        let muted = p.theme().color_required("muted-foreground");

                        let (stats, stats_origin) = {
                            let (services, scene) = p.services_and_scene();
                            let mut st = paint_state.borrow_mut();

                            let sample = SAMPLES
                                .get(st.selected_sample)
                                .copied()
                                .unwrap_or(SAMPLES[0]);

                            let key = PreparedKey {
                                sample: st.selected_sample,
                                max_width_bits: max_width.0.to_bits(),
                                scale_bits: scale_factor.to_bits(),
                            };

                            let needs_prepare = st.blob.is_none()
                                || st.metrics.is_none()
                                || st.prepared_key != Some(key);
                            if needs_prepare {
                                if let Some(blob) = st.blob.take() {
                                    services.text().release(blob);
                                }

                                let style = TextStyle {
                                    font: FontId::ui(),
                                    size: Px(18.0),
                                    ..Default::default()
                                };

                                let constraints = TextConstraints {
                                    max_width: Some(max_width),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                    scale_factor,
                                };

                                let (blob, metrics) =
                                    services.text().prepare_str(sample.text, &style, constraints);
                                st.prepared_key = Some(key);
                                st.blob = Some(blob);
                                st.metrics = Some(metrics);
                                st.anchor = 0;
                                st.caret = 0;
                                st.affinity = CaretAffinity::Downstream;
                            }

                            let Some(blob) = st.blob else {
                                return;
                            };
                            let Some(metrics) = st.metrics else {
                                return;
                            };

                            let click_to_local = |global: Point| -> Point {
                                Point::new(
                                    Px(global.x.0 - inner.origin.x.0),
                                    Px(global.y.0 - inner.origin.y.0),
                                )
                            };

                            if let Some((pos, extend)) = st.pending_down.take() {
                                let local = click_to_local(pos);
                                let hit = services.hit_test_point(blob, local);
                                st.caret = hit.index;
                                st.affinity = hit.affinity;
                                if !extend {
                                    st.anchor = st.caret;
                                }
                            }

                            if st.dragging {
                                if let Some(pos) = st.last_drag_pos {
                                    let local = click_to_local(pos);
                                    let hit = services.hit_test_point(blob, local);
                                    st.caret = hit.index;
                                    st.affinity = hit.affinity;
                                }
                            }

                            let range = if st.anchor <= st.caret {
                                (st.anchor, st.caret)
                            } else {
                                (st.caret, st.anchor)
                            };

                            let clip = Rect::new(Point::new(Px(0.0), Px(0.0)), inner.size);
                            let mut rects: Vec<Rect> = Vec::new();
                            services.selection_rects_clipped(blob, range, clip, &mut rects);

                            scene.push(SceneOp::PushClipRect { rect: inner });
                            for r in rects {
                                let rect = Rect::new(
                                    Point::new(
                                        Px(inner.origin.x.0 + r.origin.x.0),
                                        Px(inner.origin.y.0 + r.origin.y.0),
                                    ),
                                    r.size,
                                );
                                scene.push(SceneOp::Quad {
                                    order: DrawOrder(0),
                                    rect,
                                    background: fret_core::Paint::Solid(selection_bg),

                                    border: Edges::all(Px(0.0)),
                                    border_paint: fret_core::Paint::TRANSPARENT,

                                    corner_radii: Corners::all(Px(0.0)),
                                });
                            }

                            let text_origin = Point::new(inner.origin.x, Px(inner.origin.y.0 + metrics.baseline.0));
                            scene.push(SceneOp::Text {
                                order: DrawOrder(1),
                                origin: text_origin,
                                text: blob,
                                color: fg,
                            });

                            let caret_rect = services.caret_rect(blob, st.caret, st.affinity);
                            let caret_rect = Rect::new(
                                Point::new(
                                    Px(inner.origin.x.0 + caret_rect.origin.x.0),
                                    Px(inner.origin.y.0 + caret_rect.origin.y.0),
                                ),
                                caret_rect.size,
                            );
                            scene.push(SceneOp::Quad {
                                order: DrawOrder(2),
                                rect: caret_rect,
                                background: fret_core::Paint::Solid(fg),

                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT,

                                corner_radii: Corners::all(Px(0.0)),
                            });

                            if let Some(pos) = st.last_drag_pos {
                                let dot = Rect::new(
                                    Point::new(Px(pos.x.0 - 2.0), Px(pos.y.0 - 2.0)),
                                    Size::new(Px(4.0), Px(4.0)),
                                );
                                scene.push(SceneOp::Quad {
                                    order: DrawOrder(3),
                                    rect: dot,
                                    background: fret_core::Paint::Solid(fg),

                                    border: Edges::all(Px(0.0)),
                                    border_paint: fret_core::Paint::TRANSPARENT,

                                    corner_radii: Corners::all(Px(2.0)),
                                });
                            }

                            scene.push(SceneOp::PopClip);

                            let sample_text: &str = sample.text;
                            let context = format_utf8_context(sample_text, st.caret);

                            let stats = format!(
                                "sample: {} | caret: {} ({:?}) | anchor: {} | range: {:?} | context: {}",
                                sample.label, st.caret, st.affinity, st.anchor, range, context
                            );
                            let stats_origin = Point::new(
                                Px(bounds.origin.x.0 + 12.0),
                                Px(bounds.origin.y.0 + 10.0),
                            );
                            (stats, stats_origin)
                        };

                        let stats_style = TextStyle {
                            font: FontId::ui(),
                            size: Px(12.0),
                            ..Default::default()
                        };
                        let _ = p.text(
                            p.key(&"text_bidi_rtl_conformance_stats"),
                            DrawOrder(10),
                            stats_origin,
                            stats,
                            stats_style,
                            muted,
                            fret_ui::canvas::CanvasTextConstraints::default(),
                            scale_factor,
                        );
                    });

                    vec![canvas]
                });

                vec![content]
            },
        )
    };

    let panel = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |_cx| vec![sample_buttons, selectable_samples, diagnostic],
    )
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-text-bidi-rtl-conformance-root"),
    );

    vec![header, panel]
}

pub(in crate::ui) fn preview_web_ime_harness(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    text_input: Model<String>,
    text_area: Model<String>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ImeHarnessState {
        committed: String,
        preedit: Option<String>,
        ime_enabled: bool,
        text_input_count: u64,
        ime_commit_count: u64,
        ime_preedit_count: u64,
        ime_delete_surrounding_count: u64,
        ime_enabled_count: u64,
        ime_disabled_count: u64,
        last: String,
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(ImeHarnessState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: validate the wasm textarea IME bridge (ADR 0180)."),
                cx.text("Try: CJK IME preedit → commit; ensure no double insert on compositionend + input."),
                cx.text("Click inside the region to focus it (IME should enable)."),
            ]
        },
    );

    let inputs = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default().w_full(),
        ),
        |cx| {
            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N2),
                |cx| {
                    vec![
                        cx.text("Editable widgets (sanity check):"),
                        shadcn::Input::new(text_input)
                            .a11y_label("Web IME input")
                            .placeholder("Type here (IME should work on web)")
                            .into_element(cx),
                        shadcn::Textarea::new(text_area)
                            .a11y_label("Web IME textarea")
                            .into_element(cx),
                    ]
                },
            );
            vec![body]
        },
    );

    let mut region_props = fret_ui::element::TextInputRegionProps::default();
    region_props.layout.size.width = fret_ui::element::Length::Fill;
    region_props.layout.size.height = fret_ui::element::Length::Fill;

    let region = cx.text_input_region(region_props, |cx| {
        let state_for_text_input = state.clone();
        cx.text_input_region_on_text_input(std::sync::Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  text: &str| {
                let mut st = state_for_text_input.borrow_mut();
                st.text_input_count = st.text_input_count.saturating_add(1);
                st.last = format!("TextInput({:?})", text);
                st.committed.push_str(text);
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
                true
            },
        ));

        let state_for_ime = state.clone();
        cx.text_input_region_on_ime(std::sync::Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  ime: &fret_core::ImeEvent| {
                let mut st = state_for_ime.borrow_mut();
                match ime {
                    fret_core::ImeEvent::Enabled => {
                        st.ime_enabled = true;
                        st.ime_enabled_count = st.ime_enabled_count.saturating_add(1);
                        st.last = "Ime(Enabled)".to_string();
                    }
                    fret_core::ImeEvent::Disabled => {
                        st.ime_enabled = false;
                        st.preedit = None;
                        st.ime_disabled_count = st.ime_disabled_count.saturating_add(1);
                        st.last = "Ime(Disabled)".to_string();
                    }
                    fret_core::ImeEvent::Commit(text) => {
                        st.ime_commit_count = st.ime_commit_count.saturating_add(1);
                        st.last = format!("Ime(Commit({:?}))", text);
                        st.committed.push_str(text);
                        st.preedit = None;
                    }
                    fret_core::ImeEvent::Preedit { text, .. } => {
                        st.ime_preedit_count = st.ime_preedit_count.saturating_add(1);
                        st.last = format!("Ime(Preedit({:?}))", text);
                        st.preedit = (!text.is_empty()).then(|| text.clone());
                    }
                    fret_core::ImeEvent::DeleteSurrounding {
                        before_bytes,
                        after_bytes,
                    } => {
                        st.ime_delete_surrounding_count =
                            st.ime_delete_surrounding_count.saturating_add(1);
                        st.last = format!(
                            "Ime(DeleteSurrounding(before_bytes={before_bytes}, after_bytes={after_bytes}))"
                        );
                    }
                }

                host.notify(action_cx);
                host.request_redraw(action_cx.window);
                true
            },
        ));

        let st = state.borrow();
        let committed_tail = {
            const MAX_CHARS: usize = 120;
            let total = st.committed.chars().count();
            if total <= MAX_CHARS {
                st.committed.clone()
            } else {
                let tail: String = st
                    .committed
                    .chars()
                    .skip(total.saturating_sub(MAX_CHARS))
                    .collect();
                format!("…{tail}")
            }
        };

        let preedit = st
            .preedit
            .as_deref()
            .unwrap_or("<none>");
        let harness_region_ime_enabled = st.ime_enabled as u8;

        let panel = cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .bg(ColorRef::Color(theme.color_required("background"))),
                LayoutRefinement::default()
                    .w_full()
                    .h_px(MetricRef::Px(Px(240.0))),
            ),
            |cx| {
                let body = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N2),
                    |cx| {
                        let mut lines = vec![
                            cx.text(format!(
                                "harness_region_ime_enabled={harness_region_ime_enabled}"
                            )),
                            cx.text(format!("preedit={preedit:?}")),
                            cx.text(format!("committed_tail={committed_tail:?}")),
                            cx.text(format!("last_event={:?}", st.last)),
                            cx.text("Console logging: add ?ime_debug=1 or set window.__FRET_IME_DEBUG=true"),
                            cx.text(format!(
                                "counts: text_input={} ime_commit={} ime_preedit={} ime_delete_surrounding={} enabled={} disabled={}",
                                st.text_input_count,
                                st.ime_commit_count,
                                st.ime_preedit_count,
                                st.ime_delete_surrounding_count,
                                st.ime_enabled_count,
                                st.ime_disabled_count
                            )),
                        ];

                        if let Some(snapshot) = cx
                            .app
                            .global::<fret_runtime::WindowTextInputSnapshotService>()
                            .and_then(|svc| svc.snapshot(cx.window))
                            .cloned()
                        {
                            lines.push(cx.text("window_text_input_snapshot:"));
                            lines.push(cx.text(format!(
                                "  focus_is_text_input={} is_composing={}",
                                snapshot.focus_is_text_input as u8, snapshot.is_composing as u8
                            )));
                            lines.push(cx.text(format!(
                                "  text_len_utf16={} selection_utf16={:?} marked_utf16={:?}",
                                snapshot.text_len_utf16, snapshot.selection_utf16, snapshot.marked_utf16
                            )));
                            lines.push(cx.text(format!(
                                "  ime_cursor_area={:?}",
                                snapshot.ime_cursor_area
                            )));
                        } else {
                            lines.push(cx.text("window_text_input_snapshot: <unavailable>"));
                        }

                        if let Some(input_ctx) = cx
                            .app
                            .global::<fret_runtime::WindowInputContextService>()
                            .and_then(|svc| svc.snapshot(cx.window))
                            .cloned()
                        {
                            lines.push(cx.text("window_input_context_snapshot:"));
                            lines.push(cx.text(format!(
                                "  focus_is_text_input={} text_boundary_mode={:?}",
                                input_ctx.focus_is_text_input as u8, input_ctx.text_boundary_mode
                            )));
                        } else {
                            lines.push(cx.text("window_input_context_snapshot: <unavailable>"));
                        }

                        if let Some(key) = cx.app.global::<fret_runtime::TextFontStackKey>() {
                            lines.push(cx.text(format!("text_font_stack_key={}", key.0)));
                        } else {
                            lines.push(cx.text("text_font_stack_key: <unavailable>"));
                        }

                        if let Some(cfg) = cx.app.global::<fret_core::TextFontFamilyConfig>().cloned()
                        {
                            let fmt = |v: &[String]| -> String {
                                let head = v.iter().take(4).cloned().collect::<Vec<_>>().join(", ");
                                if v.len() > 4 {
                                    format!("[{head}, …] (len={})", v.len())
                                } else {
                                    format!("[{head}] (len={})", v.len())
                                }
                            };
                            lines.push(cx.text("text_font_families:"));
                            lines.push(cx.text(format!("  ui_sans={}", fmt(&cfg.ui_sans))));
                            lines.push(cx.text(format!("  ui_serif={}", fmt(&cfg.ui_serif))));
                            lines.push(cx.text(format!("  ui_mono={}", fmt(&cfg.ui_mono))));
                            lines.push(cx.text(format!(
                                "  common_fallback={}",
                                fmt(&cfg.common_fallback)
                            )));
                        } else {
                            lines.push(cx.text("text_font_families: <unavailable>"));
                        }

                        if let Some(catalog) = cx.app.global::<fret_runtime::FontCatalog>().cloned()
                        {
                            let head = catalog
                                .families
                                .iter()
                                .take(6)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ");
                            lines.push(cx.text("font_catalog:"));
                            lines.push(cx.text(format!(
                                "  revision={} families_len={}",
                                catalog.revision,
                                catalog.families.len()
                            )));
                            if !catalog.families.is_empty() {
                                lines.push(cx.text(format!("  head=[{head}]")));
                            }
                        } else {
                            lines.push(cx.text("font_catalog: <unavailable>"));
                        }

                        let snapshot = cx
                            .app
                            .global::<fret_core::input::WebImeBridgeDebugSnapshot>()
                            .cloned();
                        if let Some(snapshot) = snapshot {
                            lines.push(cx.text("bridge_debug_snapshot (wasm textarea):"));
                            lines.push(cx.text(format!(
                                "  enabled={} composing={} suppress_next_input={}",
                                snapshot.enabled as u8,
                                snapshot.composing as u8,
                                snapshot.suppress_next_input as u8
                            )));
                            lines.push(cx.text(format!(
                                "  last_preedit_text={:?} preedit_cursor_utf16={:?}",
                                snapshot.last_preedit_text.as_deref(),
                                snapshot.last_preedit_cursor_utf16
                            )));
                            lines.push(cx.text(format!(
                                "  last_commit_text={:?}",
                                snapshot.last_commit_text.as_deref()
                            )));
                            lines.push(cx.text(format!(
                                "  position_mode={:?} mount_kind={:?} dpr={:?}",
                                snapshot.position_mode.as_deref(),
                                snapshot.mount_kind.as_deref(),
                                snapshot.device_pixel_ratio,
                            )));
                            lines.push(cx.text(format!(
                                "  textarea_has_focus={:?} active_element_tag={:?}",
                                snapshot.textarea_has_focus, snapshot.active_element_tag
                            )));
                            lines.push(cx.text(format!(
                                "  last_input_type={:?}",
                                snapshot.last_input_type.as_deref()
                            )));
                            lines.push(cx.text(format!(
                                "  last_beforeinput_data={:?}",
                                snapshot.last_beforeinput_data.as_deref()
                            )));
                            lines.push(cx.text(format!(
                                "  last_input_data={:?}",
                                snapshot.last_input_data.as_deref()
                            )));
                            lines.push(cx.text(format!(
                                "  last_key_code={:?} last_cursor_area={:?}",
                                snapshot.last_key_code, snapshot.last_cursor_area
                            )));
                            lines.push(cx.text(format!(
                                "  last_cursor_anchor_px={:?}",
                                snapshot.last_cursor_anchor_px
                            )));
                            lines.push(cx.text(format!(
                                "  counts: beforeinput={} input={} suppressed={} comp_start={} comp_update={} comp_end={} cursor_area_set={}",
                                snapshot.beforeinput_seen,
                                snapshot.input_seen,
                                snapshot.suppressed_input_seen,
                                snapshot.composition_start_seen,
                                snapshot.composition_update_seen,
                                snapshot.composition_end_seen,
                                snapshot.cursor_area_set_seen,
                            )));
                            lines.push(cx.text(format!(
                                "  textarea: chars={:?} sel_utf16={:?}..{:?} client={:?}x{:?} scroll={:?}x{:?}",
                                snapshot.textarea_value_chars,
                                snapshot.textarea_selection_start_utf16,
                                snapshot.textarea_selection_end_utf16,
                                snapshot.textarea_client_width_px,
                                snapshot.textarea_client_height_px,
                                snapshot.textarea_scroll_width_px,
                                snapshot.textarea_scroll_height_px,
                            )));

                            if !snapshot.recent_events.is_empty() {
                                lines.push(cx.text("  recent_events:"));
                                for e in snapshot.recent_events.iter().rev().take(10) {
                                    lines.push(cx.text(format!("    {e}")));
                                }
                            }
                        } else {
                            lines.push(cx.text("bridge_debug_snapshot: <unavailable>"));
                        }

                        lines
                    },
                );
                vec![body]
            },
        );

        vec![panel]
    });

    vec![header, inputs, region]
}

pub(in crate::ui) fn preview_text_measure_overlay(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Clone, Copy)]
    struct Case {
        label: &'static str,
        text: &'static str,
        wrap: TextWrap,
        overflow: TextOverflow,
        height: Px,
    }

    const CASES: &[Case] = &[
        Case {
            label: "Wrap=None, Overflow=Clip (expect overflow past measured width)",
            text: "Left (fill) • A_very_long_token_without_spaces_that_should_not_wrap_but_can_overflow_the_box",
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            height: Px(56.0),
        },
        Case {
            label: "Wrap=Word, Overflow=Clip (expect multi-line height growth)",
            text: "Word wrap should break on spaces and increase measured height when max_width is tight.",
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            height: Px(88.0),
        },
        Case {
            label: "Wrap=Grapheme, Overflow=Clip (expect long tokens to wrap)",
            text: "GraphemeWrap: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa (and emoji 😀😀😀) should wrap without whitespace.",
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            height: Px(88.0),
        },
        Case {
            label: "Wrap=None, Overflow=Ellipsis (expect measured width ~= max_width)",
            text: "Ellipsis overflow should clamp the visual width and replace the suffix…",
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            height: Px(56.0),
        },
    ];

    #[derive(Default)]
    struct MeasureOverlayState {
        last_metrics: Vec<Option<fret_core::TextMetrics>>,
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(MeasureOverlayState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: visualize measured text bounds vs allocated container bounds."),
                cx.text("Green = container bounds; Yellow = measured TextMetrics.size; Cyan = baseline."),
            ]
        },
    );

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(440.0))),
        ),
        move |cx| {
            let mut canvas = CanvasProps::default();
            canvas.layout.size.width = fret_ui::element::Length::Fill;
            canvas.layout.size.height = fret_ui::element::Length::Fill;
            canvas.layout.overflow = fret_ui::element::Overflow::Clip;
            canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

            let paint_state = state.clone();

            let canvas = cx.canvas(canvas, move |p| {
                let bounds = p.bounds();
                let pad = Px(14.0);
                let gap = Px(14.0);

                let outer = Rect::new(
                    Point::new(Px(bounds.origin.x.0 + pad.0), Px(bounds.origin.y.0 + pad.0)),
                    Size::new(
                        Px((bounds.size.width.0 - 2.0 * pad.0).max(0.0)),
                        Px((bounds.size.height.0 - 2.0 * pad.0).max(0.0)),
                    ),
                );
                if outer.size.width.0 <= 0.0 || outer.size.height.0 <= 0.0 {
                    return;
                }

                let green = fret_core::Color {
                    r: 0.20,
                    g: 0.85,
                    b: 0.35,
                    a: 1.0,
                };
                let yellow = fret_core::Color {
                    r: 0.95,
                    g: 0.85,
                    b: 0.10,
                    a: 1.0,
                };
                let cyan = fret_core::Color {
                    r: 0.10,
                    g: 0.80,
                    b: 0.95,
                    a: 1.0,
                };

                let fg = p.theme().color_required("foreground");
                let muted = p.theme().color_required("muted-foreground");
                let bg = p.theme().color_required("background");
                let border = p.theme().color_required("border");

                let scale = p.scale_factor();
                let mut y = outer.origin.y;
                let scope = p.key_scope(&"text_measure_overlay");

                let mut st = paint_state.borrow_mut();
                if st.last_metrics.len() < CASES.len() {
                    st.last_metrics.resize(CASES.len(), None);
                }

                for (i, case) in CASES.iter().enumerate() {
                    let case_rect = Rect::new(
                        Point::new(outer.origin.x, y),
                        Size::new(outer.size.width, case.height),
                    );

                    // Case chrome.
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: case_rect,
                        background: fret_core::Paint::Solid(bg),

                        border: Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(border),

                        corner_radii: Corners::all(Px(8.0)),
                    });

                    let label_style = TextStyle {
                        font: FontId::ui(),
                        size: Px(12.0),
                        ..Default::default()
                    };
                    let label_metrics = p.text(
                        p.child_key(scope, &format!("label_{i}")).0,
                        DrawOrder(1),
                        Point::new(case_rect.origin.x + Px(10.0), case_rect.origin.y + Px(16.0)),
                        case.label,
                        label_style,
                        muted,
                        fret_ui::canvas::CanvasTextConstraints {
                            max_width: Some(Px((case_rect.size.width.0 - 20.0).max(0.0))),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        },
                        scale,
                    );

                    let text_box = Rect::new(
                        Point::new(
                            case_rect.origin.x + Px(10.0),
                            Px(case_rect.origin.y.0 + 16.0 + label_metrics.size.height.0 + 8.0),
                        ),
                        Size::new(
                            Px((case_rect.size.width.0 - 20.0).max(0.0)),
                            Px((case_rect.size.height.0
                                - 16.0
                                - label_metrics.size.height.0
                                - 18.0)
                                .max(0.0)),
                        ),
                    );

                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: text_box,
                        background: fret_core::Paint::TRANSPARENT,

                        border: Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(green),

                        corner_radii: Corners::all(Px(6.0)),
                    });

                    let text_style = TextStyle {
                        font: FontId::ui(),
                        size: Px(16.0),
                        ..Default::default()
                    };

                    let baseline_y = match st.last_metrics[i] {
                        Some(m) => text_box.origin.y + m.baseline,
                        None => text_box.origin.y + Px(text_style.size.0 * 0.8),
                    };

                    let metrics = p.text(
                        p.child_key(scope, &format!("text_{i}")).0,
                        DrawOrder(2),
                        Point::new(text_box.origin.x, baseline_y),
                        case.text,
                        text_style,
                        fg,
                        fret_ui::canvas::CanvasTextConstraints {
                            max_width: Some(text_box.size.width),
                            wrap: case.wrap,
                            overflow: case.overflow,
                        },
                        scale,
                    );
                    st.last_metrics[i] = Some(metrics);

                    // Baseline.
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(3),
                        rect: Rect::new(
                            Point::new(text_box.origin.x, text_box.origin.y + metrics.baseline),
                            Size::new(text_box.size.width, Px(1.0)),
                        ),
                        background: fret_core::Paint::Solid(cyan),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,

                        corner_radii: Corners::all(Px(0.0)),
                    });

                    // Measured text box.
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(4),
                        rect: Rect::new(text_box.origin, metrics.size),
                        background: fret_core::Paint::TRANSPARENT,

                        border: Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(yellow),

                        corner_radii: Corners::all(Px(0.0)),
                    });

                    y = Px(y.0 + case.height.0 + gap.0);
                    if y.0 >= outer.origin.y.0 + outer.size.height.0 {
                        break;
                    }
                }
            });

            vec![canvas]
        },
    );

    let panel = panel.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-text-measure-overlay-root"),
    );

    vec![header, panel]
}

pub(in crate::ui) fn preview_chart_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use delinea::data::{Column, DataTable};
    use delinea::{
        AxisKind, AxisPointerSpec, AxisPointerTrigger, AxisPointerType, AxisRange, AxisScale,
        ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesKind, SeriesSpec,
        TimeAxisScale,
    };
    use fret_chart::ChartCanvas;
    use fret_ui::element::{LayoutStyle, Length, SemanticsProps};
    use fret_ui::retained_bridge::RetainedSubtreeProps;

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress canvas charts with pan/zoom (candidate for prepaint-windowed sampling)."),
                cx.text("Use scripted drag+wheel steps to validate correctness and collect perf bundles."),
            ]
        },
    );

    let chart =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let dataset_id = delinea::ids::DatasetId::new(1);
            let grid_id = delinea::ids::GridId::new(1);
            let x_axis = delinea::AxisId::new(1);
            let y_axis = delinea::AxisId::new(2);
            let series_id = delinea::ids::SeriesId::new(1);
            let x_field = delinea::FieldId::new(1);
            let y_field = delinea::FieldId::new(2);

            let spec = ChartSpec {
                id: delinea::ids::ChartId::new(1),
                viewport: None,
                datasets: vec![DatasetSpec {
                    id: dataset_id,
                    fields: vec![
                        FieldSpec {
                            id: x_field,
                            column: 0,
                        },
                        FieldSpec {
                            id: y_field,
                            column: 1,
                        },
                    ],
                    ..Default::default()
                }],
                grids: vec![GridSpec { id: grid_id }],
                axes: vec![
                    delinea::AxisSpec {
                        id: x_axis,
                        name: Some("Time".to_string()),
                        kind: AxisKind::X,
                        grid: grid_id,
                        position: None,
                        scale: AxisScale::Time(TimeAxisScale),
                        range: Some(AxisRange::Auto),
                    },
                    delinea::AxisSpec {
                        id: y_axis,
                        name: Some("Value".to_string()),
                        kind: AxisKind::Y,
                        grid: grid_id,
                        position: None,
                        scale: Default::default(),
                        range: Some(AxisRange::Auto),
                    },
                ],
                data_zoom_x: vec![],
                data_zoom_y: vec![],
                tooltip: None,
                axis_pointer: Some(AxisPointerSpec {
                    enabled: true,
                    trigger: AxisPointerTrigger::Axis,
                    pointer_type: AxisPointerType::Line,
                    label: Default::default(),
                    snap: false,
                    trigger_distance_px: 12.0,
                    throttle_px: 0.75,
                }),
                visual_maps: vec![],
                series: vec![SeriesSpec {
                    id: series_id,
                    name: Some("Series".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                }],
            };

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Px(Px(520.0));

            let props = RetainedSubtreeProps::new::<App>(move |ui| {
                use fret_ui::retained_bridge::UiTreeRetainedExt as _;

                let mut canvas =
                    ChartCanvas::new(spec.clone()).expect("chart spec should be valid");
                canvas.set_input_map(fret_chart::input_map::ChartInputMap::default());

                let base_ms = 1_735_689_600_000.0;
                let interval_ms = 60_000.0;

                let n = 200_000usize;
                let mut x: Vec<f64> = Vec::with_capacity(n);
                let mut y: Vec<f64> = Vec::with_capacity(n);
                for i in 0..n {
                    let t = i as f64 / (n - 1) as f64;
                    let xi = base_ms + interval_ms * i as f64;
                    let theta = t * std::f64::consts::TAU;
                    let yi = (theta * 8.0).sin() * 0.8;
                    x.push(xi);
                    y.push(yi);
                }

                let mut table = DataTable::default();
                table.push_column(Column::F64(x));
                table.push_column(Column::F64(y));
                canvas.engine_mut().datasets_mut().insert(dataset_id, table);

                let node = ui.create_node_retained(canvas);
                ui.set_node_view_cache_flags(node, true, true, false);
                node
            })
            .with_layout(layout);

            let subtree = cx.retained_subtree(props);
            vec![cx.semantics(
                SemanticsProps {
                    role: fret_core::SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-chart-torture-root")),
                    ..Default::default()
                },
                |_cx| vec![subtree],
            )]
        });

    vec![header, chart]
}

pub(in crate::ui) fn preview_canvas_cull_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_canvas::ui::{
        PanZoomCanvasSurfacePanelProps, PanZoomInputPreset, pan_zoom_canvas_surface_panel,
    };
    use fret_canvas::view::{PanZoom2D, visible_canvas_rect};
    use fret_core::{
        Corners, DrawOrder, Edges, FontId, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
    };
    use fret_ui::canvas::CanvasTextConstraints;
    use fret_ui::element::{CanvasCachePolicy, Length};
    use std::cmp::Ordering;

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress a pan/zoom canvas scene with viewport-driven culling (candidate for prepaint-windowed cull windows)."),
                cx.text("Use scripted middle-drag + wheel steps to validate correctness and collect perf bundles."),
            ]
        },
    );

    let canvas =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let bg_even = theme.color_required("background");
            let bg_odd = theme.color_required("muted");
            let fg = theme.color_required("foreground");
            let grid = theme.color_required("border");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: Px(11.0),
                ..Default::default()
            };

            let mut props = PanZoomCanvasSurfacePanelProps::default();
            props.preset = PanZoomInputPreset::DesktopCanvasCad;
            props.pointer_region.layout.size.width = Length::Fill;
            props.pointer_region.layout.size.height = Length::Px(Px(520.0));
            props.canvas.cache_policy = CanvasCachePolicy::smooth_default();
            props.default_view = PanZoom2D {
                pan: fret_core::Point::new(Px(0.0), Px(0.0)),
                zoom: 1.0,
            };
            props.min_zoom = 0.05;
            props.max_zoom = 64.0;

            let cell_size = 48.0f32;
            let cell_pad = 3.0f32;
            let max_cells = 40_000i64;

            let canvas = pan_zoom_canvas_surface_panel(cx, props, move |painter, paint_cx| {
                let bounds = painter.bounds();

                let Some(transform) = paint_cx.view.render_transform(bounds) else {
                    return;
                };

                let vis = visible_canvas_rect(bounds, paint_cx.view);
                let min_x = vis.origin.x.0;
                let max_x = vis.origin.x.0 + vis.size.width.0;
                let min_y = vis.origin.y.0;
                let max_y = vis.origin.y.0 + vis.size.height.0;

                let start_x = (min_x / cell_size).floor() as i32 - 2;
                let end_x = (max_x / cell_size).ceil() as i32 + 2;
                let start_y = (min_y / cell_size).floor() as i32 - 2;
                let end_y = (max_y / cell_size).ceil() as i32 + 2;

                let x_count = (end_x - start_x + 1).max(0) as i64;
                let y_count = (end_y - start_y + 1).max(0) as i64;
                let estimated = x_count.saturating_mul(y_count);

                let stride = match estimated.cmp(&max_cells) {
                    Ordering::Less | Ordering::Equal => 1i32,
                    Ordering::Greater => {
                        ((estimated as f64 / max_cells as f64).ceil() as i32).max(1)
                    }
                };

                let clip = bounds;
                painter.with_clip_rect(clip, |painter| {
                    painter.with_transform(transform, |painter| {
                        let scope = painter.key_scope(&"ui-gallery-canvas-cull");

                        let mut y = start_y;
                        while y <= end_y {
                            let mut x = start_x;
                            while x <= end_x {
                                let ox = x as f32 * cell_size + cell_pad;
                                let oy = y as f32 * cell_size + cell_pad;
                                let size = cell_size - cell_pad * 2.0;
                                if size.is_finite() && size > 0.0 {
                                    let rect = fret_core::Rect::new(
                                        fret_core::Point::new(Px(ox), Px(oy)),
                                        fret_core::Size::new(Px(size), Px(size)),
                                    );

                                    let background =
                                        if ((x ^ y) & 1) == 0 { bg_even } else { bg_odd };
                                    painter.scene().push(fret_core::SceneOp::Quad {
                                        order: DrawOrder(0),
                                        rect,
                                        background: fret_core::Paint::Solid(background),
                                        border: Edges::all(Px(1.0)),
                                        border_paint: fret_core::Paint::Solid(grid),

                                        corner_radii: Corners::all(Px(4.0)),
                                    });

                                    if x == 0 && y == 0 {
                                        painter.scene().push(fret_core::SceneOp::Quad {
                                            order: DrawOrder(1),
                                            rect,
                                            background: fret_core::Paint::TRANSPARENT,

                                            border: Edges::all(Px(2.0)),
                                            border_paint: fret_core::Paint::Solid(fg),

                                            corner_radii: Corners::all(Px(4.0)),
                                        });
                                    }

                                    if (x % 20) == 0 && (y % 20) == 0 {
                                        let key: u64 = painter.child_key(scope, &(x, y)).into();
                                        let label = format!("{x},{y}");
                                        let origin = fret_core::Point::new(
                                            Px(rect.origin.x.0 + 6.0),
                                            Px(rect.origin.y.0 + 6.0),
                                        );
                                        let _ = painter.text(
                                            key,
                                            DrawOrder(2),
                                            origin,
                                            label,
                                            text_style.clone(),
                                            fg,
                                            CanvasTextConstraints {
                                                max_width: Some(Px(
                                                    (rect.size.width.0 - 12.0).max(0.0)
                                                )),
                                                wrap: TextWrap::None,
                                                overflow: TextOverflow::Clip,
                                            },
                                            painter.scale_factor(),
                                        );
                                    }
                                }

                                x = x.saturating_add(stride);
                            }
                            y = y.saturating_add(stride);
                        }
                    });
                });
            });

            vec![
                canvas.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id("ui-gallery-canvas-cull-root"),
                ),
            ]
        });

    vec![header, canvas]
}

pub(in crate::ui) fn preview_node_graph_cull_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::{Px, SemanticsRole};
    use fret_node::io::NodeGraphViewState;
    use fret_node::ui::NodeGraphCanvas;
    use fret_node::{
        Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port, PortCapacity,
        PortDirection, PortId, PortKey, PortKind, TypeDesc,
    };
    use fret_ui::element::{LayoutStyle, Length, SemanticsProps};
    use fret_ui::retained_bridge::RetainedSubtreeProps;

    fn uuid_from_tag(tag: u64, ix: u64) -> uuid::Uuid {
        uuid::Uuid::from_u128(((tag as u128) << 64) | (ix as u128))
    }

    fn build_stress_graph(graph_id: GraphId, target_nodes: usize) -> Graph {
        let mut graph = Graph::new(graph_id);

        let add_nodes = target_nodes.saturating_sub(1) / 2;
        let float_nodes = add_nodes.saturating_add(1);

        let cols: usize = 64;
        let x_step = 360.0f32;
        let y_step = 220.0f32;

        let float_x_offset = -260.0f32;
        let float_y_offset = 40.0f32;

        let node_tag = u64::from_le_bytes(*b"NODEGRAF");
        let port_tag = u64::from_le_bytes(*b"PORTGRAF");
        let edge_tag = u64::from_le_bytes(*b"EDGEGRAF");

        let node_id = |ix: u64| NodeId(uuid_from_tag(node_tag, ix));
        let port_id = |ix: u64| PortId(uuid_from_tag(port_tag, ix));
        let edge_id = |ix: u64| EdgeId(uuid_from_tag(edge_tag, ix));

        let mut next_node_ix: u64 = 1;
        let mut next_port_ix: u64 = 1;
        let mut next_edge_ix: u64 = 1;

        let mut float_out_ports: Vec<PortId> = Vec::with_capacity(float_nodes);
        for i in 0..float_nodes {
            let node_id = {
                let id = node_id(next_node_ix);
                next_node_ix = next_node_ix.saturating_add(1);
                id
            };
            let port_out = {
                let id = port_id(next_port_ix);
                next_port_ix = next_port_ix.saturating_add(1);
                id
            };

            let col = i % cols;
            let row = i / cols;
            let x = col as f32 * x_step + float_x_offset;
            let y = row as f32 * y_step + float_y_offset;
            let value = (i as f64) * 0.001;

            graph.nodes.insert(
                node_id,
                Node {
                    kind: NodeKindKey::new("demo.float"),
                    kind_version: 1,
                    pos: fret_node::CanvasPoint { x, y },
                    selectable: None,
                    draggable: None,
                    connectable: None,
                    deletable: None,
                    parent: None,
                    extent: None,
                    expand_parent: None,
                    size: None,
                    hidden: false,
                    collapsed: false,
                    ports: vec![port_out],
                    data: serde_json::json!({ "value": value }),
                },
            );
            graph.ports.insert(
                port_out,
                Port {
                    node: node_id,
                    key: PortKey::new("out"),
                    dir: PortDirection::Out,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Multi,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: Some(TypeDesc::Float),
                    data: serde_json::Value::Null,
                },
            );

            float_out_ports.push(port_out);
        }

        let mut prev_out: Option<PortId> = None;
        for i in 0..add_nodes {
            let node_id = {
                let id = node_id(next_node_ix);
                next_node_ix = next_node_ix.saturating_add(1);
                id
            };
            let port_a = {
                let id = port_id(next_port_ix);
                next_port_ix = next_port_ix.saturating_add(1);
                id
            };
            let port_b = {
                let id = port_id(next_port_ix);
                next_port_ix = next_port_ix.saturating_add(1);
                id
            };
            let port_out = {
                let id = port_id(next_port_ix);
                next_port_ix = next_port_ix.saturating_add(1);
                id
            };

            let col = i % cols;
            let row = i / cols;
            let x = col as f32 * x_step;
            let y = row as f32 * y_step;

            graph.nodes.insert(
                node_id,
                Node {
                    kind: NodeKindKey::new("demo.add"),
                    kind_version: 1,
                    pos: fret_node::CanvasPoint { x, y },
                    selectable: None,
                    draggable: None,
                    connectable: None,
                    deletable: None,
                    parent: None,
                    extent: None,
                    expand_parent: None,
                    size: None,
                    hidden: false,
                    collapsed: false,
                    ports: vec![port_a, port_b, port_out],
                    data: serde_json::Value::Null,
                },
            );
            graph.ports.insert(
                port_a,
                Port {
                    node: node_id,
                    key: PortKey::new("a"),
                    dir: PortDirection::In,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: Some(TypeDesc::Float),
                    data: serde_json::Value::Null,
                },
            );
            graph.ports.insert(
                port_b,
                Port {
                    node: node_id,
                    key: PortKey::new("b"),
                    dir: PortDirection::In,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: Some(TypeDesc::Float),
                    data: serde_json::Value::Null,
                },
            );
            graph.ports.insert(
                port_out,
                Port {
                    node: node_id,
                    key: PortKey::new("out"),
                    dir: PortDirection::Out,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Multi,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: Some(TypeDesc::Float),
                    data: serde_json::Value::Null,
                },
            );

            let port_a_source = prev_out.unwrap_or(float_out_ports[0]);
            let port_b_source =
                float_out_ports[(i + 1).min(float_out_ports.len().saturating_sub(1))];

            let edge_a = edge_id(next_edge_ix);
            next_edge_ix = next_edge_ix.saturating_add(1);
            graph.edges.insert(
                edge_a,
                Edge {
                    kind: EdgeKind::Data,
                    from: port_a_source,
                    to: port_a,
                    selectable: None,
                    deletable: None,
                    reconnectable: None,
                },
            );

            let edge_b = edge_id(next_edge_ix);
            next_edge_ix = next_edge_ix.saturating_add(1);
            graph.edges.insert(
                edge_b,
                Edge {
                    kind: EdgeKind::Data,
                    from: port_b_source,
                    to: port_b,
                    selectable: None,
                    deletable: None,
                    reconnectable: None,
                },
            );

            prev_out = Some(port_out);
        }

        graph
    }

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress a large node-graph canvas with viewport-driven culling (candidate for prepaint-windowed cull windows)."),
                cx.text("Use scripted middle-drag + wheel steps to validate correctness and collect perf bundles."),
            ]
        },
    );

    #[derive(Default)]
    struct HarnessState {
        graph: Option<Model<Graph>>,
        view: Option<Model<NodeGraphViewState>>,
    }

    let existing = cx.with_state(HarnessState::default, |st| {
        match (st.graph.clone(), st.view.clone()) {
            (Some(graph), Some(view)) => Some((graph, view)),
            _ => None,
        }
    });

    let (graph, view) = if let Some((graph, view)) = existing {
        (graph, view)
    } else {
        let graph_id = GraphId::from_u128(1);
        let graph = build_stress_graph(graph_id, 8_000);
        let graph = cx.app.models_mut().insert(graph);
        let view = cx.app.models_mut().insert(NodeGraphViewState::default());

        cx.with_state(HarnessState::default, |st| {
            st.graph = Some(graph.clone());
            st.view = Some(view.clone());
        });

        (graph, view)
    };

    let surface =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let graph = graph.clone();
            let view = view.clone();

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Px(Px(520.0));

            let props = RetainedSubtreeProps::new::<App>(move |ui| {
                use fret_ui::retained_bridge::UiTreeRetainedExt as _;
                let canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
                ui.create_node_retained(canvas)
            })
            .with_layout(layout);

            let subtree = cx.retained_subtree(props);
            vec![cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-node-graph-cull-root")),
                    ..Default::default()
                },
                |_cx| vec![subtree],
            )]
        });

    vec![header, surface]
}

pub(in crate::ui) fn preview_chrome_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: exercise hover/focus/pressed chrome under view-cache + shell."),
                cx.text(
                    "This page intentionally mixes many focusable widgets and overlay triggers.",
                ),
            ]
        },
    );

    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4),
        |cx| {
            let mut out = Vec::new();

            out.extend(preview_overlay(
                cx,
                popover_open,
                dialog_open,
                alert_dialog_open,
                sheet_open,
                portal_geometry_popover_open,
                dropdown_open,
                context_menu_open,
                context_menu_edge_open,
                last_action,
            ));

            let controls = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N3),
                |cx| {
                    let mut out: Vec<AnyElement> = Vec::new();

                    let row = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Button::new("One")
                                    .test_id("ui-gallery-chrome-btn-1")
                                    .into_element(cx),
                                shadcn::Button::new("Two")
                                    .variant(shadcn::ButtonVariant::Secondary)
                                    .test_id("ui-gallery-chrome-btn-2")
                                    .into_element(cx),
                                shadcn::Button::new("Three")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .test_id("ui-gallery-chrome-btn-3")
                                    .into_element(cx),
                                shadcn::Button::new("Disabled")
                                    .disabled(true)
                                    .test_id("ui-gallery-chrome-btn-disabled")
                                    .into_element(cx),
                            ]
                        },
                    );
                    out.push(row);

                    let fields = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_start(),
                        |cx| {
                            vec![
                                stack::vstack(
                                    cx,
                                    stack::VStackProps::default().gap(Space::N1),
                                    |cx| {
                                        let input = shadcn::Input::new(text_input.clone())
                                            .a11y_label("Chrome torture input")
                                            .placeholder("Type")
                                            .into_element(cx);
                                        let input = input.attach_semantics(
                                            SemanticsDecoration::default()
                                                .role(fret_core::SemanticsRole::TextField)
                                                .test_id("ui-gallery-chrome-text-input"),
                                        );
                                        vec![cx.text("Text input"), input]
                                    },
                                ),
                                stack::vstack(
                                    cx,
                                    stack::VStackProps::default().gap(Space::N1),
                                    |cx| {
                                        let textarea = shadcn::Textarea::new(text_area.clone())
                                            .a11y_label("Chrome torture textarea")
                                            .into_element(cx);
                                        let textarea = textarea.attach_semantics(
                                            SemanticsDecoration::default()
                                                .role(fret_core::SemanticsRole::TextField)
                                                .test_id("ui-gallery-chrome-text-area"),
                                        );
                                        vec![cx.text("Text area"), textarea]
                                    },
                                ),
                            ]
                        },
                    );
                    out.push(fields);

                    let toggles = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N3).items_center(),
                        |cx| {
                            vec![
                                shadcn::Checkbox::new(checkbox.clone())
                                    .a11y_label("Chrome torture checkbox")
                                    .test_id("ui-gallery-chrome-checkbox")
                                    .into_element(cx),
                                shadcn::Switch::new(switch.clone())
                                    .a11y_label("Chrome torture switch")
                                    .test_id("ui-gallery-chrome-switch")
                                    .into_element(cx),
                            ]
                        },
                    );
                    out.push(toggles);

                    out
                },
            );
            out.push(controls);

            out
        },
    );

    let content = body.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-chrome-torture-root"),
    );

    vec![header, content]
}

pub(in crate::ui) fn preview_windowed_rows_surface_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::{
        Corners, DrawOrder, Edges, FontId, SemanticsRole, TextOverflow, TextStyle, TextWrap,
    };
    use fret_ui::canvas::CanvasTextConstraints;
    use fret_ui_kit::declarative::windowed_rows_surface::{
        WindowedRowsSurfaceProps, windowed_rows_surface,
    };

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline scroll windowing via a stable element tree (Scroll + Canvas)."),
                cx.text("This is the 'single-node surface' escape hatch: paint only visible rows, avoid per-row subtrees."),
            ]
        },
    );

    let len = 200_000usize;
    let row_h = Px(22.0);
    let overscan = 16usize;

    let scroll_handle = cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());

    let surface =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let bg_even = theme.color_required("background");
            let bg_odd = theme.color_required("muted");
            let fg = theme.color_required("foreground");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: Px(12.0),
                ..Default::default()
            };

            let mut props = WindowedRowsSurfaceProps::default();
            props.scroll.layout.size.width = fret_ui::element::Length::Fill;
            props.scroll.layout.size.height = fret_ui::element::Length::Px(Px(420.0));
            props.scroll.layout.overflow = fret_ui::element::Overflow::Clip;
            props.len = len;
            props.row_height = row_h;
            props.overscan = overscan;
            props.scroll_handle = scroll_handle.clone();
            props.canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

            let surface = windowed_rows_surface(cx, props, move |painter, index, rect| {
                let background = if (index % 2) == 0 { bg_even } else { bg_odd };
                painter.scene().push(fret_core::SceneOp::Quad {
                    order: DrawOrder(0),
                    rect,
                    background: fret_core::Paint::Solid(background),
                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

                    corner_radii: Corners::all(Px(0.0)),
                });

                let label = format!("Row {index}");
                let origin =
                    fret_core::Point::new(Px(rect.origin.x.0 + 8.0), Px(rect.origin.y.0 + 4.0));
                let scope = painter.key_scope(&"ui-gallery-windowed-rows");
                let key: u64 = painter.child_key(scope, &index).into();
                let _ = painter.text(
                    key,
                    DrawOrder(1),
                    origin,
                    label,
                    text_style.clone(),
                    fg,
                    CanvasTextConstraints {
                        max_width: Some(Px(rect.size.width.0.max(0.0) - 16.0)),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    },
                    painter.scale_factor(),
                );
            });

            vec![
                surface.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id("ui-gallery-windowed-rows-root"),
                ),
            ]
        });

    vec![header, surface]
}

pub(in crate::ui) fn preview_windowed_rows_surface_interactive_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use std::cell::RefCell;
    use std::rc::Rc;

    use fret_core::{Corners, CursorIcon, DrawOrder, Edges, FontId, SemanticsRole, TextStyle};
    use fret_ui::Invalidation;
    use fret_ui::action::{ActionCx, PointerDownCx, PointerMoveCx};
    use fret_ui::canvas::CanvasTextConstraints;
    use fret_ui::element::{Length, PointerRegionProps, SemanticsProps};
    use fret_ui_kit::declarative::windowed_rows_surface::{
        WindowedRowsSurfacePointerHandlers, WindowedRowsSurfaceProps,
        windowed_rows_surface_with_pointer_region,
    };

    #[derive(Default)]
    struct RowChromeState {
        hovered: Option<usize>,
        selected: Option<usize>,
    }

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: demonstrate paint-only hover/selection chrome on a prepaint-windowed row surface (ADR 0175 + ADR 0166)."),
                cx.text("Pattern: stable tree (Scroll + PointerRegion + Canvas), row hit-testing in pointer hooks, paint-only visuals in Canvas."),
            ]
        },
    );

    let len = 200_000usize;
    let row_h = Px(22.0);
    let overscan = 16usize;

    let scroll_handle = cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());

    let surface =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let bg_even = theme.color_required("background");
            let bg_odd = theme.color_required("muted");
            let bg_hover = theme.color_required("accent");
            let fg = theme.color_required("foreground");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: Px(12.0),
                ..Default::default()
            };

            let root = cx.semantics_with_id(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-windowed-rows-interactive-root")),
                    ..Default::default()
                },
                move |cx, root_id| {
                    let state = cx.with_state_for(
                        root_id,
                        || Rc::new(RefCell::new(RowChromeState::default())),
                        |s| s.clone(),
                    );

                    let on_move_state = state.clone();
                    let on_pointer_move: fret_ui_kit::declarative::windowed_rows_surface::OnWindowedRowsPointerMove =
                        Arc::new(move |host, action_cx: ActionCx, idx, _mv: PointerMoveCx| {
                            host.set_cursor_icon(CursorIcon::Pointer);
                            let mut st = on_move_state.borrow_mut();
                            if st.hovered == idx {
                                return true;
                            }
                            st.hovered = idx;
                            host.invalidate(Invalidation::Paint);
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let on_down_state = state.clone();
                    let on_pointer_down: fret_ui_kit::declarative::windowed_rows_surface::OnWindowedRowsPointerDown =
                        Arc::new(move |host, action_cx: ActionCx, idx, down: PointerDownCx| {
                            if down.button != fret_core::MouseButton::Left {
                                return false;
                            }
                            let mut st = on_down_state.borrow_mut();
                            st.selected = Some(idx);
                            st.hovered = Some(idx);
                            host.invalidate(Invalidation::Paint);
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let handlers = WindowedRowsSurfacePointerHandlers {
                        on_pointer_down: Some(on_pointer_down),
                        on_pointer_move: Some(on_pointer_move),
                        ..Default::default()
                    };

                    let mut props = WindowedRowsSurfaceProps::default();
                    props.scroll.layout.size.width = Length::Fill;
                    props.scroll.layout.size.height = Length::Px(Px(420.0));
                    props.scroll.layout.overflow = fret_ui::element::Overflow::Clip;
                    props.len = len;
                    props.row_height = row_h;
                    props.overscan = overscan;
                    props.scroll_handle = scroll_handle.clone();
                    props.canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

                    let mut pointer = PointerRegionProps::default();
                    pointer.layout.size.width = Length::Fill;
                    pointer.layout.size.height = Length::Fill;

                    let paint_state = state.clone();
                    let content_semantics = SemanticsProps {
                        role: SemanticsRole::Group,
                        test_id: Some(Arc::<str>::from(
                            "ui-gallery-windowed-rows-interactive-canvas",
                        )),
                        ..Default::default()
                    };

                    vec![windowed_rows_surface_with_pointer_region(
                        cx,
                        props,
                        pointer,
                        handlers,
                        Some(content_semantics),
                        move |painter, index, rect| {
                            let st = paint_state.borrow();
                            let hovered = st.hovered == Some(index);
                            let selected = st.selected == Some(index);

                            let background = if hovered || selected {
                                bg_hover
                            } else if (index % 2) == 0 {
                                bg_even
                            } else {
                                bg_odd
                            };

                            painter.scene().push(fret_core::SceneOp::Quad {
                                order: DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(background),
                                border: if selected {
                                    Edges::all(Px(1.0))
                                } else {
                                    Edges::all(Px(0.0))
                                },
                                border_paint: fret_core::Paint::Solid(if selected {
                                    fg
                                } else {
                                    fret_core::Color::TRANSPARENT
                                }),
                                corner_radii: Corners::all(Px(0.0)),
                            });

                            let label = format!("Row {index}");
                            let origin = fret_core::Point::new(
                                Px(rect.origin.x.0 + 8.0),
                                Px(rect.origin.y.0 + 4.0),
                            );
                            let scope = painter.key_scope(&"ui-gallery-windowed-rows-interactive");
                            let key: u64 = painter.child_key(scope, &index).into();
                            let _ = painter.text(
                                key,
                                DrawOrder(1),
                                origin,
                                label,
                                text_style.clone(),
                                fg,
                                CanvasTextConstraints {
                                    max_width: Some(Px(rect.size.width.0.max(0.0) - 16.0)),
                                    wrap: fret_core::TextWrap::None,
                                    overflow: fret_core::TextOverflow::Clip,
                                },
                                painter.scale_factor(),
                            );
                        },
                    )]
                },
            );

            vec![root]
        });

    vec![header, surface]
}

pub(in crate::ui) fn preview_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

    let outline_fg = ColorRef::Color(theme.color_required("foreground"));
    let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));
    let muted_fg = ColorRef::Color(theme.color_required("muted-foreground"));

    let icon = |cx: &mut ElementContext<'_, App>, name: &'static str, fg: ColorRef| {
        shadcn::icon::icon_with(cx, fret_icons::IconId::new_static(name), None, Some(fg))
    };

    let content_text = |cx: &mut ElementContext<'_, App>, text: &'static str, fg: ColorRef| {
        ui::text(cx, text)
            .font_medium()
            .nowrap()
            .text_color(fg)
            .into_element(cx)
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let size = {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N3).items_start(),
            |cx| {
                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Button::new("Small")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx),
                                shadcn::Button::new("Submit")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::IconSm)
                                    .children([icon(
                                        cx,
                                        "lucide.arrow-up-right",
                                        outline_fg.clone(),
                                    )])
                                    .into_element(cx),
                            ]
                        },
                    ),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Button::new("Default")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .into_element(cx),
                                shadcn::Button::new("Submit")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Icon)
                                    .children([icon(
                                        cx,
                                        "lucide.arrow-up-right",
                                        outline_fg.clone(),
                                    )])
                                    .into_element(cx),
                            ]
                        },
                    ),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Button::new("Large")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Lg)
                                    .into_element(cx),
                                shadcn::Button::new("Submit")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::IconLg)
                                    .children([icon(
                                        cx,
                                        "lucide.arrow-up-right",
                                        outline_fg.clone(),
                                    )])
                                    .into_element(cx),
                            ]
                        },
                    ),
                ]
            },
        );
        section(cx, "Size", body)
    };

    let default_body = shadcn::Button::new("Button").into_element(cx);
    let default = section(cx, "Default", default_body);

    let outline_body = shadcn::Button::new("Outline")
        .variant(shadcn::ButtonVariant::Outline)
        .into_element(cx);
    let outline = section(cx, "Outline", outline_body);

    let secondary_body = shadcn::Button::new("Secondary")
        .variant(shadcn::ButtonVariant::Secondary)
        .into_element(cx);
    let secondary = section(cx, "Secondary", secondary_body);

    let ghost_body = shadcn::Button::new("Ghost")
        .variant(shadcn::ButtonVariant::Ghost)
        .into_element(cx);
    let ghost = section(cx, "Ghost", ghost_body);

    let destructive_body = shadcn::Button::new("Destructive")
        .variant(shadcn::ButtonVariant::Destructive)
        .into_element(cx);
    let destructive = section(cx, "Destructive", destructive_body);

    let link_body = shadcn::Button::new("Link")
        .variant(shadcn::ButtonVariant::Link)
        .into_element(cx);
    let link = section(cx, "Link", link_body);

    let icon_only_body = shadcn::Button::new("Submit")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .children([icon(cx, "lucide.arrow-up-right", outline_fg.clone())])
        .into_element(cx);
    let icon_only = section(cx, "Icon", icon_only_body);

    let with_icon = {
        let body = shadcn::Button::new("New Branch")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .children([
                icon(cx, "lucide.git-branch", outline_fg.clone())
                    .test_id("ui-gallery-button-with-icon-icon"),
                content_text(cx, "New Branch", outline_fg.clone())
                    .test_id("ui-gallery-button-with-icon-label"),
            ])
            .into_element(cx)
            .test_id("ui-gallery-button-with-icon");
        section(cx, "With Icon", body)
    };

    let rounded_body = shadcn::Button::new("Scroll to top")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .children([icon(cx, "lucide.arrow-up", outline_fg.clone())])
        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
        .into_element(cx);
    let rounded = section(cx, "Rounded", rounded_body);

    let spinner = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Button::new("Generating")
                        .variant(shadcn::ButtonVariant::Outline)
                        .disabled(true)
                        .children([
                            shadcn::Spinner::new()
                                .color(outline_fg.clone())
                                .into_element(cx),
                            content_text(cx, "Generating", outline_fg.clone()),
                        ])
                        .into_element(cx),
                    shadcn::Button::new("Downloading")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .disabled(true)
                        .children([
                            content_text(cx, "Downloading", secondary_fg.clone()),
                            shadcn::Spinner::new()
                                .color(secondary_fg.clone())
                                .into_element(cx),
                        ])
                        .into_element(cx),
                ]
            },
        );
        section(cx, "Spinner", body)
    };

    let button_group = {
        let demo = preview_button_group(cx)
            .into_iter()
            .next()
            .unwrap_or_else(|| cx.text("ButtonGroup demo is missing"));
        section(cx, "Button Group", demo)
    };

    let render_link = {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            |cx| {
                vec![
                    shadcn::Button::new("Documentation")
                        .variant(shadcn::ButtonVariant::Outline)
                        .on_click(CMD_APP_OPEN)
                        .into_element(cx),
                    ui::text(cx, "TODO: `Button::render` / `asChild` composition is not implemented yet in fret-ui-shadcn. For now, use `variant=Link` or a dedicated link component.")
                        .text_color(muted_fg.clone())
                        .into_element(cx),
                ]
            },
        );
        section(cx, "Link (render)", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Button::new("التالي")
                                .variant(shadcn::ButtonVariant::Outline)
                                .into_element(cx),
                            shadcn::Button::new("السابق")
                                .variant(shadcn::ButtonVariant::Outline)
                                .into_element(cx),
                        ]
                    },
                )
            },
        );
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N4).items_start(),
        |_cx| {
            vec![
                size,
                default,
                outline,
                secondary,
                ghost,
                destructive,
                link,
                icon_only,
                with_icon,
                rounded,
                spinner,
                button_group,
                render_link,
                rtl,
            ]
        },
    )]
}

pub(in crate::ui) fn preview_alert(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_alert(cx)
}

pub(in crate::ui) fn preview_shadcn_extras(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_shadcn_extras(cx)
}

pub(in crate::ui) fn preview_checkbox(
    cx: &mut ElementContext<'_, App>,
    model: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_checkbox(cx, model)
}

pub(in crate::ui) fn preview_switch(
    cx: &mut ElementContext<'_, App>,
    model: Model<bool>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct SwitchModels {
        description: Option<Model<bool>>,
        choice_share: Option<Model<bool>>,
        choice_notifications: Option<Model<bool>>,
        invalid: Option<Model<bool>>,
        size_small: Option<Model<bool>>,
        size_default: Option<Model<bool>>,
        rtl: Option<Model<bool>>,
    }

    let (description, choice_share, choice_notifications, invalid, size_small, size_default, rtl) =
        cx.with_state(SwitchModels::default, |st| {
            (
                st.description.clone(),
                st.choice_share.clone(),
                st.choice_notifications.clone(),
                st.invalid.clone(),
                st.size_small.clone(),
                st.size_default.clone(),
                st.rtl.clone(),
            )
        });

    let (description, choice_share, choice_notifications, invalid, size_small, size_default, rtl) =
        match (
            description,
            choice_share,
            choice_notifications,
            invalid,
            size_small,
            size_default,
            rtl,
        ) {
            (
                Some(description),
                Some(choice_share),
                Some(choice_notifications),
                Some(invalid),
                Some(size_small),
                Some(size_default),
                Some(rtl),
            ) => (
                description,
                choice_share,
                choice_notifications,
                invalid,
                size_small,
                size_default,
                rtl,
            ),
            _ => {
                let description = cx.app.models_mut().insert(false);
                let choice_share = cx.app.models_mut().insert(false);
                let choice_notifications = cx.app.models_mut().insert(true);
                let invalid = cx.app.models_mut().insert(false);
                let size_small = cx.app.models_mut().insert(false);
                let size_default = cx.app.models_mut().insert(true);
                let rtl = cx.app.models_mut().insert(false);
                cx.with_state(SwitchModels::default, |st| {
                    st.description = Some(description.clone());
                    st.choice_share = Some(choice_share.clone());
                    st.choice_notifications = Some(choice_notifications.clone());
                    st.invalid = Some(invalid.clone());
                    st.size_small = Some(size_small.clone());
                    st.size_default = Some(size_default.clone());
                    st.rtl = Some(rtl.clone());
                });
                (
                    description,
                    choice_share,
                    choice_notifications,
                    invalid,
                    size_small,
                    size_default,
                    rtl,
                )
            }
        };

    let destructive = cx.with_theme(|theme| theme.color_required("destructive"));

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let demo = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Switch::new(model.clone())
                        .a11y_label("Airplane mode")
                        .test_id("ui-gallery-switch-demo-toggle")
                        .into_element(cx),
                    shadcn::Label::new("Airplane Mode").into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-switch-demo");
        let body = centered(cx, row);
        section(cx, "Demo", body)
    };

    let description_section = {
        let field = shadcn::Field::new([
            shadcn::FieldContent::new([
                shadcn::FieldLabel::new("Share across devices").into_element(cx),
                shadcn::FieldDescription::new(
                    "Focus is shared across devices, and turns off when you leave the app.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(description)
                .a11y_label("Share across devices")
                .test_id("ui-gallery-switch-description-toggle")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
        .test_id("ui-gallery-switch-description");

        let body = centered(cx, field);
        section(cx, "Description", body)
    };

    let choice_card = {
        let share = shadcn::Field::new([
            shadcn::FieldContent::new([
                shadcn::FieldTitle::new("Share across devices").into_element(cx),
                shadcn::FieldDescription::new(
                    "Focus is shared across devices, and turns off when you leave the app.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(choice_share)
                .a11y_label("Share across devices")
                .test_id("ui-gallery-switch-choice-card-share")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_style(
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Lg)
                .p(Space::N4),
        )
        .into_element(cx);

        let notifications = shadcn::Field::new([
            shadcn::FieldContent::new([
                shadcn::FieldTitle::new("Enable notifications").into_element(cx),
                shadcn::FieldDescription::new(
                    "Receive notifications when focus mode is enabled or disabled.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(choice_notifications)
                .a11y_label("Enable notifications")
                .test_id("ui-gallery-switch-choice-card-notifications")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_style(
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Lg)
                .p(Space::N4),
        )
        .into_element(cx);

        let group = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |_cx| vec![share, notifications],
        )
        .test_id("ui-gallery-switch-choice-card");

        let body = centered(cx, group);
        section(cx, "Choice Card", body)
    };

    let disabled_section = {
        let row = shadcn::Field::new([
            shadcn::Switch::new(model.clone())
                .disabled(true)
                .a11y_label("Disabled switch")
                .test_id("ui-gallery-switch-disabled-toggle")
                .into_element(cx),
            shadcn::FieldLabel::new("Disabled").into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
        .into_element(cx)
        .test_id("ui-gallery-switch-disabled");

        let body = centered(cx, row);
        section(cx, "Disabled", body)
    };

    let invalid_section = {
        let invalid_style = shadcn::switch::SwitchStyle::default().border_color(
            fret_ui_kit::WidgetStateProperty::new(Some(ColorRef::Color(destructive))),
        );

        let field = shadcn::Field::new([
            shadcn::FieldContent::new([
                ui::label(cx, "Accept terms and conditions")
                    .text_color(ColorRef::Color(destructive))
                    .into_element(cx),
                shadcn::FieldDescription::new(
                    "You must accept the terms and conditions to continue.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(invalid)
                .a11y_label("Accept terms and conditions")
                .style(invalid_style)
                .test_id("ui-gallery-switch-invalid-toggle")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
        .test_id("ui-gallery-switch-invalid");

        let body = centered(cx, field);
        section(cx, "Invalid", body)
    };

    let size_section = {
        let small = shadcn::Field::new([
            shadcn::Switch::new(size_small)
                .a11y_label("Small switch")
                .refine_layout(LayoutRefinement::default().w_px(Px(28.0)).h_px(Px(16.0)))
                .test_id("ui-gallery-switch-size-small")
                .into_element(cx),
            shadcn::FieldLabel::new("Small").into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx);

        let default = shadcn::Field::new([
            shadcn::Switch::new(size_default)
                .a11y_label("Default switch")
                .test_id("ui-gallery-switch-size-default")
                .into_element(cx),
            shadcn::FieldLabel::new("Default").into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx);

        let group = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .layout(LayoutRefinement::default().w_full().max_w(Px(160.0))),
            |_cx| vec![small, default],
        )
        .test_id("ui-gallery-switch-size");

        let body = centered(cx, group);
        section(cx, "Size", body)
    };

    let rtl_section = {
        let rtl_field = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Field::new([
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Share across devices").into_element(cx),
                        shadcn::FieldDescription::new(
                            "Focus is shared across devices, and turns off when you leave the app.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::Switch::new(rtl)
                        .a11y_label("Share across devices")
                        .test_id("ui-gallery-switch-rtl-toggle")
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-switch-rtl");

        let body = centered(cx, rtl_field);
        section(cx, "RTL", body)
    };

    let examples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                description_section,
                choice_card,
                disabled_section,
                invalid_section,
                size_section,
                rtl_section,
            ]
        },
    );

    let note = shadcn::typography::muted(
        cx,
        "Note: size/invalid are approximated with layout/style overrides because this Switch API has no dedicated size/aria-invalid props."
            .to_string(),
    );

    vec![demo, examples, note]
}

pub(in crate::ui) fn preview_input(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
) -> Vec<AnyElement> {
    pages::preview_input(cx, value)
}

pub(in crate::ui) fn preview_textarea(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct TextareaModels {
        field: Option<Model<String>>,
        disabled: Option<Model<String>>,
        invalid: Option<Model<String>>,
        button: Option<Model<String>>,
        rtl: Option<Model<String>>,
    }

    let state = cx.with_state(TextareaModels::default, |st| st.clone());
    let (field_value, disabled_value, invalid_value, button_value, rtl_value) = match (
        state.field,
        state.disabled,
        state.invalid,
        state.button,
        state.rtl,
    ) {
        (
            Some(field_value),
            Some(disabled_value),
            Some(invalid_value),
            Some(button_value),
            Some(rtl_value),
        ) => (
            field_value,
            disabled_value,
            invalid_value,
            button_value,
            rtl_value,
        ),
        _ => {
            let field_value = cx.app.models_mut().insert(String::new());
            let disabled_value = cx.app.models_mut().insert(String::new());
            let invalid_value = cx.app.models_mut().insert(String::new());
            let button_value = cx.app.models_mut().insert(String::new());
            let rtl_value = cx.app.models_mut().insert(String::new());
            cx.with_state(TextareaModels::default, |st| {
                st.field = Some(field_value.clone());
                st.disabled = Some(disabled_value.clone());
                st.invalid = Some(invalid_value.clone());
                st.button = Some(button_value.clone());
                st.rtl = Some(rtl_value.clone());
            });
            (
                field_value,
                disabled_value,
                invalid_value,
                button_value,
                rtl_value,
            )
        }
    };

    let destructive = cx.with_theme(|theme| theme.color_required("destructive"));

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(420.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let area_layout = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let demo = {
        let area = shadcn::Textarea::new(value)
            .a11y_label("Message")
            .min_height(Px(96.0))
            .refine_layout(area_layout.clone())
            .into_element(cx)
            .test_id("ui-gallery-textarea-demo");

        let body = centered(cx, area);
        section(cx, "Demo", body)
    };

    let field = {
        let field = shadcn::Field::new([
            shadcn::FieldLabel::new("Message").into_element(cx),
            shadcn::FieldDescription::new("Enter your message below.").into_element(cx),
            shadcn::Textarea::new(field_value)
                .a11y_label("Message field")
                .min_height(Px(96.0))
                .refine_layout(area_layout.clone())
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-textarea-field");

        let body = centered(cx, field);
        section(cx, "Field", body)
    };

    let disabled = {
        let field = shadcn::Field::new([
            shadcn::FieldLabel::new("Message").into_element(cx),
            shadcn::Textarea::new(disabled_value)
                .a11y_label("Disabled message")
                .disabled(true)
                .min_height(Px(96.0))
                .refine_layout(area_layout.clone())
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-textarea-disabled");

        let body = centered(cx, field);
        section(cx, "Disabled", body)
    };

    let invalid = {
        let field = shadcn::Field::new([
            ui::label(cx, "Message")
                .text_color(ColorRef::Color(destructive))
                .into_element(cx),
            shadcn::Textarea::new(invalid_value)
                .a11y_label("Invalid message")
                .aria_invalid(true)
                .min_height(Px(96.0))
                .refine_layout(area_layout.clone())
                .into_element(cx),
            shadcn::FieldDescription::new("Please enter a valid message.").into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-textarea-invalid");

        let body = centered(cx, field);
        section(cx, "Invalid", body)
    };

    let button = {
        let group = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
            |cx| {
                vec![
                    shadcn::Textarea::new(button_value)
                        .a11y_label("Send message")
                        .min_height(Px(96.0))
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    shadcn::Button::new("Send message").into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-textarea-button");

        let body = centered(cx, group);
        section(cx, "Button", body)
    };

    let rtl = {
        let rtl_field = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Field::new([
                    shadcn::FieldLabel::new("Feedback").into_element(cx),
                    shadcn::Textarea::new(rtl_value)
                        .a11y_label("Feedback")
                        .min_height(Px(96.0))
                        .refine_layout(area_layout.clone())
                        .into_element(cx),
                    shadcn::FieldDescription::new("Share your thoughts about our service.")
                        .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-textarea-rtl");

        let rtl_shell = shell(cx, rtl_field);
        let body = centered(cx, rtl_shell);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Displays a form textarea or a component that looks like a textarea."),
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N6)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![demo, field, disabled, invalid, button, rtl],
        ),
    ]
}

pub(in crate::ui) fn preview_label(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_label(cx)
}

pub(in crate::ui) fn preview_kbd(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_kbd(cx)
}
pub(in crate::ui) fn preview_separator(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                layout,
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let demo = {
        let top = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N1).items_start(),
            |cx| {
                vec![
                    shadcn::typography::small(cx, "Radix Primitives"),
                    shadcn::typography::muted(cx, "An open-source UI component library."),
                ]
            },
        );

        let links = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_center()
                .layout(LayoutRefinement::default().w_full().h_px(Px(20.0))),
            |cx| {
                vec![
                    cx.text("Blog"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Docs"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Source"),
                ]
            },
        );

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |cx| {
                vec![
                    top,
                    shadcn::Separator::new()
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                    links,
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-demo"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "Demo", body)
    };

    let vertical = {
        let content = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_center()
                .layout(LayoutRefinement::default().h_px(Px(20.0))),
            |cx| {
                vec![
                    cx.text("Blog"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Docs"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Source"),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-vertical"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "Vertical", body)
    };

    let menu = {
        let menu_item =
            |cx: &mut ElementContext<'_, App>, title: &'static str, desc: &'static str| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default().gap(Space::N1).items_start(),
                    move |cx| {
                        vec![
                            shadcn::typography::small(cx, title),
                            shadcn::typography::muted(cx, desc),
                        ]
                    },
                )
            };

        let content = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N3)
                .items_center()
                .layout(LayoutRefinement::default().w_full().max_w(Px(560.0))),
            |cx| {
                vec![
                    menu_item(cx, "Settings", "Manage preferences"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    menu_item(cx, "Account", "Profile & security"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    menu_item(cx, "Help", "Support & docs"),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-menu"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "Menu", body)
    };

    let list = {
        let row = |cx: &mut ElementContext<'_, App>, key: &'static str, value: &'static str| {
            stack::hstack(
                cx,
                stack::HStackProps::default()
                    .justify_between()
                    .items_center()
                    .layout(LayoutRefinement::default().w_full()),
                move |cx| vec![cx.text(key), shadcn::typography::muted(cx, value)],
            )
        };

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |cx| {
                vec![
                    row(cx, "Item 1", "Value 1"),
                    shadcn::Separator::new().into_element(cx),
                    row(cx, "Item 2", "Value 2"),
                    shadcn::Separator::new().into_element(cx),
                    row(cx, "Item 3", "Value 3"),
                ]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-list"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "List", body)
    };

    let rtl = {
        let content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N4)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
                    |cx| {
                        vec![
                            stack::vstack(
                                cx,
                                stack::VStackProps::default().gap(Space::N1).items_start(),
                                |cx| {
                                    vec![
                                        shadcn::typography::small(cx, "shadcn/ui"),
                                        shadcn::typography::muted(cx, "أساس نظام التصميم الخاص بك"),
                                    ]
                                },
                            ),
                            shadcn::Separator::new().into_element(cx),
                            shadcn::typography::muted(
                                cx,
                                "مجموعة مكونات مصممة بشكل جميل يمكنك تخصيصها وتوسيعها.",
                            ),
                        ]
                    },
                )
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-separator-rtl"),
        );

        let card = shell(cx, LayoutRefinement::default(), content);
        let body = centered(cx, card);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Visually or semantically separates content."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, vertical, menu, list, rtl]
        }),
    ]
}

pub(in crate::ui) fn preview_spinner(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct SpinnerModels {
        input_value: Option<Model<String>>,
        textarea_value: Option<Model<String>>,
    }

    let (input_value, textarea_value) = cx.with_state(SpinnerModels::default, |st| {
        (st.input_value.clone(), st.textarea_value.clone())
    });
    let (input_value, textarea_value) = match (input_value, textarea_value) {
        (Some(input_value), Some(textarea_value)) => (input_value, textarea_value),
        _ => {
            let input_value = cx.app.models_mut().insert(String::new());
            let textarea_value = cx.app.models_mut().insert(String::new());
            cx.with_state(SpinnerModels::default, |st| {
                st.input_value = Some(input_value.clone());
                st.textarea_value = Some(textarea_value.clone());
            });
            (input_value, textarea_value)
        }
    };

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                layout,
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let demo = {
        let item = shadcn::Item::new([
            shadcn::ItemMedia::new([shadcn::Spinner::new().into_element(cx)]).into_element(cx),
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new("Processing payment...").into_element(cx)
            ])
            .into_element(cx),
            shadcn::ItemActions::new([cx.text("$100.00")]).into_element(cx),
        ])
        .variant(shadcn::ItemVariant::Muted)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-demo");
        let body = centered(cx, item);
        section(cx, "Demo", body)
    };

    let custom = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| {
                vec![
                    shadcn::Spinner::new().into_element(cx),
                    shadcn::Spinner::new()
                        .icon(fret_icons::ids::ui::SETTINGS)
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-custom");
        let body = centered(cx, row);
        section(cx, "Customization", body)
    };

    let size = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N6).items_center(),
            |cx| {
                vec![
                    shadcn::Spinner::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(12.0)).h_px(Px(12.0)))
                        .into_element(cx),
                    shadcn::Spinner::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(16.0)).h_px(Px(16.0)))
                        .into_element(cx),
                    shadcn::Spinner::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(24.0)).h_px(Px(24.0)))
                        .into_element(cx),
                    shadcn::Spinner::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-size");
        let body = centered(cx, row);
        section(cx, "Size", body)
    };

    let button = {
        let group = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N3).items_center(),
            |cx| {
                vec![
                    shadcn::Button::new("Loading...")
                        .size(shadcn::ButtonSize::Sm)
                        .disabled(true)
                        .children([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                    shadcn::Button::new("Please wait")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .disabled(true)
                        .children([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                    shadcn::Button::new("Processing")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .disabled(true)
                        .children([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-button");
        let body = centered(cx, group);
        section(cx, "Button", body)
    };

    let badge = {
        let (secondary_fg, outline_fg) = cx.with_theme(|theme| {
            (
                ColorRef::Color(theme.color_required("secondary-foreground")),
                ColorRef::Color(theme.color_required("foreground")),
            )
        });

        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| {
                vec![
                    shadcn::Badge::new("Syncing")
                        .children([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                    shadcn::Badge::new("Updating")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .children([shadcn::Spinner::new()
                            .color(secondary_fg.clone())
                            .into_element(cx)])
                        .into_element(cx),
                    shadcn::Badge::new("Processing")
                        .variant(shadcn::BadgeVariant::Outline)
                        .children([shadcn::Spinner::new()
                            .color(outline_fg.clone())
                            .into_element(cx)])
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-badge");
        let body = centered(cx, row);
        section(cx, "Badge", body)
    };

    let input_group = {
        let input = shadcn::InputGroup::new(input_value)
            .a11y_label("Send a message")
            .trailing([shadcn::Spinner::new().into_element(cx)])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        let textarea = shadcn::InputGroup::new(textarea_value)
            .textarea()
            .a11y_label("Send a message textarea")
            .block_end([stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N2)
                    .items_center(),
                |cx| {
                    vec![
                        shadcn::Spinner::new().into_element(cx),
                        shadcn::typography::muted(cx, "Validating..."),
                        shadcn::InputGroupButton::new("")
                            .size(shadcn::InputGroupButtonSize::IconSm)
                            .children([shadcn::icon::icon(
                                cx,
                                fret_icons::IconId::new_static("lucide.arrow-up"),
                            )])
                            .into_element(cx),
                    ]
                },
            )])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        let group = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![input, textarea],
        );

        let card = shell(
            cx,
            LayoutRefinement::default().w_full().max_w(Px(480.0)),
            group,
        )
        .test_id("ui-gallery-spinner-input-group");

        let body = centered(cx, card);
        section(cx, "Input Group", body)
    };

    let empty = {
        let card = shadcn::Empty::new([
            shadcn::empty::EmptyHeader::new([
                shadcn::empty::EmptyMedia::new([shadcn::Spinner::new().into_element(cx)])
                    .variant(shadcn::empty::EmptyMediaVariant::Icon)
                    .into_element(cx),
                shadcn::empty::EmptyTitle::new("Processing your request").into_element(cx),
                shadcn::empty::EmptyDescription::new(
                    "Please wait while we process your request. Do not refresh the page.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::empty::EmptyContent::new([shadcn::Button::new("Cancel")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into_element(cx)])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-empty");

        let body = centered(cx, card);
        section(cx, "Empty", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Item::new([
                    shadcn::ItemMedia::new([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                    shadcn::ItemContent::new([
                        shadcn::ItemTitle::new("Processing payment...").into_element(cx)
                    ])
                    .into_element(cx),
                    shadcn::ItemActions::new([cx.text("$100.00")]).into_element(cx),
                ])
                .variant(shadcn::ItemVariant::Muted)
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-spinner-rtl");

        let centered_body = centered(cx, body);
        section(cx, "RTL", centered_body)
    };

    vec![
        cx.text("An indicator that can be used to show a loading state."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, custom, size, button, badge, input_group, empty, rtl]
        }),
    ]
}

pub(in crate::ui) fn preview_aspect_ratio(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_aspect_ratio(cx)
}

pub(in crate::ui) fn preview_breadcrumb(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    pages::preview_breadcrumb(cx, last_action)
}

pub(in crate::ui) fn preview_button_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ButtonGroupModels {
        search_value: Option<Model<String>>,
        message_value: Option<Model<String>>,
        amount_value: Option<Model<String>>,
        dropdown_open: Option<Model<bool>>,
        select_open: Option<Model<bool>>,
        select_value: Option<Model<Option<Arc<str>>>>,
        popover_open: Option<Model<bool>>,
        popover_text: Option<Model<String>>,
    }

    let search_value = cx.with_state(ButtonGroupModels::default, |st| st.search_value.clone());
    let search_value = match search_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(ButtonGroupModels::default, |st| {
                st.search_value = Some(model.clone())
            });
            model
        }
    };

    let message_value = cx.with_state(ButtonGroupModels::default, |st| st.message_value.clone());
    let message_value = match message_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(ButtonGroupModels::default, |st| {
                st.message_value = Some(model.clone())
            });
            model
        }
    };

    let amount_value = cx.with_state(ButtonGroupModels::default, |st| st.amount_value.clone());
    let amount_value = match amount_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(ButtonGroupModels::default, |st| {
                st.amount_value = Some(model.clone())
            });
            model
        }
    };

    let dropdown_open = cx.with_state(ButtonGroupModels::default, |st| st.dropdown_open.clone());
    let dropdown_open = match dropdown_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ButtonGroupModels::default, |st| {
                st.dropdown_open = Some(model.clone())
            });
            model
        }
    };

    let select_open = cx.with_state(ButtonGroupModels::default, |st| st.select_open.clone());
    let select_open = match select_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ButtonGroupModels::default, |st| {
                st.select_open = Some(model.clone())
            });
            model
        }
    };

    let select_value = cx.with_state(ButtonGroupModels::default, |st| st.select_value.clone());
    let select_value = match select_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("$")));
            cx.with_state(ButtonGroupModels::default, |st| {
                st.select_value = Some(model.clone())
            });
            model
        }
    };

    let popover_open = cx.with_state(ButtonGroupModels::default, |st| st.popover_open.clone());
    let popover_open = match popover_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ButtonGroupModels::default, |st| {
                st.popover_open = Some(model.clone())
            });
            model
        }
    };

    let popover_text = cx.with_state(ButtonGroupModels::default, |st| st.popover_text.clone());
    let popover_text = match popover_text {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(String::from("Describe your task in natural language."));
            cx.with_state(ButtonGroupModels::default, |st| {
                st.popover_text = Some(model.clone())
            });
            model
        }
    };

    let theme = Theme::global(&*cx.app).snapshot();
    let outline_fg = ColorRef::Color(theme.color_required("foreground"));
    let secondary_fg = ColorRef::Color(theme.color_required("secondary-foreground"));

    let icon = |cx: &mut ElementContext<'_, App>, name: &'static str, fg: ColorRef| {
        shadcn::icon::icon_with(cx, fret_icons::IconId::new_static(name), None, Some(fg))
    };

    // Mirrors the top-level `button-group-demo` preview slot.
    let demo = shadcn::ButtonGroup::new([
        shadcn::Button::new("Left").into(),
        shadcn::Button::new("Middle").into(),
        shadcn::Button::new("Right").into(),
    ])
    .a11y_label("Button group")
    .into_element(cx);

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let orientation = {
        let body = shadcn::ButtonGroup::new([
            shadcn::Button::new("Increase")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.plus", outline_fg.clone())])
                .into(),
            shadcn::Button::new("Decrease")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.minus", outline_fg.clone())])
                .into(),
        ])
        .orientation(shadcn::ButtonGroupOrientation::Vertical)
        .a11y_label("Media controls")
        .into_element(cx);
        section(cx, "Orientation", body)
    };

    let size = {
        let small = shadcn::ButtonGroup::new([
            shadcn::Button::new("Small")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("Button")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("Group")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("Add")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconSm)
                .children([icon(cx, "lucide.plus", outline_fg.clone())])
                .into(),
        ])
        .into_element(cx);

        let medium = shadcn::ButtonGroup::new([
            shadcn::Button::new("Default")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
            shadcn::Button::new("Button")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
            shadcn::Button::new("Group")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
            shadcn::Button::new("Add")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.plus", outline_fg.clone())])
                .into(),
        ])
        .into_element(cx);

        let large = shadcn::ButtonGroup::new([
            shadcn::Button::new("Large")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Lg)
                .into(),
            shadcn::Button::new("Button")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Lg)
                .into(),
            shadcn::Button::new("Group")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Lg)
                .into(),
            shadcn::Button::new("Add")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconLg)
                .children([icon(cx, "lucide.plus", outline_fg.clone())])
                .into(),
        ])
        .into_element(cx);

        let body = stack::vstack(cx, stack::VStackProps::default().gap(Space::N4), |_cx| {
            vec![small, medium, large]
        });
        section(cx, "Size", body)
    };

    let nested = {
        let digits = shadcn::ButtonGroup::new([
            shadcn::Button::new("1")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("2")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("3")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("4")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Button::new("5")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into(),
        ]);

        let nav = shadcn::ButtonGroup::new([
            shadcn::Button::new("Previous")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconSm)
                .children([icon(cx, "lucide.arrow-left", outline_fg.clone())])
                .into(),
            shadcn::Button::new("Next")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconSm)
                .children([icon(cx, "lucide.arrow-right", outline_fg.clone())])
                .into(),
        ]);

        let body = shadcn::ButtonGroup::new([digits.into(), nav.into()]).into_element(cx);
        section(cx, "Nested", body)
    };

    let separator = {
        let body = shadcn::ButtonGroup::new([
            shadcn::Button::new("Copy")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .into(),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .into(),
            shadcn::Button::new("Paste")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .into(),
        ])
        .into_element(cx);
        section(cx, "Separator", body)
    };

    let split = {
        let body = shadcn::ButtonGroup::new([
            shadcn::Button::new("Button")
                .variant(shadcn::ButtonVariant::Secondary)
                .into(),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .into(),
            shadcn::Button::new("Add")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.plus", secondary_fg.clone())])
                .into(),
        ])
        .into_element(cx);
        section(cx, "Split", body)
    };

    let input = {
        let body = shadcn::ButtonGroup::new([
            shadcn::Input::new(search_value.clone())
                .a11y_label("Search")
                .placeholder("Search...")
                .refine_layout(LayoutRefinement::default().w_px(Px(220.0)))
                .into_element(cx)
                .into(),
            shadcn::Button::new("Search")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.search", outline_fg.clone())])
                .into(),
        ])
        .into_element(cx);
        section(cx, "Input", body)
    };

    let input_group = {
        let group = shadcn::InputGroup::new(message_value.clone())
            .a11y_label("Message")
            .leading([shadcn::InputGroupText::new("To").into_element(cx)])
            .trailing([shadcn::InputGroupButton::new("Send").into_element(cx)]);

        let body = shadcn::ButtonGroup::new([
            shadcn::Button::new("Add")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Icon)
                .children([icon(cx, "lucide.plus", outline_fg.clone())])
                .into(),
            group.into(),
        ])
        .into_element(cx);
        section(cx, "Input Group", body)
    };

    let dropdown = {
        let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone()).into_element(
            cx,
            |cx| {
                shadcn::Button::new("More")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Icon)
                    .children([icon(cx, "lucide.chevron-down", outline_fg.clone())])
                    .toggle_model(dropdown_open.clone())
                    .into_element(cx)
            },
            |cx| {
                vec![
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Mute Conversation").leading(icon(
                            cx,
                            "lucide.volume-x",
                            outline_fg.clone(),
                        )),
                    ),
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Mark as Read").leading(icon(
                            cx,
                            "lucide.check",
                            outline_fg.clone(),
                        )),
                    ),
                    shadcn::DropdownMenuEntry::Separator,
                    shadcn::DropdownMenuEntry::Item(
                        shadcn::DropdownMenuItem::new("Delete Conversation")
                            .variant(shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive)
                            .leading(icon(cx, "lucide.trash", outline_fg.clone())),
                    ),
                ]
            },
        );

        let body = shadcn::ButtonGroup::new([
            shadcn::Button::new("Follow")
                .variant(shadcn::ButtonVariant::Outline)
                .into(),
            dropdown.into(),
        ])
        .into_element(cx);
        section(cx, "Dropdown Menu", body)
    };

    let select = {
        let currency = shadcn::Select::new(select_value.clone(), select_open.clone())
            .placeholder("$")
            .refine_layout(LayoutRefinement::default().w_px(Px(96.0)))
            .items([
                shadcn::SelectItem::new("$", "US Dollar"),
                shadcn::SelectItem::new("€", "Euro"),
                shadcn::SelectItem::new("£", "British Pound"),
            ])
            .into_element(cx);

        let amount = shadcn::Input::new(amount_value.clone())
            .a11y_label("Amount")
            .placeholder("10.00")
            .refine_layout(LayoutRefinement::default().w_px(Px(140.0)))
            .into_element(cx);

        let send = shadcn::Button::new("Send")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Icon)
            .children([icon(cx, "lucide.arrow-right", outline_fg.clone())]);

        let body = shadcn::ButtonGroup::new([
            shadcn::ButtonGroup::new([currency.into(), amount.into()]).into(),
            shadcn::ButtonGroup::new([send.into()]).into(),
        ])
        .into_element(cx);
        section(cx, "Select", body)
    };

    let popover = {
        let popover = shadcn::Popover::new(popover_open.clone())
            .side(shadcn::PopoverSide::Bottom)
            .align(shadcn::PopoverAlign::End)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open Popover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Icon)
                        .children([icon(cx, "lucide.chevron-down", outline_fg.clone())])
                        .toggle_model(popover_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new(vec![
                        shadcn::PopoverTitle::new("Agent Tasks").into_element(cx),
                        shadcn::Separator::new().into_element(cx),
                        shadcn::Textarea::new(popover_text.clone())
                            .a11y_label("Task")
                            .refine_layout(LayoutRefinement::default().w_px(Px(260.0)))
                            .into_element(cx),
                    ])
                    .into_element(cx)
                },
            );

        let body = shadcn::ButtonGroup::new([
            shadcn::Button::new("Copilot")
                .variant(shadcn::ButtonVariant::Outline)
                .children([icon(cx, "lucide.bot", outline_fg.clone())])
                .into(),
            popover.into(),
        ])
        .into_element(cx);
        section(cx, "Popover", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::ButtonGroup::new([
                    shadcn::Button::new("التالي")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into(),
                    shadcn::Button::new("السابق")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into(),
                ])
                .into_element(cx)
            },
        );
        section(cx, "RTL", body)
    };

    let examples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                orientation,
                size,
                nested,
                separator,
                split,
                input,
                input_group,
                dropdown,
                select,
                popover,
                rtl,
            ]
        },
    );

    vec![demo, examples]
}

pub(in crate::ui) fn preview_calendar(
    cx: &mut ElementContext<'_, App>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    use fret_ui_headless::calendar::DateRangeSelection;

    let theme = Theme::global(&*cx.app).snapshot();
    let today = time::OffsetDateTime::now_utc().date();

    #[derive(Default, Clone)]
    struct CalendarModels {
        caption_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        caption_selected: Option<Model<Option<Date>>>,
        range_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        range_selected: Option<Model<DateRangeSelection>>,
        presets_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        presets_selected: Option<Model<Option<Date>>>,
        time_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        time_selected: Option<Model<Option<Date>>>,
        time_from: Option<Model<String>>,
        time_to: Option<Model<String>>,
        booked_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        booked_selected: Option<Model<Option<Date>>>,
        custom_cell_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        custom_cell_selected: Option<Model<Option<Date>>>,
        week_number_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        week_number_selected: Option<Model<Option<Date>>>,
        rtl_month: Option<Model<fret_ui_headless::calendar::CalendarMonth>>,
        rtl_selected: Option<Model<Option<Date>>>,
    }

    let initial_month = cx
        .get_model_copied(&month, Invalidation::Layout)
        .unwrap_or_else(|| fret_ui_headless::calendar::CalendarMonth::from_date(today));

    let state = cx.with_state(CalendarModels::default, |st| st.clone());

    let caption_month = match state.caption_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.caption_month = Some(model.clone())
            });
            model
        }
    };
    let caption_selected = match state.caption_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.caption_selected = Some(model.clone())
            });
            model
        }
    };

    let range_month = match state.range_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.range_month = Some(model.clone())
            });
            model
        }
    };
    let range_selected = match state.range_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(DateRangeSelection::default());
            cx.with_state(CalendarModels::default, |st| {
                st.range_selected = Some(model.clone())
            });
            model
        }
    };

    let preset_date = time::Date::from_calendar_date(today.year(), time::Month::February, 12)
        .expect("valid preset date");
    let presets_initial_month = fret_ui_headless::calendar::CalendarMonth::from_date(preset_date);
    let presets_month = match state.presets_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(presets_initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.presets_month = Some(model.clone())
            });
            model
        }
    };
    let presets_selected = match state.presets_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(preset_date));
            cx.with_state(CalendarModels::default, |st| {
                st.presets_selected = Some(model.clone())
            });
            model
        }
    };

    let time_date = time::Date::from_calendar_date(today.year(), today.month(), 12)
        .expect("valid time picker date");
    let time_initial_month = fret_ui_headless::calendar::CalendarMonth::from_date(time_date);
    let time_month = match state.time_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(time_initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.time_month = Some(model.clone())
            });
            model
        }
    };
    let time_selected = match state.time_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(time_date));
            cx.with_state(CalendarModels::default, |st| {
                st.time_selected = Some(model.clone())
            });
            model
        }
    };
    let time_from = match state.time_from {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("10:30:00"));
            cx.with_state(CalendarModels::default, |st| {
                st.time_from = Some(model.clone())
            });
            model
        }
    };
    let time_to = match state.time_to {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("12:30:00"));
            cx.with_state(CalendarModels::default, |st| {
                st.time_to = Some(model.clone())
            });
            model
        }
    };

    let booked_month = match state.booked_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.booked_month = Some(model.clone())
            });
            model
        }
    };
    let booked_selected = match state.booked_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.booked_selected = Some(model.clone())
            });
            model
        }
    };

    let custom_cell_month = match state.custom_cell_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.custom_cell_month = Some(model.clone())
            });
            model
        }
    };
    let custom_cell_selected = match state.custom_cell_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.custom_cell_selected = Some(model.clone())
            });
            model
        }
    };

    let week_number_month = match state.week_number_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.week_number_month = Some(model.clone())
            });
            model
        }
    };
    let week_number_selected = match state.week_number_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Date>);
            cx.with_state(CalendarModels::default, |st| {
                st.week_number_selected = Some(model.clone())
            });
            model
        }
    };

    let rtl_month = match state.rtl_month {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(initial_month);
            cx.with_state(CalendarModels::default, |st| {
                st.rtl_month = Some(model.clone())
            });
            model
        }
    };
    let rtl_selected = match state.rtl_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(today));
            cx.with_state(CalendarModels::default, |st| {
                st.rtl_selected = Some(model.clone())
            });
            model
        }
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let basic = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N6).items_start(),
            |cx| {
                let selected_str = cx
                    .get_model_copied(&selected, Invalidation::Layout)
                    .flatten()
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "<none>".to_string());

                vec![
                    shadcn::Calendar::new(month.clone(), selected.clone())
                        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                        .into_element(cx),
                    stack::vstack(
                        cx,
                        stack::VStackProps::default().gap(Space::N1).items_start(),
                        |cx| {
                            vec![cx.text_props(TextProps {
                                layout: Default::default(),
                                text: Arc::from(format!("selected={}", selected_str)),
                                style: None,
                                color: Some(theme.color_required("muted-foreground")),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                            })]
                        },
                    ),
                ]
            },
        );
        section(cx, "Basic", body)
    };

    let range = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N6).items_start(),
            |cx| {
                let range = cx
                    .get_model_copied(&range_selected, Invalidation::Layout)
                    .unwrap_or_default();
                let from = range
                    .from
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "<none>".to_string());
                let to = range
                    .to
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "<none>".to_string());

                vec![
                    shadcn::CalendarRange::new(range_month.clone(), range_selected.clone())
                        .number_of_months(2)
                        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                        .into_element(cx),
                    stack::vstack(
                        cx,
                        stack::VStackProps::default().gap(Space::N1).items_start(),
                        |cx| {
                            vec![
                                cx.text_props(TextProps {
                                    layout: Default::default(),
                                    text: Arc::from(format!("from={}", from)),
                                    style: None,
                                    color: Some(theme.color_required("muted-foreground")),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                                cx.text_props(TextProps {
                                    layout: Default::default(),
                                    text: Arc::from(format!("to={}", to)),
                                    style: None,
                                    color: Some(theme.color_required("muted-foreground")),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                            ]
                        },
                    ),
                ]
            },
        );
        section(cx, "Range Calendar", body)
    };

    let month_year_selector = {
        let body = shadcn::Calendar::new(caption_month.clone(), caption_selected.clone())
            .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
            .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
            .into_element(cx);
        section(cx, "Month and Year Selector", body)
    };

    let presets = {
        let preset_button =
            |cx: &mut ElementContext<'_, App>, label: &'static str, days: i64| -> AnyElement {
                let month = presets_month.clone();
                let selected = presets_selected.clone();
                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .refine_layout(LayoutRefinement::default().flex_1().w_full())
                    .on_activate(Arc::new(move |host, _acx, _reason| {
                        let new_date = today + time::Duration::days(days);
                        let _ = host.models_mut().update(&selected, |v| *v = Some(new_date));
                        let _ = host.models_mut().update(&month, |m| {
                            *m = fret_ui_headless::calendar::CalendarMonth::from_date(new_date);
                        });
                    }))
                    .into_element(cx)
            };

        let calendar = shadcn::Calendar::new(presets_month.clone(), presets_selected.clone())
            .cell_size(Px(38.0))
            .refine_style(ChromeRefinement::default().p(Space::N0))
            .into_element(cx);

        let footer = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full())
                .items_start(),
            |cx| {
                vec![
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .layout(LayoutRefinement::default().w_full()),
                        |cx| {
                            vec![
                                preset_button(cx, "Today", 0),
                                preset_button(cx, "Tomorrow", 1),
                                preset_button(cx, "In 3 days", 3),
                            ]
                        },
                    ),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .layout(LayoutRefinement::default().w_full()),
                        |cx| {
                            vec![
                                preset_button(cx, "In a week", 7),
                                preset_button(cx, "In 2 weeks", 14),
                            ]
                        },
                    ),
                ]
            },
        );

        let card = shadcn::Card::new(vec![
            shadcn::CardContent::new(vec![calendar]).into_element(cx),
            shadcn::CardFooter::new(vec![footer]).into_element(cx),
        ])
        .size(shadcn::CardSize::Sm)
        .refine_layout(
            LayoutRefinement::default()
                .max_w(MetricRef::Px(Px(300.0)))
                .min_w_0(),
        )
        .into_element(cx);

        section(cx, "Presets", card)
    };

    let date_and_time_picker = {
        let clock_fg = ColorRef::Color(theme.color_required("muted-foreground"));
        let clock_icon = |cx: &mut ElementContext<'_, App>| {
            shadcn::icon::icon_with(
                cx,
                fret_icons::IconId::new_static("lucide.clock-2"),
                None,
                Some(clock_fg.clone()),
            )
        };

        let calendar = shadcn::Calendar::new(time_month.clone(), time_selected.clone())
            .refine_style(ChromeRefinement::default().p(Space::N0))
            .into_element(cx);

        let footer = shadcn::FieldGroup::new([
            shadcn::Field::new([
                shadcn::FieldLabel::new("Start Time").into_element(cx),
                shadcn::InputGroup::new(time_from.clone())
                    .a11y_label("Start Time")
                    .trailing([clock_icon(cx)])
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("End Time").into_element(cx),
                shadcn::InputGroup::new(time_to.clone())
                    .a11y_label("End Time")
                    .trailing([clock_icon(cx)])
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let card = shadcn::Card::new(vec![
            shadcn::CardContent::new(vec![calendar]).into_element(cx),
            shadcn::CardFooter::new(vec![footer]).into_element(cx),
        ])
        .size(shadcn::CardSize::Sm)
        .refine_layout(LayoutRefinement::default().min_w_0())
        .into_element(cx);

        section(cx, "Date and Time Picker", card)
    };

    let booked_dates = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N6).items_start(),
            |cx| {
                vec![
                    shadcn::Calendar::new(booked_month.clone(), booked_selected.clone())
                        .disabled_by(|d| {
                            matches!(d.weekday(), time::Weekday::Saturday | time::Weekday::Sunday)
                        })
                        .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                        .into_element(cx),
                    cx.text_props(TextProps {
                        layout: Default::default(),
                        text: Arc::from("Disabled: weekends"),
                        style: None,
                        color: Some(theme.color_required("muted-foreground")),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    }),
                ]
            },
        );
        section(cx, "Booked dates", body)
    };

    let custom_cell_size = {
        let body = shadcn::Calendar::new(custom_cell_month.clone(), custom_cell_selected.clone())
            .cell_size(Px(44.0))
            .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
            .into_element(cx);
        section(cx, "Custom Cell Size", body)
    };

    let week_numbers = {
        let body = shadcn::Calendar::new(week_number_month.clone(), week_number_selected.clone())
            .show_week_number(true)
            .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
            .into_element(cx);
        section(cx, "Week Numbers", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Calendar::new(rtl_month.clone(), rtl_selected.clone())
                    .cell_size(Px(36.0))
                    .caption_layout(shadcn::CalendarCaptionLayout::Dropdown)
                    .refine_style(ChromeRefinement::default().border_1().rounded(Radius::Lg))
                    .into_element(cx)
            },
        );
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                basic,
                range,
                month_year_selector,
                presets,
                date_and_time_picker,
                booked_dates,
                custom_cell_size,
                week_numbers,
                rtl,
            ]
        },
    )]
}

pub(in crate::ui) fn preview_collapsible(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_collapsible(cx)
}

pub(in crate::ui) fn preview_drawer(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_drawer(cx)
}

pub(in crate::ui) fn preview_hover_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_hover_card(cx)
}

pub(in crate::ui) fn preview_input_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_input_group(cx)
}

pub(in crate::ui) fn preview_input_otp(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_input_otp(cx)
}

pub(in crate::ui) fn preview_menubar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_menubar(cx)
}
pub(in crate::ui) fn preview_navigation_menu(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_navigation_menu(cx)
}
pub(in crate::ui) fn preview_pagination(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct PaginationModels {
        rows_per_page: Option<Model<Option<Arc<str>>>>,
        rows_per_page_open: Option<Model<bool>>,
    }

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let state = cx.with_state(PaginationModels::default, |st| st.clone());
    let rows_per_page = match state.rows_per_page {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("25")));
            cx.with_state(PaginationModels::default, |st| {
                st.rows_per_page = Some(model.clone())
            });
            model
        }
    };
    let rows_per_page_open = match state.rows_per_page_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(PaginationModels::default, |st| {
                st.rows_per_page_open = Some(model.clone())
            });
            model
        }
    };

    let demo = {
        let content = shadcn::PaginationContent::new([
            shadcn::PaginationItem::new(
                shadcn::PaginationPrevious::new()
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("1")])
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("2")])
                    .on_click(CMD_APP_SAVE)
                    .active(true)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("3")])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(shadcn::PaginationEllipsis::new().into_element(cx))
                .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationNext::new()
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let pagination = shadcn::Pagination::new([content]).into_element(cx);
        let body = centered(cx, pagination);
        section(cx, "Demo", body)
    };

    let simple = {
        let content = shadcn::PaginationContent::new([
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("1")])
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("2")])
                    .on_click(CMD_APP_SAVE)
                    .active(true)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("3")])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("4")])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationLink::new([cx.text("5")])
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let pagination = shadcn::Pagination::new([content]).into_element(cx);
        let body = centered(cx, pagination);
        section(cx, "Simple", body)
    };

    let icons_only = {
        let rows_per_page = shadcn::Select::new(rows_per_page.clone(), rows_per_page_open.clone())
            .placeholder("25")
            .refine_layout(LayoutRefinement::default().w_px(Px(80.0)))
            .items([
                shadcn::SelectItem::new("10", "10"),
                shadcn::SelectItem::new("25", "25"),
                shadcn::SelectItem::new("50", "50"),
                shadcn::SelectItem::new("100", "100"),
            ])
            .into_element(cx);

        let rows_field = shadcn::Field::new([
            shadcn::FieldLabel::new("Rows per page").into_element(cx),
            rows_per_page,
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
        .into_element(cx);

        let content = shadcn::PaginationContent::new([
            shadcn::PaginationItem::new(
                shadcn::PaginationPrevious::new()
                    .on_click(CMD_APP_OPEN)
                    .into_element(cx),
            )
            .into_element(cx),
            shadcn::PaginationItem::new(
                shadcn::PaginationNext::new()
                    .on_click(CMD_APP_SAVE)
                    .into_element(cx),
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let pagination = shadcn::Pagination::new([content])
            .refine_layout(LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto))
            .into_element(cx);

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center()
                .justify_between()
                .gap(Space::N4),
            move |_cx| [rows_field, pagination],
        );

        section(cx, "Icons Only", row)
    };

    let rtl = {
        fn to_arabic_numerals(num: u32) -> String {
            const DIGITS: [&str; 10] = ["٠", "١", "٢", "٣", "٤", "٥", "٦", "٧", "٨", "٩"];
            num.to_string()
                .chars()
                .filter_map(|c| c.to_digit(10).map(|d| DIGITS[d as usize]))
                .collect()
        }

        let pagination = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let content = shadcn::PaginationContent::new([
                    shadcn::PaginationItem::new(
                        shadcn::PaginationPrevious::new()
                            .text("السابق")
                            .on_click(CMD_APP_OPEN)
                            .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::PaginationItem::new(
                        shadcn::PaginationLink::new([cx.text(to_arabic_numerals(1))])
                            .on_click(CMD_APP_OPEN)
                            .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::PaginationItem::new(
                        shadcn::PaginationLink::new([cx.text(to_arabic_numerals(2))])
                            .on_click(CMD_APP_SAVE)
                            .active(true)
                            .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::PaginationItem::new(
                        shadcn::PaginationLink::new([cx.text(to_arabic_numerals(3))])
                            .on_click(CMD_APP_SAVE)
                            .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::PaginationItem::new(shadcn::PaginationEllipsis::new().into_element(cx))
                        .into_element(cx),
                    shadcn::PaginationItem::new(
                        shadcn::PaginationNext::new()
                            .text("التالي")
                            .on_click(CMD_APP_SAVE)
                            .into_element(cx),
                    )
                    .into_element(cx),
                ])
                .into_element(cx);

                shadcn::Pagination::new([content]).into_element(cx)
            },
        );

        let body = centered(cx, pagination);
        section(cx, "RTL", body)
    };

    vec![stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N6).items_start(),
        |_cx| vec![demo, simple, icons_only, rtl],
    )]
}

pub(in crate::ui) fn preview_carousel(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_carousel(cx)
}

pub(in crate::ui) fn preview_chart(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_chart(cx)
}

pub(in crate::ui) fn preview_item(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_item(cx)
}
pub(in crate::ui) fn preview_native_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_native_select(cx)
}

pub(in crate::ui) fn preview_sidebar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct SidebarModels {
        demo_collapsed: Option<Model<bool>>,
        demo_selected: Option<Model<Arc<str>>>,
        controlled_collapsed: Option<Model<bool>>,
        controlled_selected: Option<Model<Arc<str>>>,
        rtl_selected: Option<Model<Arc<str>>>,
    }

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().border_1().rounded(Radius::Md),
                LayoutRefinement::default().w_full(),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let state = cx.with_state(SidebarModels::default, |st| st.clone());

    let demo_collapsed = match state.demo_collapsed {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SidebarModels::default, |st| {
                st.demo_collapsed = Some(model.clone())
            });
            model
        }
    };

    let demo_selected = match state.demo_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("playground"));
            cx.with_state(SidebarModels::default, |st| {
                st.demo_selected = Some(model.clone())
            });
            model
        }
    };

    let controlled_collapsed = match state.controlled_collapsed {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SidebarModels::default, |st| {
                st.controlled_collapsed = Some(model.clone())
            });
            model
        }
    };

    let controlled_selected = match state.controlled_selected {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(Arc::<str>::from("design-engineering"));
            cx.with_state(SidebarModels::default, |st| {
                st.controlled_selected = Some(model.clone())
            });
            model
        }
    };

    let rtl_selected = match state.rtl_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("playground"));
            cx.with_state(SidebarModels::default, |st| {
                st.rtl_selected = Some(model.clone())
            });
            model
        }
    };

    let resolve_selected =
        |cx: &mut ElementContext<'_, App>, model: &Model<Arc<str>>, fallback: &'static str| {
            cx.get_model_cloned(model, Invalidation::Layout)
                .unwrap_or_else(|| Arc::<str>::from(fallback))
        };

    let menu_button = |cx: &mut ElementContext<'_, App>,
                       selected_model: Model<Arc<str>>,
                       active_value: &Arc<str>,
                       value: &'static str,
                       label: &'static str,
                       icon: &'static str,
                       collapsed: bool,
                       test_id: Arc<str>| {
        let is_active = active_value.as_ref() == value;
        let selected_for_activate = selected_model.clone();
        let value_for_activate: Arc<str> = Arc::from(value);
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&selected_for_activate, |v| *v = value_for_activate.clone());
            host.request_redraw(action_cx.window);
        });

        shadcn::SidebarMenuButton::new(label)
            .icon(fret_icons::IconId::new_static(icon))
            .active(is_active)
            .collapsed(collapsed)
            .on_activate(on_activate)
            .test_id(test_id)
            .into_element(cx)
    };

    let demo = {
        let is_collapsed = cx
            .watch_model(&demo_collapsed)
            .layout()
            .copied()
            .unwrap_or(false);
        let selected_value = resolve_selected(cx, &demo_selected, "playground");

        let toolbar = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Button::new("Toggle")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .toggle_model(demo_collapsed.clone())
                        .test_id("ui-gallery-sidebar-demo-toggle")
                        .into_element(cx),
                    shadcn::typography::muted(
                        cx,
                        if is_collapsed {
                            "Collapsed to icon rail"
                        } else {
                            "Expanded"
                        },
                    ),
                    shadcn::typography::muted(cx, format!("active={}", selected_value.as_ref())),
                ]
            },
        );

        let platform = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Platform")
                .collapsed(is_collapsed)
                .into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "playground",
                    "Playground",
                    "lucide.square-terminal",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-playground"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "models",
                    "Models",
                    "lucide.bot",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-models"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "documentation",
                    "Documentation",
                    "lucide.book-open",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-documentation"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "settings",
                    "Settings",
                    "lucide.settings-2",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-settings"),
                ))
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let projects = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Projects")
                .collapsed(is_collapsed)
                .into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "design-engineering",
                    "Design Engineering",
                    "lucide.frame",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-design-engineering"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "sales-marketing",
                    "Sales & Marketing",
                    "lucide.chart-pie",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-sales-marketing"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    demo_selected.clone(),
                    &selected_value,
                    "travel",
                    "Travel",
                    "lucide.map",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-demo-item-travel"),
                ))
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let sidebar = shadcn::Sidebar::new([
            shadcn::SidebarHeader::new([shadcn::typography::small(cx, "Acme Inc.")])
                .into_element(cx),
            shadcn::SidebarContent::new([platform, projects])
                .collapsed(is_collapsed)
                .into_element(cx),
            shadcn::SidebarFooter::new([shadcn::typography::small(cx, "shadcn")]).into_element(cx),
        ])
        .collapsed(is_collapsed)
        .refine_layout(LayoutRefinement::default().h_full())
        .into_element(cx);

        let content = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Content").into_element(cx)])
                .into_element(cx),
            shadcn::CardContent::new(vec![
                cx.text("A sidebar that collapses to icon mode."),
                cx.text("Select any menu item to verify active and hover states."),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
        .into_element(cx);

        let frame = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_px(Px(360.0))),
            |_cx| vec![sidebar, content],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sidebar-demo"),
        );

        let framed = shell(cx, frame);
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![toolbar, framed],
        );
        section(cx, "Demo", body)
    };

    let controlled = {
        let is_collapsed = cx
            .watch_model(&controlled_collapsed)
            .layout()
            .copied()
            .unwrap_or(false);
        let selected_value = resolve_selected(cx, &controlled_selected, "design-engineering");

        let header = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            |cx| {
                vec![
                    shadcn::Button::new(if is_collapsed {
                        "Open Sidebar"
                    } else {
                        "Close Sidebar"
                    })
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .toggle_model(controlled_collapsed.clone())
                    .test_id("ui-gallery-sidebar-controlled-toggle")
                    .into_element(cx),
                    shadcn::typography::muted(
                        cx,
                        "Controlled via model (approximation of SidebarProvider open state).",
                    ),
                ]
            },
        );

        let projects = shadcn::SidebarGroup::new([
            shadcn::SidebarGroupLabel::new("Projects")
                .collapsed(is_collapsed)
                .into_element(cx),
            shadcn::SidebarMenu::new([
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "design-engineering",
                    "Design Engineering",
                    "lucide.frame",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-design-engineering"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "sales-marketing",
                    "Sales & Marketing",
                    "lucide.chart-pie",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-sales-marketing"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "travel",
                    "Travel",
                    "lucide.map",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-travel"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "support",
                    "Support",
                    "lucide.life-buoy",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-support"),
                ))
                .into_element(cx),
                shadcn::SidebarMenuItem::new(menu_button(
                    cx,
                    controlled_selected.clone(),
                    &selected_value,
                    "feedback",
                    "Feedback",
                    "lucide.send",
                    is_collapsed,
                    Arc::from("ui-gallery-sidebar-controlled-item-feedback"),
                ))
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx);

        let sidebar = shadcn::Sidebar::new([shadcn::SidebarContent::new([projects])
            .collapsed(is_collapsed)
            .into_element(cx)])
        .collapsed(is_collapsed)
        .refine_layout(LayoutRefinement::default().h_full())
        .into_element(cx);

        let inset = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Sidebar Inset").into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![
                cx.text("Use a main content panel next to Sidebar when controlled."),
                cx.text(format!("selected={}", selected_value.as_ref())),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
        .into_element(cx);

        let frame = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_px(Px(320.0))),
            |_cx| vec![sidebar, inset],
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sidebar-controlled"),
        );

        let framed = shell(cx, frame);
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![header, framed],
        );

        section(cx, "Controlled", body)
    };

    let rtl = {
        let selected_value = resolve_selected(cx, &rtl_selected, "playground");

        let rtl_layout = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let platform = shadcn::SidebarGroup::new([
                    shadcn::SidebarGroupLabel::new("??????")
                        .collapsed(false)
                        .into_element(cx),
                    shadcn::SidebarMenu::new([
                        shadcn::SidebarMenuItem::new(menu_button(
                            cx,
                            rtl_selected.clone(),
                            &selected_value,
                            "playground",
                            "????",
                            "lucide.square-terminal",
                            false,
                            Arc::from("ui-gallery-sidebar-rtl-item-playground"),
                        ))
                        .into_element(cx),
                        shadcn::SidebarMenuItem::new(menu_button(
                            cx,
                            rtl_selected.clone(),
                            &selected_value,
                            "documentation",
                            "???????",
                            "lucide.book-open",
                            false,
                            Arc::from("ui-gallery-sidebar-rtl-item-documentation"),
                        ))
                        .into_element(cx),
                        shadcn::SidebarMenuItem::new(menu_button(
                            cx,
                            rtl_selected.clone(),
                            &selected_value,
                            "settings",
                            "?????????",
                            "lucide.settings-2",
                            false,
                            Arc::from("ui-gallery-sidebar-rtl-item-settings"),
                        ))
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx);

                let sidebar = shadcn::Sidebar::new([
                    shadcn::SidebarHeader::new([shadcn::typography::small(cx, "?????? ???????")])
                        .into_element(cx),
                    shadcn::SidebarContent::new([platform])
                        .collapsed(false)
                        .into_element(cx),
                    shadcn::SidebarFooter::new([shadcn::typography::small(cx, "??????")])
                        .into_element(cx),
                ])
                .collapsed(false)
                .refine_layout(LayoutRefinement::default().h_full())
                .into_element(cx);

                let content = shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![shadcn::CardTitle::new("RTL").into_element(cx)])
                        .into_element(cx),
                    shadcn::CardContent::new(vec![
                        cx.text("Direction provider flips layout and inline icon/text flow."),
                        cx.text(format!("active={}", selected_value.as_ref())),
                    ])
                    .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
                .into_element(cx);

                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N4)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().h_px(Px(320.0))),
                    |_cx| vec![content, sidebar],
                )
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sidebar-rtl"),
        );

        let framed = shell(cx, rtl_layout);
        let body = centered(cx, framed);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("A composable, themeable and customizable sidebar component."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, controlled, rtl]
        }),
    ]
}

pub(in crate::ui) fn preview_radio_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_ui_kit::primitives::direction as direction_prim;

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let w_fit = LayoutRefinement::default().w(fret_ui_kit::LengthRefinement::Auto);
    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));

    let demo = cx.keyed("ui_gallery.radio_group.demo", |cx| {
        let group = shadcn::RadioGroup::uncontrolled(Some("comfortable"))
            .a11y_label("Options")
            .refine_layout(w_fit.clone())
            .item(shadcn::RadioGroupItem::new("default", "Default"))
            .item(shadcn::RadioGroupItem::new("comfortable", "Comfortable"))
            .item(shadcn::RadioGroupItem::new("compact", "Compact"))
            .into_element(cx);

        let body = centered(cx, group);
        section(cx, "Demo", body)
    });

    let description = cx.keyed("ui_gallery.radio_group.description", |cx| {
        let group = shadcn::RadioGroup::uncontrolled(Some("comfortable"))
            .a11y_label("Options")
            .refine_layout(w_fit.clone())
            .item(
                shadcn::RadioGroupItem::new("default", "Default").child(
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Default").into_element(cx),
                        shadcn::FieldDescription::new("Standard spacing for most use cases.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ),
            )
            .item(
                shadcn::RadioGroupItem::new("comfortable", "Comfortable").child(
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Comfortable").into_element(cx),
                        shadcn::FieldDescription::new("More space between elements.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ),
            )
            .item(
                shadcn::RadioGroupItem::new("compact", "Compact").child(
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Compact").into_element(cx),
                        shadcn::FieldDescription::new("Minimal spacing for dense layouts.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ),
            )
            .into_element(cx);

        let body = centered(cx, group);
        section(cx, "Description", body)
    });

    let choice_card = cx.keyed("ui_gallery.radio_group.choice_card", |cx| {
        let group = shadcn::RadioGroup::uncontrolled(Some("plus"))
            .a11y_label("Subscription plans")
            .refine_layout(max_w_sm.clone())
            .item(
                shadcn::RadioGroupItem::new("plus", "Plus")
                    .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                    .child(
                        shadcn::FieldContent::new([
                            shadcn::FieldTitle::new("Plus").into_element(cx),
                            shadcn::FieldDescription::new("For individuals and small teams.")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ),
            )
            .item(
                shadcn::RadioGroupItem::new("pro", "Pro")
                    .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                    .child(
                        shadcn::FieldContent::new([
                            shadcn::FieldTitle::new("Pro").into_element(cx),
                            shadcn::FieldDescription::new("For growing businesses.")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ),
            )
            .item(
                shadcn::RadioGroupItem::new("enterprise", "Enterprise")
                    .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                    .child(
                        shadcn::FieldContent::new([
                            shadcn::FieldTitle::new("Enterprise").into_element(cx),
                            shadcn::FieldDescription::new("For large teams and enterprises.")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ),
            )
            .into_element(cx);

        let body = centered(cx, group);
        section(cx, "Choice Card", body)
    });

    let fieldset = cx.keyed("ui_gallery.radio_group.fieldset", |cx| {
        let group = shadcn::RadioGroup::uncontrolled(Some("monthly"))
            .a11y_label("Subscription plan")
            .item(shadcn::RadioGroupItem::new(
                "monthly",
                "Monthly ($9.99/month)",
            ))
            .item(shadcn::RadioGroupItem::new(
                "yearly",
                "Yearly ($99.99/year)",
            ))
            .item(shadcn::RadioGroupItem::new(
                "lifetime",
                "Lifetime ($299.99)",
            ))
            .into_element(cx);

        let fieldset = shadcn::FieldSet::new([
            shadcn::FieldLegend::new("Subscription Plan")
                .variant(shadcn::FieldLegendVariant::Label)
                .into_element(cx),
            shadcn::FieldDescription::new("Yearly and lifetime plans offer significant savings.")
                .into_element(cx),
            group,
        ])
        .refine_layout(max_w_xs.clone())
        .into_element(cx);

        let body = centered(cx, fieldset);
        section(cx, "Fieldset", body)
    });

    let disabled = cx.keyed("ui_gallery.radio_group.disabled", |cx| {
        let group = shadcn::RadioGroup::uncontrolled(Some("option2"))
            .a11y_label("Options")
            .refine_layout(w_fit.clone())
            .item(shadcn::RadioGroupItem::new("option1", "Disabled").disabled(true))
            .item(shadcn::RadioGroupItem::new("option2", "Option 2"))
            .item(shadcn::RadioGroupItem::new("option3", "Option 3"))
            .into_element(cx);

        let body = centered(cx, group);
        section(cx, "Disabled", body)
    });

    let invalid = cx.keyed("ui_gallery.radio_group.invalid", |cx| {
        let destructive = cx.with_theme(|theme| theme.color_required("destructive"));

        let group = shadcn::RadioGroup::uncontrolled(Some("email"))
            .a11y_label("Notification Preferences")
            .refine_layout(LayoutRefinement::default().w_full())
            .item(
                shadcn::RadioGroupItem::new("email", "Email only")
                    .aria_invalid(true)
                    .child(
                        ui::label(cx, "Email only")
                            .text_color(ColorRef::Color(destructive))
                            .into_element(cx),
                    ),
            )
            .item(
                shadcn::RadioGroupItem::new("sms", "SMS only")
                    .aria_invalid(true)
                    .child(
                        ui::label(cx, "SMS only")
                            .text_color(ColorRef::Color(destructive))
                            .into_element(cx),
                    ),
            )
            .item(
                shadcn::RadioGroupItem::new("both", "Both Email & SMS")
                    .aria_invalid(true)
                    .child(
                        ui::label(cx, "Both Email & SMS")
                            .text_color(ColorRef::Color(destructive))
                            .into_element(cx),
                    ),
            )
            .into_element(cx);

        let fieldset = shadcn::FieldSet::new([
            shadcn::FieldLegend::new("Notification Preferences")
                .variant(shadcn::FieldLegendVariant::Label)
                .into_element(cx),
            shadcn::FieldDescription::new("Choose how you want to receive notifications.")
                .into_element(cx),
            group,
        ])
        .refine_layout(max_w_xs.clone())
        .into_element(cx);

        let body = centered(cx, fieldset);
        section(cx, "Invalid", body)
    });

    let rtl = cx.keyed("ui_gallery.radio_group.rtl", |cx| {
        let group = direction_prim::with_direction_provider(
            cx,
            direction_prim::LayoutDirection::Rtl,
            |cx| {
                shadcn::RadioGroup::uncontrolled(Some("comfortable"))
                    .a11y_label("خيارات")
                    .refine_layout(w_fit.clone())
                    .item(
                        shadcn::RadioGroupItem::new("default", "افتراضي").child(
                            shadcn::FieldContent::new([
                                shadcn::FieldLabel::new("افتراضي").into_element(cx),
                                shadcn::FieldDescription::new("تباعد قياسي لمعظم حالات الاستخدام.")
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                        ),
                    )
                    .item(
                        shadcn::RadioGroupItem::new("comfortable", "مريح").child(
                            shadcn::FieldContent::new([
                                shadcn::FieldLabel::new("مريح").into_element(cx),
                                shadcn::FieldDescription::new("مساحة أكبر بين العناصر.")
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                        ),
                    )
                    .item(
                        shadcn::RadioGroupItem::new("compact", "مضغوط").child(
                            shadcn::FieldContent::new([
                                shadcn::FieldLabel::new("مضغوط").into_element(cx),
                                shadcn::FieldDescription::new("تباعد أدنى للتخطيطات الكثيفة.")
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                        ),
                    )
                    .into_element(cx)
            },
        );

        let body = centered(cx, group);
        section(cx, "RTL", body)
    });

    let examples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![description, choice_card, fieldset, disabled, invalid, rtl],
    );

    vec![demo, examples]
}

pub(in crate::ui) fn preview_toggle(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_toggle(cx)
}

pub(in crate::ui) fn preview_toggle_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_toggle_group(cx)
}

pub(in crate::ui) fn preview_typography(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_typography(cx)
}

pub(in crate::ui) fn preview_alert_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_alert_dialog(cx, open)
}

pub(in crate::ui) fn preview_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_dialog(cx, open)
}

pub(in crate::ui) fn preview_popover(
    cx: &mut ElementContext<'_, App>,
    _open: Model<bool>,
) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct PopoverModels {
        demo_width: Option<Model<String>>,
        demo_max_width: Option<Model<String>>,
        demo_height: Option<Model<String>>,
        demo_max_height: Option<Model<String>>,
        form_width: Option<Model<String>>,
        form_height: Option<Model<String>>,
    }

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let state = cx.with_state(PopoverModels::default, |st| st.clone());
    let demo_width = match state.demo_width {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("100%"));
            cx.with_state(PopoverModels::default, |st| {
                st.demo_width = Some(model.clone())
            });
            model
        }
    };
    let demo_max_width = match state.demo_max_width {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("300px"));
            cx.with_state(PopoverModels::default, |st| {
                st.demo_max_width = Some(model.clone())
            });
            model
        }
    };
    let demo_height = match state.demo_height {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("25px"));
            cx.with_state(PopoverModels::default, |st| {
                st.demo_height = Some(model.clone())
            });
            model
        }
    };
    let demo_max_height = match state.demo_max_height {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("none"));
            cx.with_state(PopoverModels::default, |st| {
                st.demo_max_height = Some(model.clone())
            });
            model
        }
    };
    let form_width = match state.form_width {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("100%"));
            cx.with_state(PopoverModels::default, |st| {
                st.form_width = Some(model.clone())
            });
            model
        }
    };
    let form_height = match state.form_height {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("25px"));
            cx.with_state(PopoverModels::default, |st| {
                st.form_height = Some(model.clone())
            });
            model
        }
    };

    let demo = {
        let popover = shadcn::Popover::new_controllable(cx, None, false).into_element(
            cx,
            |cx| {
                shadcn::Button::new("Open popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| {
                let row = |cx: &mut ElementContext<'_, App>,
                           label: &'static str,
                           model: Model<_>| {
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N4)
                            .items_center(),
                        move |cx| {
                            vec![
                                ui::label(cx, label)
                                    .layout(
                                        LayoutRefinement::default().w_px(Px(96.0)).flex_shrink_0(),
                                    )
                                    .into_element(cx),
                                shadcn::Input::new(model)
                                    .size(fret_ui_kit::Size::Small)
                                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                                    .into_element(cx),
                            ]
                        },
                    )
                };

                let header = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full()),
                    |cx| {
                        vec![
                            shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                            shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                                .into_element(cx),
                        ]
                    },
                );

                let fields = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N2)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full()),
                    move |cx| {
                        vec![
                            row(cx, "Width", demo_width.clone()),
                            row(cx, "Max. width", demo_max_width.clone()),
                            row(cx, "Height", demo_height.clone()),
                            row(cx, "Max. height", demo_max_height.clone()),
                        ]
                    },
                );

                shadcn::PopoverContent::new([header, fields])
                    .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
                    .into_element(cx)
            },
        );
        let body = centered(cx, popover);
        section(cx, "Demo", body)
    };

    let basic = {
        let popover = shadcn::Popover::new_controllable(cx, None, false)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open Popover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new([shadcn::PopoverHeader::new([
                        shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                        shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                            .into_element(cx),
                    ])
                    .into_element(cx)])
                    .into_element(cx)
                },
            );
        let body = centered(cx, popover);
        section(cx, "Basic", body)
    };

    let align = {
        let body = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N6)
                .items_center()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            |cx| {
                vec![
                    shadcn::Popover::new_controllable(cx, None, false)
                        .align(shadcn::PopoverAlign::Start)
                        .into_element(
                            cx,
                            |cx| {
                                shadcn::Button::new("Start")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([cx.text("Aligned to start")])
                                    .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                    .into_element(cx)
                            },
                        ),
                    shadcn::Popover::new_controllable(cx, None, false)
                        .align(shadcn::PopoverAlign::Center)
                        .into_element(
                            cx,
                            |cx| {
                                shadcn::Button::new("Center")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([cx.text("Aligned to center")])
                                    .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                    .into_element(cx)
                            },
                        ),
                    shadcn::Popover::new_controllable(cx, None, false)
                        .align(shadcn::PopoverAlign::End)
                        .into_element(
                            cx,
                            |cx| {
                                shadcn::Button::new("End")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([cx.text("Aligned to end")])
                                    .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                    .into_element(cx)
                            },
                        ),
                ]
            },
        );
        section(cx, "Align", body)
    };

    let with_form = {
        let popover = shadcn::Popover::new_controllable(cx, None, false)
            .align(shadcn::PopoverAlign::Start)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open Popover")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new([
                        shadcn::PopoverHeader::new([
                            shadcn::PopoverTitle::new("Dimensions").into_element(cx),
                            shadcn::PopoverDescription::new("Set the dimensions for the layer.")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::FieldGroup::new([
                            shadcn::Field::new([
                                shadcn::FieldLabel::new("Width")
                                    .refine_layout(LayoutRefinement::default().w_px(Px(128.0)))
                                    .into_element(cx),
                                shadcn::Input::new(form_width.clone())
                                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                                    .into_element(cx),
                            ])
                            .orientation(shadcn::FieldOrientation::Horizontal)
                            .into_element(cx),
                            shadcn::Field::new([
                                shadcn::FieldLabel::new("Height")
                                    .refine_layout(LayoutRefinement::default().w_px(Px(128.0)))
                                    .into_element(cx),
                                shadcn::Input::new(form_height.clone())
                                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                                    .into_element(cx),
                            ])
                            .orientation(shadcn::FieldOrientation::Horizontal)
                            .into_element(cx),
                        ])
                        .gap(Space::N4)
                        .into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_px(Px(256.0)))
                    .into_element(cx)
                },
            );
        let body = centered(cx, popover);
        section(cx, "With Form", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let popover = |cx: &mut ElementContext<'_, App>,
                               label: &'static str,
                               side: shadcn::PopoverSide| {
                    shadcn::Popover::new_controllable(cx, None, false)
                        .side(side)
                        .into_element(
                            cx,
                            |cx| {
                                shadcn::Button::new(label)
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .into_element(cx)
                            },
                            |cx| {
                                shadcn::PopoverContent::new([shadcn::PopoverHeader::new([
                                    shadcn::PopoverTitle::new("الأبعاد").into_element(cx),
                                    shadcn::PopoverDescription::new("تعيين الأبعاد للطبقة.")
                                        .into_element(cx),
                                ])
                                .into_element(cx)])
                                .into_element(cx)
                            },
                        )
                };

                let physical = stack::hstack_build(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N2)
                        .items_center()
                        .layout(LayoutRefinement::default().w_full())
                        .justify_center(),
                    |cx, out| {
                        for (id, label, side) in [
                            ("left", "يسار", shadcn::PopoverSide::Left),
                            ("top", "أعلى", shadcn::PopoverSide::Top),
                            ("bottom", "أسفل", shadcn::PopoverSide::Bottom),
                            ("right", "يمين", shadcn::PopoverSide::Right),
                        ] {
                            out.push(cx.keyed(id, |cx| popover(cx, label, side)));
                        }
                    },
                );

                let logical = stack::hstack_build(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N2)
                        .items_center()
                        .layout(LayoutRefinement::default().w_full())
                        .justify_center(),
                    |cx, out| {
                        for (id, label, side) in [
                            (
                                "inline-start",
                                "بداية السطر",
                                shadcn::PopoverSide::InlineStart,
                            ),
                            ("inline-end", "نهاية السطر", shadcn::PopoverSide::InlineEnd),
                        ] {
                            out.push(cx.keyed(id, |cx| popover(cx, label, side)));
                        }
                    },
                );

                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N4)
                        .layout(LayoutRefinement::default().w_full()),
                    move |_cx| [physical, logical],
                )
            },
        );
        section(cx, "RTL", body)
    };

    vec![demo, basic, align, with_form, rtl]
}

pub(in crate::ui) fn preview_sheet(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct SheetModels {
        demo_name: Option<Model<String>>,
        demo_username: Option<Model<String>>,
        side_top_open: Option<Model<bool>>,
        side_right_open: Option<Model<bool>>,
        side_bottom_open: Option<Model<bool>>,
        side_left_open: Option<Model<bool>>,
        no_close_open: Option<Model<bool>>,
        rtl_open: Option<Model<bool>>,
        rtl_name: Option<Model<String>>,
        rtl_username: Option<Model<String>>,
    }

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default().border_1().rounded(Radius::Md),
                layout,
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let state = cx.with_state(SheetModels::default, |st| st.clone());

    let demo_name = match state.demo_name {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("Pedro Duarte"));
            cx.with_state(SheetModels::default, |st| {
                st.demo_name = Some(model.clone())
            });
            model
        }
    };

    let demo_username = match state.demo_username {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("@peduarte"));
            cx.with_state(SheetModels::default, |st| {
                st.demo_username = Some(model.clone())
            });
            model
        }
    };

    let side_top_open = match state.side_top_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.side_top_open = Some(model.clone())
            });
            model
        }
    };

    let side_right_open = match state.side_right_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.side_right_open = Some(model.clone())
            });
            model
        }
    };

    let side_bottom_open = match state.side_bottom_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.side_bottom_open = Some(model.clone())
            });
            model
        }
    };

    let side_left_open = match state.side_left_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.side_left_open = Some(model.clone())
            });
            model
        }
    };

    let no_close_open = match state.no_close_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| {
                st.no_close_open = Some(model.clone())
            });
            model
        }
    };

    let rtl_open = match state.rtl_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(SheetModels::default, |st| st.rtl_open = Some(model.clone()));
            model
        }
    };

    let rtl_name = match state.rtl_name {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("Pedro Duarte"));
            cx.with_state(SheetModels::default, |st| st.rtl_name = Some(model.clone()));
            model
        }
    };

    let rtl_username = match state.rtl_username {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("peduarte"));
            cx.with_state(SheetModels::default, |st| {
                st.rtl_username = Some(model.clone())
            });
            model
        }
    };

    let profile_fields =
        |cx: &mut ElementContext<'_, App>, name: Model<String>, username: Model<String>| {
            let field =
                |cx: &mut ElementContext<'_, App>, label: &'static str, model: Model<String>| {
                    shadcn::Field::new([
                        shadcn::FieldLabel::new(label).into_element(cx),
                        shadcn::Input::new(model)
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                    ])
                    .into_element(cx)
                };

            shadcn::FieldSet::new([field(cx, "Name", name), field(cx, "Username", username)])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
        };

    let demo = {
        let trigger_open = open.clone();
        let save_open = open.clone();
        let close_open = open.clone();
        let name_model = demo_name.clone();
        let username_model = demo_username.clone();

        let demo_sheet = shadcn::Sheet::new(open.clone())
            .side(shadcn::SheetSide::Right)
            .size(Px(420.0))
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(trigger_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::SheetContent::new([
                        shadcn::SheetHeader::new([
                            shadcn::SheetTitle::new("Edit profile").into_element(cx),
                            shadcn::SheetDescription::new(
                                "Make changes to your profile here. Click save when you're done.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        profile_fields(cx, name_model.clone(), username_model.clone()),
                        shadcn::SheetFooter::new([
                            shadcn::Button::new("Save changes")
                                .toggle_model(save_open.clone())
                                .into_element(cx),
                            shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Outline)
                                .toggle_model(close_open.clone())
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default().test_id("ui-gallery-sheet-demo-content"),
                    )
                },
            )
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-gallery-sheet-demo"),
            );

        let card = shell(cx, LayoutRefinement::default(), demo_sheet);
        let body = centered(cx, card);
        section(cx, "Demo", body)
    };

    let side = {
        let side_sheet = |cx: &mut ElementContext<'_, App>,
                          id: &'static str,
                          label: &'static str,
                          side: shadcn::SheetSide,
                          open_model: Model<bool>| {
            let trigger_open = open_model.clone();
            let save_open = open_model.clone();
            let cancel_open = open_model.clone();
            let size = if matches!(side, shadcn::SheetSide::Top | shadcn::SheetSide::Bottom) {
                Px(320.0)
            } else {
                Px(420.0)
            };

            shadcn::Sheet::new(open_model)
                .side(side)
                .size(size)
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new(label)
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(trigger_open.clone())
                            .test_id(format!("ui-gallery-sheet-side-{id}-trigger"))
                            .into_element(cx)
                    },
                    |cx| {
                        let paragraphs = stack::vstack(
                            cx,
                            stack::VStackProps::default().gap(Space::N2),
                            |cx| {
                                (0..8)
                                    .map(|idx| {
                                        shadcn::typography::muted(
                                            cx,
                                            format!(
                                                "Profile section line {}. Keep this content scrollable for constrained sheets.",
                                                idx + 1
                                            ),
                                        )
                                    })
                                    .collect::<Vec<_>>()
                            },
                        );

                        let scroll = shadcn::ScrollArea::new([paragraphs])
                            .axis(fret_ui::element::ScrollAxis::Y)
                            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(180.0)))
                            .into_element(cx);

                        shadcn::SheetContent::new([
                            shadcn::SheetHeader::new([
                                shadcn::SheetTitle::new("Edit profile").into_element(cx),
                                shadcn::SheetDescription::new(
                                    "Use side to control which edge the sheet appears from.",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                            scroll,
                            shadcn::SheetFooter::new([
                                shadcn::Button::new("Save changes")
                                    .toggle_model(save_open.clone())
                                    .into_element(cx),
                                shadcn::Button::new("Cancel")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .toggle_model(cancel_open.clone())
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                        ])
                        .into_element(cx)
                    },
                )
        };

        let row = stack::hstack_build(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            |cx, out| {
                let items = [
                    ("top", "Top", shadcn::SheetSide::Top, side_top_open.clone()),
                    (
                        "right",
                        "Right",
                        shadcn::SheetSide::Right,
                        side_right_open.clone(),
                    ),
                    (
                        "bottom",
                        "Bottom",
                        shadcn::SheetSide::Bottom,
                        side_bottom_open.clone(),
                    ),
                    (
                        "left",
                        "Left",
                        shadcn::SheetSide::Left,
                        side_left_open.clone(),
                    ),
                ];
                for (id, label, side, open_model) in items {
                    out.push(
                        cx.keyed(id, |cx| side_sheet(cx, id, label, side, open_model.clone())),
                    );
                }
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sheet-side"),
        );

        let card = shell(cx, LayoutRefinement::default(), row);
        let body = centered(cx, card);
        section(cx, "Side", body)
    };

    let no_close_button = {
        let trigger_open = no_close_open.clone();

        let sheet = shadcn::Sheet::new(no_close_open.clone()).into_element(
            cx,
            |cx| {
                shadcn::Button::new("Open Sheet")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(trigger_open.clone())
                    .into_element(cx)
            },
            |cx| {
                shadcn::SheetContent::new([
                    shadcn::SheetHeader::new([
                        shadcn::SheetTitle::new("No Close Button").into_element(cx),
                        shadcn::SheetDescription::new(
                            "This example intentionally omits footer actions. Use outside press or Escape to close.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx)
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sheet-no-close-button"),
        );

        let card = shell(cx, LayoutRefinement::default(), sheet);
        let body = centered(cx, card);
        section(cx, "No Close Button", body)
    };

    let rtl = {
        let rtl_demo = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let trigger_open = rtl_open.clone();
                let save_open = rtl_open.clone();
                let close_open = rtl_open.clone();
                let name_model = rtl_name.clone();
                let username_model = rtl_username.clone();

                shadcn::Sheet::new(rtl_open.clone())
                    .side(shadcn::SheetSide::Left)
                    .size(Px(420.0))
                    .into_element(
                        cx,
                        |cx| {
                            shadcn::Button::new("Open")
                                .variant(shadcn::ButtonVariant::Outline)
                                .toggle_model(trigger_open.clone())
                                .into_element(cx)
                        },
                        |cx| {
                            shadcn::SheetContent::new([
                                shadcn::SheetHeader::new([
                                    shadcn::SheetTitle::new("Edit profile").into_element(cx),
                                    shadcn::SheetDescription::new(
                                        "RTL layout keeps spacing and focus flow aligned.",
                                    )
                                    .into_element(cx),
                                ])
                                .into_element(cx),
                                profile_fields(cx, name_model.clone(), username_model.clone()),
                                shadcn::SheetFooter::new([
                                    shadcn::Button::new("Save changes")
                                        .toggle_model(save_open.clone())
                                        .into_element(cx),
                                    shadcn::Button::new("Close")
                                        .variant(shadcn::ButtonVariant::Outline)
                                        .toggle_model(close_open.clone())
                                        .into_element(cx),
                                ])
                                .into_element(cx),
                            ])
                            .into_element(cx)
                        },
                    )
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sheet-rtl"),
        );

        let card = shell(cx, LayoutRefinement::default(), rtl_demo);
        let body = centered(cx, card);
        section(cx, "RTL", body)
    };

    vec![
        cx.text("Extends dialog to display side-aligned panels for supplementary tasks."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, side, no_close_button, rtl]
        }),
    ]
}

pub(in crate::ui) fn preview_empty(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_empty(cx)
}

pub(in crate::ui) fn preview_material3_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let row = |cx: &mut ElementContext<'_, App>,
               variant: material3::ButtonVariant,
               label: &'static str| {
        let (hover_container, hover_label) = cx.with_theme(|theme| {
            (
                theme.color_required("md.sys.color.tertiary-container"),
                theme.color_required("md.sys.color.on-tertiary-container"),
            )
        });

        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                let hover_style = material3::ButtonStyle::default()
                    .container_background(WidgetStateProperty::new(None).when(
                        WidgetStates::HOVERED,
                        Some(ColorRef::Color(hover_container)),
                    ))
                    .label_color(
                        WidgetStateProperty::new(None)
                            .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_label))),
                    );

                let accent = fret_core::Color {
                    r: 0.9,
                    g: 0.2,
                    b: 0.9,
                    a: 1.0,
                };
                let override_style = material3::ButtonStyle::default()
                    .label_color(WidgetStateProperty::new(Some(ColorRef::Color(accent))))
                    .state_layer_color(
                        WidgetStateProperty::new(None)
                            .when(WidgetStates::HOVERED, Some(ColorRef::Color(accent))),
                    );
                vec![
                    material3::Button::new(label)
                        .variant(variant)
                        .into_element(cx),
                    material3::Button::new("Override")
                        .variant(variant)
                        .style(override_style)
                        .into_element(cx),
                    material3::Button::new("Disabled")
                        .variant(variant)
                        .disabled(true)
                        .into_element(cx),
                    material3::Button::new("Hover Override")
                        .variant(variant)
                        .style(hover_style)
                        .into_element(cx),
                ]
            },
        )
    };

    vec![
        cx.text("Material 3 Buttons: token-driven colors + state layer + bounded ripple."),
        row(cx, material3::ButtonVariant::Filled, "Filled"),
        row(cx, material3::ButtonVariant::Tonal, "Tonal"),
        row(cx, material3::ButtonVariant::Elevated, "Elevated"),
        row(cx, material3::ButtonVariant::Outlined, "Outlined"),
        row(cx, material3::ButtonVariant::Text, "Text"),
    ]
}

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

pub(in crate::ui) fn preview_material3_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_core::{Corners, Px};
    use fret_ui::element::{AnyElement, ContainerProps, Length};

    let anchor = |cx: &mut ElementContext<'_, App>, size: Px, test_id: &'static str| {
        let mut props = ContainerProps::default();
        props.layout.size.width = Length::Px(size);
        props.layout.size.height = Length::Px(size);
        props.background =
            Some(cx.with_theme(|theme| theme.color_required("md.sys.color.surface-container-low")));
        props.corner_radii = Corners::all(Px(8.0));
        cx.container(props, |_cx| Vec::<AnyElement>::new())
            .test_id(test_id)
    };

    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        |cx| {
            let small = Px(24.0);
            vec![
                material3::Badge::dot()
                    .navigation_anchor_size(small)
                    .test_id("ui-gallery-material3-badge-dot-nav")
                    .into_element(cx, |cx| vec![anchor(cx, small, "badge-anchor-dot-nav")]),
                material3::Badge::text("9")
                    .navigation_anchor_size(small)
                    .test_id("ui-gallery-material3-badge-text-nav")
                    .into_element(cx, |cx| vec![anchor(cx, small, "badge-anchor-text-nav")]),
                material3::Badge::dot()
                    .placement(material3::BadgePlacement::TopRight)
                    .test_id("ui-gallery-material3-badge-dot-top-right")
                    .into_element(cx, |cx| {
                        vec![anchor(cx, Px(40.0), "badge-anchor-dot-top-right")]
                    }),
                material3::Badge::text("99+")
                    .placement(material3::BadgePlacement::TopRight)
                    .test_id("ui-gallery-material3-badge-text-top-right")
                    .into_element(cx, |cx| {
                        vec![anchor(cx, Px(40.0), "badge-anchor-text-top-right")]
                    }),
            ]
        },
    );

    vec![
        cx.text("Material 3 Badge: dot + large/value variants via md.comp.badge.*."),
        row,
    ]
}

pub(in crate::ui) fn preview_material3_top_app_bar(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    use fret_icons::ids;
    use fret_ui_material3::{
        TopAppBar, TopAppBarAction, TopAppBarScrollBehavior, TopAppBarVariant,
    };

    let bar = |cx: &mut ElementContext<'_, App>,
               variant: TopAppBarVariant,
               scrolled: bool,
               title: &'static str,
               test_id: &'static str| {
        TopAppBar::new(title)
            .variant(variant)
            .scrolled(scrolled)
            .navigation_icon(
                TopAppBarAction::new(ids::ui::CHEVRON_RIGHT)
                    .a11y_label("Navigate")
                    .test_id(format!("{test_id}-nav")),
            )
            .actions(vec![
                TopAppBarAction::new(ids::ui::SEARCH)
                    .a11y_label("Search")
                    .test_id(format!("{test_id}-search")),
                TopAppBarAction::new(ids::ui::MORE_HORIZONTAL)
                    .a11y_label("More actions")
                    .test_id(format!("{test_id}-more")),
            ])
            .test_id(test_id)
            .into_element(cx)
    };

    let scroll_demo = |cx: &mut ElementContext<'_, App>,
                       key: &'static str,
                       title: &'static str,
                       variant: TopAppBarVariant,
                       behavior: fn(fret_ui::scroll::ScrollHandle) -> TopAppBarScrollBehavior,
                       test_prefix: &'static str| {
        cx.keyed(key, |cx| {
            let scroll_handle =
                cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());
            let behavior = cx.with_state(
                || behavior(scroll_handle.clone()),
                |behavior| behavior.clone(),
            );
            let bar = TopAppBar::new(title)
                .variant(variant)
                .scroll_behavior(behavior)
                .navigation_icon(
                    TopAppBarAction::new(ids::ui::CHEVRON_RIGHT)
                        .a11y_label("Navigate")
                        .test_id(format!("{test_prefix}-nav")),
                )
                .actions(vec![
                    TopAppBarAction::new(ids::ui::MORE_HORIZONTAL)
                        .a11y_label("More actions")
                        .test_id(format!("{test_prefix}-more")),
                ])
                .test_id(test_prefix)
                .into_element(cx);

            let mut content_props = stack::VStackProps::default();
            content_props.gap = Space::N2;
            let content = stack::vstack(cx, content_props, |cx| {
                let mut out: Vec<AnyElement> = Vec::new();
                out.push(cx.text("Scroll this area to drive the TopAppBar scroll behavior."));
                for i in 0..80usize {
                    out.push(cx.text(format!("Row {i:02}")));
                }
                out
            });

            let scroll = shadcn::ScrollArea::new([content])
                .scroll_handle(scroll_handle.clone())
                .refine_layout(LayoutRefinement::default().w_full().h_px(Px(240.0)))
                .viewport_test_id(format!("{test_prefix}-scroll-viewport"))
                .into_element(cx);

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N4),
                |_cx| [bar, scroll],
            )
        })
    };

    let mut props = stack::VStackProps::default();
    props.gap = Space::N4;
    let content = stack::vstack(cx, props, |cx| {
        vec![
            cx.text("Material 3 Top App Bar: primitives driven by md.comp.top-app-bar.* tokens."),
            cx.text("Scroll behavior demos (policy-only, no fret-ui mechanism changes):"),
            scroll_demo(
                cx,
                "ui-gallery-material3-top-app-bar-scroll-pinned",
                "Pinned scroll behavior (toggle scrolled container treatment)",
                TopAppBarVariant::Small,
                TopAppBarScrollBehavior::pinned,
                "ui-gallery-material3-top-app-bar-pinned",
            ),
            scroll_demo(
                cx,
                "ui-gallery-material3-top-app-bar-scroll-enter-always",
                "EnterAlways scroll behavior (collapses fully, shows on reverse scroll)",
                TopAppBarVariant::Small,
                TopAppBarScrollBehavior::enter_always,
                "ui-gallery-material3-top-app-bar-enter-always",
            ),
            scroll_demo(
                cx,
                "ui-gallery-material3-top-app-bar-scroll-enter-always-settle-on-idle",
                "EnterAlways + settleOnIdle (policy-only spring settle after idle)",
                TopAppBarVariant::Small,
                |h| TopAppBarScrollBehavior::enter_always(h).settle_on_idle(),
                "ui-gallery-material3-top-app-bar-enter-always-settle-on-idle",
            ),
            scroll_demo(
                cx,
                "ui-gallery-material3-top-app-bar-scroll-exit-until-collapsed",
                "ExitUntilCollapsed scroll behavior (Large collapses down to Small height)",
                TopAppBarVariant::Large,
                TopAppBarScrollBehavior::exit_until_collapsed,
                "ui-gallery-material3-top-app-bar-exit-until-collapsed",
            ),
            scroll_demo(
                cx,
                "ui-gallery-material3-top-app-bar-scroll-exit-until-collapsed-settle-on-idle",
                "ExitUntilCollapsed + settleOnIdle (policy-only snap; content moves)",
                TopAppBarVariant::Large,
                |h| TopAppBarScrollBehavior::exit_until_collapsed(h).settle_on_idle(),
                "ui-gallery-material3-top-app-bar-exit-until-collapsed-settle-on-idle",
            ),
            bar(
                cx,
                TopAppBarVariant::Small,
                false,
                "Small (idle)",
                "ui-gallery-material3-top-app-bar-small",
            ),
            bar(
                cx,
                TopAppBarVariant::Small,
                true,
                "Small (scrolled)",
                "ui-gallery-material3-top-app-bar-small-scrolled",
            ),
            bar(
                cx,
                TopAppBarVariant::SmallCentered,
                false,
                "Small Centered (idle)",
                "ui-gallery-material3-top-app-bar-small-centered",
            ),
            bar(
                cx,
                TopAppBarVariant::SmallCentered,
                true,
                "Small Centered (scrolled)",
                "ui-gallery-material3-top-app-bar-small-centered-scrolled",
            ),
            bar(
                cx,
                TopAppBarVariant::Medium,
                false,
                "Medium (idle)",
                "ui-gallery-material3-top-app-bar-medium",
            ),
            bar(
                cx,
                TopAppBarVariant::Medium,
                true,
                "Medium (scrolled)",
                "ui-gallery-material3-top-app-bar-medium-scrolled",
            ),
            bar(
                cx,
                TopAppBarVariant::Large,
                false,
                "Large (idle)",
                "ui-gallery-material3-top-app-bar-large",
            ),
            bar(
                cx,
                TopAppBarVariant::Large,
                true,
                "Large (scrolled)",
                "ui-gallery-material3-top-app-bar-large-scrolled",
            ),
        ]
    });

    vec![content]
}

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

pub(in crate::ui) fn preview_material3_tabs(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let fixed_tabs = material3::Tabs::new(value.clone())
        .a11y_label("Material 3 Tabs")
        .test_id("ui-gallery-material3-tabs")
        .items(vec![
            material3::TabItem::new("overview", "Overview")
                .a11y_label("Tab Overview")
                .test_id("ui-gallery-material3-tab-overview"),
            material3::TabItem::new("settings", "Settings")
                .a11y_label("Tab Settings")
                .test_id("ui-gallery-material3-tab-settings"),
            material3::TabItem::new("disabled", "Disabled")
                .disabled(true)
                .a11y_label("Tab Disabled")
                .test_id("ui-gallery-material3-tab-disabled"),
        ])
        .into_element(cx);

    let override_style = material3::TabsStyle::default()
        .label_color(WidgetStateProperty::new(None).when(
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
        ))
        .active_indicator_color(WidgetStateProperty::new(None).when(
            WidgetStates::SELECTED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.2,
                g: 0.8,
                b: 0.4,
                a: 1.0,
            })),
        ));
    let fixed_tabs_overridden = material3::Tabs::new(value.clone())
        .a11y_label("Material 3 Tabs (overridden)")
        .test_id("ui-gallery-material3-tabs-overridden")
        .style(override_style)
        .items(vec![
            material3::TabItem::new("overview", "Overview")
                .a11y_label("Tab Overview")
                .test_id("ui-gallery-material3-tab-overview-overridden"),
            material3::TabItem::new("settings", "Settings")
                .a11y_label("Tab Settings")
                .test_id("ui-gallery-material3-tab-settings-overridden"),
            material3::TabItem::new("disabled", "Disabled")
                .disabled(true)
                .a11y_label("Tab Disabled")
                .test_id("ui-gallery-material3-tab-disabled-overridden"),
        ])
        .into_element(cx);

    let scrollable_tabs = material3::Tabs::new(value)
        .a11y_label("Material 3 Tabs (scrollable)")
        .test_id("ui-gallery-material3-tabs-scrollable")
        .scrollable(true)
        .items(vec![
            material3::TabItem::new("overview", "Overview"),
            material3::TabItem::new("settings", "Settings"),
            material3::TabItem::new("typography", "Typography"),
            material3::TabItem::new("very_long_label", "Very Long Label For Layout Probe"),
            material3::TabItem::new("tokens", "Tokens"),
            material3::TabItem::new("motion", "Motion"),
            material3::TabItem::new("disabled", "Disabled").disabled(true),
        ])
        .into_element(cx);

    vec![
        cx.text("Material 3 Tabs: roving focus + state layer + bounded ripple."),
        fixed_tabs,
        cx.text(
            "Override preview: hover label/state-layer + active-indicator color via TabsStyle.",
        ),
        fixed_tabs_overridden,
        cx.text("Scrollable/variable width preview (measurement-driven indicator)."),
        scrollable_tabs,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

pub(in crate::ui) fn preview_material3_navigation_bar(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let bar = material3::NavigationBar::new(value)
        .a11y_label("Material 3 Navigation Bar")
        .test_id("ui-gallery-material3-navigation-bar")
        .items(vec![
            material3::NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                .badge_dot()
                .a11y_label("Destination Search")
                .test_id("ui-gallery-material3-nav-search"),
            material3::NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS)
                .a11y_label("Destination Settings")
                .test_id("ui-gallery-material3-nav-settings"),
            material3::NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL)
                .badge_text("9")
                .a11y_label("Destination More")
                .test_id("ui-gallery-material3-nav-more"),
        ])
        .into_element(cx);

    vec![
        cx.text("Material 3 Navigation Bar: roving focus + state layer + bounded ripple."),
        bar,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

pub(in crate::ui) fn preview_material3_navigation_rail(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let rail = material3::NavigationRail::new(value)
        .a11y_label("Material 3 Navigation Rail")
        .test_id("ui-gallery-material3-navigation-rail")
        .items(vec![
            material3::NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                .badge_dot()
                .a11y_label("Destination Search")
                .test_id("ui-gallery-material3-rail-search"),
            material3::NavigationRailItem::new("settings", "Settings", ids::ui::SETTINGS)
                .a11y_label("Destination Settings")
                .test_id("ui-gallery-material3-rail-settings"),
            material3::NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                .badge_text("99+")
                .a11y_label("Destination Play")
                .test_id("ui-gallery-material3-rail-play"),
            material3::NavigationRailItem::new("disabled", "Disabled", ids::ui::SLASH)
                .disabled(true)
                .a11y_label("Destination Disabled")
                .test_id("ui-gallery-material3-rail-disabled"),
        ])
        .into_element(cx);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = fret_ui::element::Length::Fill;
    layout.size.height = fret_ui::element::Length::Px(Px(360.0));

    let container = cx.container(
        fret_ui::element::ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| [rail],
    );

    vec![
        cx.text("Material 3 Navigation Rail: roving focus + state layer + bounded ripple."),
        container,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

pub(in crate::ui) fn preview_material3_navigation_drawer(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let drawer = material3::NavigationDrawer::new(value)
        .a11y_label("Material 3 Navigation Drawer")
        .test_id("ui-gallery-material3-navigation-drawer")
        .items(vec![
            material3::NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                .a11y_label("Destination Search")
                .test_id("ui-gallery-material3-drawer-search"),
            material3::NavigationDrawerItem::new("settings", "Settings", ids::ui::SETTINGS)
                .badge_label("2")
                .a11y_label("Destination Settings")
                .test_id("ui-gallery-material3-drawer-settings"),
            material3::NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                .badge_label("99+")
                .a11y_label("Destination Play")
                .test_id("ui-gallery-material3-drawer-play"),
            material3::NavigationDrawerItem::new("disabled", "Disabled", ids::ui::SLASH)
                .disabled(true)
                .a11y_label("Destination Disabled")
                .test_id("ui-gallery-material3-drawer-disabled"),
        ])
        .into_element(cx);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = fret_ui::element::Length::Fill;
    layout.size.height = fret_ui::element::Length::Px(Px(280.0));

    let container = cx.container(
        fret_ui::element::ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| [drawer],
    );

    vec![
        cx.text("Material 3 Navigation Drawer: roving focus + state layer + bounded ripple."),
        container,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

pub(in crate::ui) fn preview_material3_modal_navigation_drawer(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;
    use fret_ui::action::OnActivate;

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let open_drawer: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };

    let modal = material3::ModalNavigationDrawer::new(open.clone())
        .test_id("ui-gallery-material3-modal-navigation-drawer")
        .into_element(
            cx,
            move |cx| {
                material3::NavigationDrawer::new(value)
                    .variant(material3::NavigationDrawerVariant::Modal)
                    .a11y_label("Material 3 Modal Navigation Drawer")
                    .test_id("ui-gallery-material3-modal-navigation-drawer-panel")
                    .items(vec![
                        material3::NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("ui-gallery-material3-modal-drawer-search"),
                        material3::NavigationDrawerItem::new(
                            "settings",
                            "Settings",
                            ids::ui::SETTINGS,
                        )
                        .badge_label("2")
                        .a11y_label("Destination Settings")
                        .test_id("ui-gallery-material3-modal-drawer-settings"),
                        material3::NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .badge_label("99+")
                            .a11y_label("Destination Play")
                            .test_id("ui-gallery-material3-modal-drawer-play"),
                        material3::NavigationDrawerItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("ui-gallery-material3-modal-drawer-disabled"),
                    ])
                    .into_element(cx)
            },
            move |cx| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N4),
                    move |cx| {
                        vec![
                            material3::Button::new("Open drawer")
                                .variant(material3::ButtonVariant::Filled)
                                .on_activate(open_drawer.clone())
                                .test_id("ui-gallery-material3-modal-drawer-open")
                                .into_element(cx),
                            material3::Button::new("Underlay focus probe")
                                .variant(material3::ButtonVariant::Outlined)
                                .test_id("ui-gallery-material3-modal-drawer-underlay-probe")
                                .into_element(cx),
                            cx.text(
                                "Tip: click the scrim or press Esc to close; Tab/Shift+Tab should stay inside the drawer while open.",
                            ),
                        ]
                    },
                )
            },
        );

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = fret_ui::element::Length::Fill;
    layout.size.height = fret_ui::element::Length::Px(Px(360.0));

    let container = cx.container(
        fret_ui::element::ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| [modal],
    );

    vec![
        cx.text("Material 3 Modal Navigation Drawer: modal scrim + focus trap/restore + token-driven motion."),
        container,
        cx.text(format!(
            "open={} value={}",
            is_open as u8,
            current.as_ref()
        )),
    ]
}

pub(in crate::ui) fn preview_material3_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui_kit::{ColorRef, WidgetStateProperty};

    #[derive(Default)]
    struct DialogPageModels {
        override_open: Option<Model<bool>>,
        selected: Option<Model<Option<Arc<str>>>>,
    }

    let override_open = cx.with_state(DialogPageModels::default, |st| st.override_open.clone());
    let override_open = match override_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DialogPageModels::default, |st| {
                st.override_open = Some(model.clone())
            });
            model
        }
    };

    let selected = cx.with_state(DialogPageModels::default, |st| st.selected.clone());
    let selected = match selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(DialogPageModels::default, |st| {
                st.selected = Some(model.clone())
            });
            model
        }
    };

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let override_is_open = cx
        .get_model_copied(&override_open, Invalidation::Layout)
        .unwrap_or(false);

    let open_dialog: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let close_dialog: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let confirm_action: OnActivate = {
        let open = open.clone();
        let last_action = last_action.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from("material3.dialog.confirm")
            });
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };

    let theme = cx.theme().clone();
    let select_items: Arc<[material3::SelectItem]> = (0..20)
        .map(|i| {
            material3::SelectItem::new(
                Arc::<str>::from(format!("item-{i:02}")),
                Arc::<str>::from(format!("Item {i:02}")),
            )
            .test_id(format!("ui-gallery-material3-dialog-select-item-{i:02}"))
        })
        .collect::<Vec<_>>()
        .into();
    let override_style = material3::DialogStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.secondary-container"),
        ))))
        .headline_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.on-secondary-container"),
        ))))
        .supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.on-secondary-container"),
        ))));

    let open_dialog_override: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            let _ = host.models_mut().update(&override_open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };
    let close_dialog_override: OnActivate = {
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let confirm_action_override: OnActivate = {
        let override_open = override_open.clone();
        let last_action = last_action.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from("material3.dialog.confirm.override")
            });
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };

    let build_dialog = |cx: &mut ElementContext<'_, App>,
                        open_model: Model<bool>,
                        style: Option<material3::DialogStyle>,
                        id_prefix: &'static str,
                        open_action: OnActivate,
                        close_action: OnActivate,
                        confirm_action: OnActivate|
     -> AnyElement {
        let mut dialog = material3::Dialog::new(open_model.clone())
            .headline("Discard draft?")
            .supporting_text("This action cannot be undone.")
            .actions(vec![
                material3::DialogAction::new("Cancel")
                    .test_id(format!("{id_prefix}-action-cancel"))
                    .on_activate(close_action.clone()),
                material3::DialogAction::new("Discard")
                    .test_id(format!("{id_prefix}-action-discard"))
                    .on_activate(confirm_action.clone()),
            ])
            .test_id(format!("{id_prefix}"));

        if let Some(style) = style {
            dialog = dialog.style(style);
        }

        dialog.into_element(
            cx,
            move |cx| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N4),
                    move |cx| {
                        vec![
                            material3::Button::new("Open dialog")
                                .variant(material3::ButtonVariant::Filled)
                                .on_activate(open_action.clone())
                                .test_id(format!("{id_prefix}-open"))
                                .into_element(cx),
                            material3::Button::new("Underlay focus probe")
                                .variant(material3::ButtonVariant::Outlined)
                                .test_id(format!("{id_prefix}-underlay-probe"))
                                .into_element(cx),
                            cx.text("Tip: press Esc or click the scrim to close; Tab should stay inside the dialog while open."),
                        ]
                    },
                )
            },
            {
                let selected = selected.clone();
                let select_items = select_items.clone();
                move |cx| {
                    let spacer = cx.container(
                        fret_ui::element::ContainerProps {
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.size.width = fret_ui::element::Length::Fill;
                                l.size.height = fret_ui::element::Length::Px(Px(480.0));
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
                                material3::Select::new(selected.clone())
                                    .a11y_label("Material 3 Select (dialog)")
                                    .placeholder("Pick one")
                                    .items(select_items.clone())
                                    .match_anchor_width(false)
                                    .test_id(format!("{id_prefix}-select"))
                                    .into_element(cx),
                                cx.text(
                                    "Bottom-edge clamping probe: open the Select menu near the window bottom.",
                                ),
                                spacer,
                                material3::Select::new(selected.clone())
                                    .a11y_label("Material 3 Select (dialog, bottom)")
                                    .placeholder("Pick one")
                                    .items(select_items.clone())
                                    .match_anchor_width(false)
                                    .test_id(format!("{id_prefix}-select-bottom"))
                                    .into_element(cx),
                            ]
                        },
                    )]
                }
            },
        )
    };

    let default_dialog = build_dialog(
        cx,
        open.clone(),
        None,
        "ui-gallery-material3-dialog",
        open_dialog.clone(),
        close_dialog.clone(),
        confirm_action.clone(),
    );
    let override_dialog = build_dialog(
        cx,
        override_open.clone(),
        Some(override_style),
        "ui-gallery-material3-dialog-override",
        open_dialog_override.clone(),
        close_dialog_override.clone(),
        confirm_action_override.clone(),
    );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let build_container = |cx: &mut ElementContext<'_, App>, dialog: AnyElement| -> AnyElement {
        let mut layout = fret_ui::element::LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Px(Px(360.0));
        cx.container(
            fret_ui::element::ContainerProps {
                layout,
                ..Default::default()
            },
            move |_cx| [dialog],
        )
    };

    let containers = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                build_container(cx, default_dialog),
                build_container(cx, override_dialog),
            ]
        },
    );

    vec![
        cx.text(
            "Material 3 Dialog: modal barrier + focus trap/restore + token-shaped dialog actions.",
        ),
        containers,
        cx.text(format!(
            "open={} override_open={} last_action={}",
            is_open as u8,
            override_is_open as u8,
            last.as_ref()
        )),
    ]
}

pub(in crate::ui) fn preview_material3_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui_kit::{ColorRef, WidgetStateProperty};

    #[derive(Default)]
    struct MenuPageModels {
        override_open: Option<Model<bool>>,
    }

    let override_open = cx.with_state(MenuPageModels::default, |st| st.override_open.clone());
    let override_open = match override_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(MenuPageModels::default, |st| {
                st.override_open = Some(model.clone())
            });
            model
        }
    };

    fn on_select(id: &'static str, last_action: Model<Arc<str>>) -> OnActivate {
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from(id);
            });
            host.request_redraw(action_cx.window);
        })
    }

    let toggle_open: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = !*v);
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let toggle_open_override: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            let _ = host.models_mut().update(&override_open, |v| *v = !*v);
            host.request_redraw(action_cx.window);
        })
    };

    let last_action_for_entries = last_action.clone();
    let dropdown = material3::DropdownMenu::new(open.clone())
        .a11y_label("Material 3 Menu")
        .test_id("ui-gallery-material3-menu")
        .into_element(
            cx,
            move |cx| {
                material3::Button::new("Open menu")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(toggle_open.clone())
                    .test_id("ui-gallery-material3-menu-trigger")
                    .into_element(cx)
            },
            move |_cx| {
                vec![
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Cut")
                            .test_id("ui-gallery-material3-menu-item-cut")
                            .on_select(on_select(
                                "material3.menu.cut",
                                last_action_for_entries.clone(),
                            )),
                    ),
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Copy")
                            .test_id("ui-gallery-material3-menu-item-copy")
                            .on_select(on_select(
                                "material3.menu.copy",
                                last_action_for_entries.clone(),
                            )),
                    ),
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Paste")
                            .test_id("ui-gallery-material3-menu-item-paste")
                            .disabled(true),
                    ),
                    material3::MenuEntry::Separator,
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Settings")
                            .test_id("ui-gallery-material3-menu-item-settings")
                            .on_select(on_select(
                                "material3.menu.settings",
                                last_action_for_entries.clone(),
                            )),
                    ),
                ]
            },
        );

    let theme = cx.theme().clone();
    let override_style = material3::MenuStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.secondary-container"),
        ))))
        .item_label_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.on-secondary-container"),
        ))))
        .item_state_layer_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.on-secondary-container"),
        ))));

    let last_action_for_override_entries = last_action.clone();
    let dropdown_override = material3::DropdownMenu::new(override_open.clone())
        .a11y_label("Material 3 Menu (override)")
        .test_id("ui-gallery-material3-menu-override")
        .menu_style(override_style)
        .into_element(
            cx,
            move |cx| {
                material3::Button::new("Open menu (override)")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(toggle_open_override.clone())
                    .test_id("ui-gallery-material3-menu-trigger-override")
                    .into_element(cx)
            },
            move |_cx| {
                vec![
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Cut")
                            .test_id("ui-gallery-material3-menu-item-cut-override")
                            .on_select(on_select(
                                "material3.menu.cut.override",
                                last_action_for_override_entries.clone(),
                            )),
                    ),
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Copy")
                            .test_id("ui-gallery-material3-menu-item-copy-override")
                            .on_select(on_select(
                                "material3.menu.copy.override",
                                last_action_for_override_entries.clone(),
                            )),
                    ),
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Paste")
                            .test_id("ui-gallery-material3-menu-item-paste-override")
                            .disabled(true),
                    ),
                    material3::MenuEntry::Separator,
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Settings")
                            .test_id("ui-gallery-material3-menu-item-settings-override")
                            .on_select(on_select(
                                "material3.menu.settings.override",
                                last_action_for_override_entries.clone(),
                            )),
                    ),
                ]
            },
        );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let card_default = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Default").into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![dropdown]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let card_override = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Override").into_element(cx),
            shadcn::CardDescription::new(
                "ADR 0220: MenuStyle overrides (container + item colors).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![dropdown_override]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    vec![
        cx.text("Tip: Arrow keys / Home / End navigate; type to jump by prefix; Esc/outside press closes."),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            move |_cx| vec![card_default, card_override],
        ),
        cx.text(format!("last action: {last}")),
    ]
}

pub(in crate::ui) fn preview_material3_list(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let build_list = |cx: &mut ElementContext<'_, App>, id_prefix: &str| -> AnyElement {
        material3::List::new(value.clone())
            .a11y_label("Material 3 List")
            .test_id(format!("{id_prefix}-list"))
            .items(vec![
                material3::ListItem::new("alpha", "Alpha")
                    .leading_icon(ids::ui::SEARCH)
                    .a11y_label("List item alpha")
                    .test_id(format!("{id_prefix}-list-item-alpha")),
                material3::ListItem::new("beta", "Beta")
                    .leading_icon(ids::ui::SETTINGS)
                    .a11y_label("List item beta")
                    .test_id(format!("{id_prefix}-list-item-beta")),
                material3::ListItem::new("disabled", "Disabled")
                    .leading_icon(ids::ui::SLASH)
                    .disabled(true)
                    .a11y_label("List item disabled")
                    .test_id(format!("{id_prefix}-list-item-disabled")),
            ])
            .into_element(cx)
    };

    let standard = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Standard").into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![build_list(cx, "ui-gallery-material3-standard")])
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let expressive = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Expressive").into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![material3::context::with_material_design_variant(
            cx,
            material3::MaterialDesignVariant::Expressive,
            |cx| build_list(cx, "ui-gallery-material3-expressive"),
        )])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let variants = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_stretch(),
        move |_cx| [standard, expressive],
    );

    vec![
        cx.text("Material 3 List: roving focus (Up/Down/Home/End) + selection follows focus."),
        cx.text("Compare Standard vs Expressive via subtree override (shape + icon size)."),
        variants,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

pub(in crate::ui) fn preview_material3_snackbar(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_runtime::CommandId;
    use fret_ui::action::OnActivate;
    use fret_ui_kit::ToastStore;

    #[derive(Default)]
    struct State {
        store: Option<Model<ToastStore>>,
    }

    let store = cx.with_state(State::default, |st| st.store.clone());
    let store = store.unwrap_or_else(|| {
        let store = cx.app.models_mut().insert(ToastStore::default());
        cx.with_state(State::default, |st| st.store = Some(store.clone()));
        store
    });

    let host_layer = material3::SnackbarHost::new(store.clone())
        .max_snackbars(1)
        .into_element(cx);

    let show_short: OnActivate = {
        let store = store.clone();
        Arc::new(move |host, acx, _reason| {
            let controller = material3::SnackbarController::new(store.clone());
            let _ = controller.show(
                host,
                acx.window,
                material3::Snackbar::new("Saved").action("Undo", CommandId::new(CMD_TOAST_ACTION)),
            );
            host.request_redraw(acx.window);
        })
    };

    let show_two_line: OnActivate = {
        let store = store.clone();
        Arc::new(move |host, acx, _reason| {
            let controller = material3::SnackbarController::new(store.clone());
            let _ = controller.show(
                host,
                acx.window,
                material3::Snackbar::new("Update available")
                    .supporting_text("Restart the app to apply the latest changes.")
                    .action("Restart", CommandId::new(CMD_TOAST_ACTION))
                    .duration(material3::SnackbarDuration::Long),
            );
            host.request_redraw(acx.window);
        })
    };

    let show_indefinite: OnActivate = {
        let store = store.clone();
        Arc::new(move |host, acx, _reason| {
            let controller = material3::SnackbarController::new(store.clone());
            let _ = controller.show(
                host,
                acx.window,
                material3::Snackbar::new("Connection lost")
                    .supporting_text("Trying to reconnect...")
                    .duration(material3::SnackbarDuration::Indefinite),
            );
            host.request_redraw(acx.window);
        })
    };

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let buttons = stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                material3::Button::new("Show (short)")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(show_short.clone())
                    .test_id("ui-gallery-material3-snackbar-show-short")
                    .into_element(cx),
                material3::Button::new("Show (two-line)")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(show_two_line.clone())
                    .test_id("ui-gallery-material3-snackbar-show-two-line")
                    .into_element(cx),
                material3::Button::new("Show (indefinite)")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(show_indefinite.clone())
                    .test_id("ui-gallery-material3-snackbar-show-indefinite")
                    .into_element(cx),
            ]
        },
    );

    let card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Snackbar").into_element(cx),
            shadcn::CardDescription::new(
                "Snackbar MVP: Material token-driven toast-layer skin (md.comp.snackbar.*).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![
            host_layer,
            buttons,
            cx.text(format!("last action: {last}")),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    vec![card]
}

pub(in crate::ui) fn preview_material3_tooltip(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    let content = material3::TooltipProvider::new().with_elements(cx, |cx| {
        let outlined = material3::ButtonVariant::Outlined;

        let top = material3::PlainTooltip::new(
            material3::Button::new("Hover (Top)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-top-trigger")
                .into_element(cx),
            "Plain tooltip (top)",
        )
        .side(material3::TooltipSide::Top)
        .into_element(cx);

        let right = material3::PlainTooltip::new(
            material3::Button::new("Hover (Right)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-right-trigger")
                .into_element(cx),
            "Plain tooltip (right)",
        )
        .side(material3::TooltipSide::Right)
        .into_element(cx);

        let bottom = material3::PlainTooltip::new(
            material3::Button::new("Hover (Bottom)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-bottom-trigger")
                .into_element(cx),
            "Plain tooltip (bottom)",
        )
        .side(material3::TooltipSide::Bottom)
        .into_element(cx);

        let left = material3::PlainTooltip::new(
            material3::Button::new("Hover (Left)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-left-trigger")
                .into_element(cx),
            "Plain tooltip (left)",
        )
        .side(material3::TooltipSide::Left)
        .into_element(cx);

        let rich = material3::RichTooltip::new(
            material3::Button::new("Hover (Rich)")
                .variant(outlined)
                .test_id("ui-gallery-material3-rich-tooltip-trigger")
                .into_element(cx),
            "Rich tooltip supporting text (body medium).",
        )
        .title("Rich tooltip title")
        .side(material3::TooltipSide::Top)
        .into_element(cx);

        let rich_no_title = material3::RichTooltip::new(
            material3::Button::new("Hover (Rich / no title)")
                .variant(outlined)
                .test_id("ui-gallery-material3-rich-tooltip-no-title-trigger")
                .into_element(cx),
            "Rich tooltip supporting text only.",
        )
        .side(material3::TooltipSide::Bottom)
        .into_element(cx);

        vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N4)
                        .layout(LayoutRefinement::default().w_full()),
                    |_cx| [top, right, bottom, left],
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N4)
                        .layout(LayoutRefinement::default().w_full()),
                    |_cx| [rich, rich_no_title],
                ),
                cx.text("Note: Tooltip open delay is controlled via Material3 TooltipProvider (delay-group)."),
            ]
    });

    let card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Tooltip").into_element(cx),
            shadcn::CardDescription::new(
                "Tooltip MVP: delay group + hover intent + safe-hover corridor + token-driven styling (plain + rich).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(content).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    vec![card]
}
