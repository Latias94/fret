use super::super::super::super::*;

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
