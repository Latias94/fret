use super::super::*;

use std::sync::Arc;

use crate::ui::doc_layout::{self, DocSection};

use fret_ui::element::{CrossAlign, FlexProps, MainAlign};

pub(super) fn preview_carousel(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.carousel_page", |cx| {
        #[derive(Default)]
        struct CarouselPageState {
            demo_inner_clicked: Option<Model<bool>>,
            expandable_selected: Option<Model<Option<usize>>>,
        }

    #[derive(Debug, Clone, Copy)]
    struct SlideVisual {
        text_px: Px,
        line_height_px: Px,
    }

    let slide = |cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual| {
        let theme = Theme::global(&*cx.app).clone();

        let number = ui::text(cx, format!("{idx}"))
            .text_size_px(visual.text_px)
            .line_height_px(visual.line_height_px)
            .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
            .font_semibold()
            .into_element(cx);

        let content = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default().w_full().aspect_ratio(1.0),
                ),
                direction: fret_core::Axis::Horizontal,
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                padding: Edges::all(Px(24.0)).into(),
                ..Default::default()
            },
            move |_cx| vec![number],
        );

        let card = shadcn::Card::new([content]).into_element(cx);
        ui::container(cx, move |_cx| vec![card])
            .p_1()
            .into_element(cx)
    };

    let max_w_sm = Px(384.0);

    // Demo: include a descendant pressable so diag scripts can gate pointer propagation
    // (drag-from-descendant should not activate; click should).
    let demo_inner_clicked =
        cx.with_state(CarouselPageState::default, |st| st.demo_inner_clicked.clone());
    let demo_inner_clicked = match demo_inner_clicked {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(CarouselPageState::default, |st| {
                st.demo_inner_clicked = Some(model.clone());
            });
            model
        }
    };
    let demo_inner_clicked_now = cx
        .watch_model(&demo_inner_clicked)
        .copied()
        .unwrap_or(false);
    let toggle_demo_inner_clicked = {
        let demo_inner_clicked = demo_inner_clicked.clone();
        Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  _reason: fret_ui::action::ActivateReason| {
                let _ = host
                    .models_mut()
                    .update(&demo_inner_clicked, |v| *v = !*v);
                host.request_redraw(action_cx.window);
            },
        ) as fret_ui::action::OnActivate
    };

    let demo_slide = |cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual| {
        let theme = Theme::global(&*cx.app).clone();

        let number = ui::text(cx, format!("{idx}"))
            .text_size_px(visual.text_px)
            .line_height_px(visual.line_height_px)
            .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
            .font_semibold()
            .into_element(cx);

        let mut children: Vec<AnyElement> = vec![number];

        if idx == 1 {
            children.push(
                shadcn::Button::new("Inner button")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(toggle_demo_inner_clicked.clone())
                    .test_id("ui-gallery-carousel-demo-inner-button")
                    .into_element(cx),
            );

            if demo_inner_clicked_now {
                children.push(
                    ui::container(cx, move |cx| {
                        vec![ui::text(cx, "clicked").text_sm().into_element(cx)]
                    })
                    .into_element(cx)
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .role(fret_core::SemanticsRole::Group)
                            .test_id("ui-gallery-carousel-demo-inner-clicked"),
                    ),
                );
            }
        }

        let gap = decl_style::space(&theme, Space::N3);
        let content = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default().w_full().aspect_ratio(1.0),
                ),
                direction: fret_core::Axis::Vertical,
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                gap: gap.into(),
                padding: Edges::all(Px(24.0)).into(),
                ..Default::default()
            },
            move |_cx| children,
        );

        let card = shadcn::Card::new([content]).into_element(cx);
        ui::container(cx, move |_cx| vec![card])
            .p_1()
            .into_element(cx)
    };

    let demo_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(44.0),
    };
    let demo_items = (1..=5)
        .map(|idx| demo_slide(cx, idx, demo_visual))
        .collect::<Vec<_>>();
    let demo = shadcn::Carousel::new(demo_items)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_sm)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-demo")
        .into_element(cx);

    let basic_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(44.0),
    };
    let basic_items = (1..=5)
        .map(|idx| slide(cx, idx, basic_visual))
        .collect::<Vec<_>>();
    let basic = shadcn::Carousel::new(basic_items)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_sm)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-basic")
        .into_element(cx);

    let align_start_visual = SlideVisual {
        text_px: Px(30.0),
        line_height_px: Px(36.0),
    };
    let align_start_items = (1..=5)
        .map(|idx| slide(cx, idx, align_start_visual))
        .collect::<Vec<_>>();
    let align_start = shadcn::Carousel::new(align_start_items)
        .item_basis_main_px(Px(192.0))
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_sm)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-align-start")
        .into_element(cx);

    let spacing_visual = SlideVisual {
        text_px: Px(24.0),
        line_height_px: Px(32.0),
    };
    let spacing_items = (1..=5)
        .map(|idx| slide(cx, idx, spacing_visual))
        .collect::<Vec<_>>();
    let spacing = shadcn::Carousel::new(spacing_items)
        .item_basis_main_px(Px(192.0))
        .track_start_neg_margin(Space::N1)
        .item_padding_start(Space::N1)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_sm)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-spacing")
        .into_element(cx);

    let notes_stack = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Carousel demo: Basic, Align Start, and Spacing.",
            "The upstream demo uses responsive item widths (`md:basis-1/2` / `lg:basis-1/3`). Fret uses a fixed `item_basis_main_px` to keep geometry deterministic in native builds.",
            "Spacing parity depends on pairing `track_start_neg_margin` with `item_padding_start`.",
        ],
    );

    // Expandable: used by the motion pilot suite to exercise content-driven resizing while the
    // carousel remains interactive.
    let expandable_selected =
        cx.with_state(CarouselPageState::default, |st| st.expandable_selected.clone());
    let expandable_selected = match expandable_selected {
        Some(model) => model,
        None => {
            let model: Model<Option<usize>> = cx.app.models_mut().insert(None);
            cx.with_state(CarouselPageState::default, |st| {
                st.expandable_selected = Some(model.clone());
            });
            model
        }
    };
    let expandable_selected_now = cx
        .watch_model(&expandable_selected)
        .copied()
        .unwrap_or(None);

    let set_expandable_selected = |next: Option<usize>| {
        let expandable_selected = expandable_selected.clone();
        Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  _reason: fret_ui::action::ActivateReason| {
                let next = next;
                let _ = host
                    .models_mut()
                    .update(&expandable_selected, |cur| *cur = next);
                host.request_redraw(action_cx.window);
            },
        ) as fret_ui::action::OnActivate
    };

    let expandable_items = (1..=5)
        .map(|idx| {
            let expanded = expandable_selected_now == Some(idx);
            let height = if expanded { Px(260.0) } else { Px(140.0) };

            let theme = Theme::global(&*cx.app).clone();
            let gap = decl_style::space(&theme, Space::N2);

            let body = cx.flex(
                FlexProps {
                    layout: decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default().w_full().h_px(height),
                    ),
                    direction: fret_core::Axis::Vertical,
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    gap: gap.into(),
                    padding: Edges::all(Px(24.0)).into(),
                    ..Default::default()
                },
                move |cx| {
                    let mut out = vec![
                        ui::text(cx, format!("Item {idx}"))
                            .text_base()
                            .font_semibold()
                            .into_element(cx),
                        shadcn::Button::new(if expanded { "Collapse" } else { "Expand" })
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .on_activate(set_expandable_selected(Some(idx)))
                            .into_element(cx),
                    ];

                    if expanded {
                        out.push(ui::text(cx, "Expandable body").text_sm().into_element(cx));
                    }

                    out
                },
            );

            shadcn::Card::new([body]).into_element(cx)
        })
        .collect::<Vec<_>>();

    let expandable = shadcn::Carousel::new(expandable_items)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_sm)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-expandable")
        .into_element(cx);

    // Orientation (vertical): aligns with upstream docs, and is used by the existing screenshot
    // diag script.
    let vertical_items = (1..=5)
        .map(|idx| {
            let theme = Theme::global(&*cx.app).clone();
            let body = cx.flex(
                FlexProps {
                    layout: decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default().w_full().h_full(),
                    ),
                    direction: fret_core::Axis::Vertical,
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    padding: Edges::all(Px(24.0)).into(),
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        ui::text(cx, format!("{idx}"))
                            .text_base()
                            .font_semibold()
                            .into_element(cx),
                    ]
                },
            );
            shadcn::Card::new([body]).into_element(cx)
        })
        .collect::<Vec<_>>();

    let orientation_vertical = shadcn::Carousel::new(vertical_items)
        .orientation(shadcn::CarouselOrientation::Vertical)
        .refine_viewport_layout(LayoutRefinement::default().h_px(Px(200.0)))
        .track_start_neg_margin(Space::N1)
        .item_padding_start(Space::N1)
        .item_basis_main_px(Px(100.0))
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_sm)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-orientation-vertical")
        .into_element(cx);

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Carousel demo cards (Fret builder API; not Embla)."),
        vec![
            DocSection::new("Demo", demo)
                .description("A carousel with 5 items and previous/next buttons.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-demo"),
            DocSection::new("Basic", basic)
                .description("Default slide width (basis-full).")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-basic")
                .code(
                    "rust",
                    r#"let items = (1..=5).map(|idx| slide(cx, idx)).collect::<Vec<_>>();

shadcn::Carousel::new(items)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)).mx_auto())
    .into_element(cx);"#,
                ),
            DocSection::new("Align Start", align_start)
                .description("Fixed basis to approximate the upstream responsive layout.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-align-start")
                .code(
                    "rust",
                    r#"// Upstream: responsive widths (`md:basis-1/2` / `lg:basis-1/3`).
// Here: fixed basis for deterministic native layout.
shadcn::Carousel::new(items)
    .item_basis_main_px(Px(192.0))
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)).mx_auto())
    .into_element(cx);"#,
                ),
            DocSection::new("Spacing", spacing)
                .description(
                    "Tighter track negative margin + item start padding (shadcn `-ml-1` / `pl-1`).",
                )
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-spacing")
                .code(
                    "rust",
                    r#"shadcn::Carousel::new(items)
    .item_basis_main_px(Px(192.0))
    .track_start_neg_margin(Space::N1)
    .item_padding_start(Space::N1)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)).mx_auto())
    .into_element(cx);"#,
                ),
            DocSection::new("Expandable", expandable)
                .description("Content-driven height changes (used by the motion pilot suite).")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-expandable"),
            DocSection::new("Orientation (Vertical)", orientation_vertical)
                .description("A vertical carousel (orientation=\"vertical\").")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-orientation-vertical"),
            DocSection::new("Notes", notes_stack).max_w(Px(760.0)),
        ],
    );

    vec![body.test_id("ui-gallery-carousel-component")]
    })
}
