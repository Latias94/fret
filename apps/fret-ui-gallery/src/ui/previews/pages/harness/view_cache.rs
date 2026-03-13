use super::super::super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use fret::UiCx;

pub(in crate::ui) fn preview_view_cache(
    cx: &mut UiCx<'_>,
    _theme: &Theme,
    view_cache_enabled: Model<bool>,
    view_cache_cache_shell: Model<bool>,
    view_cache_cache_content: Model<bool>,
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
    let cache_content = cx
        .get_model_copied(&view_cache_cache_content, Invalidation::Layout)
        .unwrap_or(true);
    let cache_inner = cx
        .get_model_copied(&view_cache_inner_enabled, Invalidation::Layout)
        .unwrap_or(true);
    let continuous = cx
        .get_model_copied(&view_cache_continuous, Invalidation::Layout)
        .unwrap_or(false);

    let toggles = ui::v_stack(|cx| {
        vec![
            ui::h_row(|cx| {
                vec![
                    shadcn::Switch::new(view_cache_enabled.clone())
                        .a11y_label("Enable view-cache mode")
                        .test_id("ui-gallery-view-cache-enabled")
                        .into_element(cx),
                    cx.text("Enable view-cache mode (global UiTree flag)"),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
            ui::h_row(|cx| {
                vec![
                    shadcn::Switch::new(view_cache_cache_shell.clone())
                        .a11y_label("Cache the gallery shell")
                        .test_id("ui-gallery-view-cache-cache-shell")
                        .into_element(cx),
                    cx.text("Cache shell (sidebar/content wrappers)"),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
            ui::h_row(|cx| {
                vec![
                    shadcn::Switch::new(view_cache_cache_content.clone())
                        .a11y_label("Cache the gallery content root")
                        .test_id("ui-gallery-view-cache-cache-content")
                        .into_element(cx),
                    cx.text("Cache content root (requires 'Cache shell')"),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
            ui::h_row(|cx| {
                vec![
                    shadcn::Switch::new(view_cache_inner_enabled.clone())
                        .a11y_label("Enable inner ViewCache boundary")
                        .test_id("ui-gallery-view-cache-inner-cache")
                        .into_element(cx),
                    cx.text("Enable inner ViewCache boundary (torture subtree)"),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
            ui::h_row(|cx| {
                vec![
                    shadcn::Switch::new(view_cache_continuous.clone())
                        .a11y_label("Request continuous frames")
                        .test_id("ui-gallery-view-cache-continuous")
                        .into_element(cx),
                    cx.text("Continuous frames (cache-hit should still keep state alive)"),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .into_element(cx);

    let actions = ui::h_row(|cx| {
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
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let header = ui::v_flex(|cx| {
            vec![
                cx.text("Goal: validate cached-subtree correctness under real interaction."),
                cx.text(format!(
                    "Current settings: view_cache={} shell_cache={} content_cache={} inner_cache={} continuous={}",
                    enabled as u8,
                    cache_shell as u8,
                    cache_content as u8,
                    cache_inner as u8,
                    continuous as u8
                )),
                toggles,
                actions,
            ]
        })
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3).into_element(cx);

    let subtree_body = |cx: &mut UiCx<'_>| -> Vec<AnyElement> {
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

        let popover = shadcn::Popover::from_open(view_cache_popover_open.clone())
            .auto_focus(true)
            .into_element_with(
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

        let list = shadcn::ScrollArea::new([ui::v_flex(|_cx| rows)
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N1)
            .into_element(cx)])
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

    let root = ui::v_flex(move |cx| {
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
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N3)
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Generic)
            .test_id("ui-gallery-view-cache-root"),
    );

    let page = doc_layout::render_doc_page(
        cx,
        Some("Compare cached vs uncached subtree execution and state retention."),
        vec![DocSection::new("Harness", root)
            .no_shell()
            .max_w(Px(980.0))
            .description("Tip: keep 'Cache shell' off while iterating so the status bar updates every frame.")],
    );

    vec![page]
}
