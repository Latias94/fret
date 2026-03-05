pub const SOURCE: &str = include_str!("expand_at_bottom.rs");

// region: example
use fret_core::{Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::SemanticsDecoration;
use fret_ui::element::SemanticsProps;
use fret_ui::scroll::ScrollHandle;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    expanded: Option<Model<bool>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.named("ui-gallery.scroll_area.expand_at_bottom", |cx| {
        let scroll_handle = cx.with_state(ScrollHandle::default, |h| h.clone());

        let expanded = cx.with_state(DemoModels::default, |st| st.expanded.clone());
        let expanded = match expanded {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(false);
                cx.with_state(DemoModels::default, |st| st.expanded = Some(model.clone()));
                model
            }
        };

        cx.semantics_with_id(
            SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("ui-gallery-scroll-area-expand-at-bottom-harness")),
                ..Default::default()
            },
            move |cx, _id| {
                let is_expanded = cx
                    .get_model_copied(&expanded, Invalidation::Paint)
                    .unwrap_or(false);

                let toggle = {
                    let expanded = expanded.clone();
                    shadcn::Button::new(if is_expanded {
                        "Collapse"
                    } else {
                        "Expand at bottom"
                    })
                    .variant(shadcn::ButtonVariant::Secondary)
                    .on_activate(Arc::new(move |host, action_cx, _reason| {
                        let _ = host.models_mut().update(&expanded, |v| *v = !*v).ok();
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                    }))
                    .test_id("ui-gallery-scroll-area-expand-at-bottom-toggle")
                    .into_element(cx)
                };

                // Deterministic sizing for diagnostics:
                // - viewport height: 200px
                // - baseline content height: 200px (10 * 20px), so y_max = 0px
                // - expanded content adds 100 rows (2000px), so y_max becomes > 0
                let row_h = Px(20.0);
                let baseline_rows = 10usize;
                let expanded_rows = 100usize;

                let mut row_layout = fret_ui::element::LayoutStyle::default();
                row_layout.size.width = fret_ui::element::Length::Fill;
                row_layout.size.height = fret_ui::element::Length::Px(row_h);
                let row_props = fret_ui::element::ContainerProps {
                    layout: row_layout,
                    ..Default::default()
                };

                let leaf = cx.column(
                    fret_ui::element::ColumnProps {
                        gap: Px(0.0).into(),
                        ..Default::default()
                    },
                    move |cx| {
                        let mut rows: Vec<AnyElement> = Vec::new();
                        for _ in 0..baseline_rows {
                            rows.push(cx.container(row_props, |_cx| Vec::new()));
                        }
                        if is_expanded {
                            for _ in 0..expanded_rows {
                                rows.push(cx.container(row_props, |_cx| Vec::new()));
                            }
                        }
                        rows
                    },
                );

                // Wrapper-heavy subtree to exercise the post-layout extents observation budget
                // fallback path (wrapper peel budget is 8).
                let wrapper_chain_len: usize = 9;
                let mut wrapper_layout = fret_ui::element::LayoutStyle::default();
                wrapper_layout.size.width = fret_ui::element::Length::Fill;
                wrapper_layout.size.height = fret_ui::element::Length::Fill;
                wrapper_layout.overflow = fret_ui::element::Overflow::Clip;
                let wrapper_props = fret_ui::element::ContainerProps {
                    layout: wrapper_layout,
                    ..Default::default()
                };

                let mut wrapped = leaf;
                for _ in 0..wrapper_chain_len {
                    let child = wrapped;
                    wrapped = cx.container(wrapper_props, move |_cx| vec![child]);
                }

                let mut scroll_layout = fret_ui::element::LayoutStyle::default();
                scroll_layout.size.width = fret_ui::element::Length::Px(Px(240.0));
                scroll_layout.size.height = fret_ui::element::Length::Px(Px(200.0));
                scroll_layout.overflow = fret_ui::element::Overflow::Clip;

                let scroll = cx
                    .scroll(
                        fret_ui::element::ScrollProps {
                            layout: scroll_layout,
                            axis: fret_ui::element::ScrollAxis::Y,
                            scroll_handle: Some(scroll_handle.clone()),
                            ..Default::default()
                        },
                        |_cx| vec![wrapped],
                    )
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .test_id("ui-gallery-scroll-area-expand-at-bottom-viewport"),
                    );

                let mut scrollbar_fill = fret_ui::element::LayoutStyle::default();
                scrollbar_fill.size.width = fret_ui::element::Length::Fill;
                scrollbar_fill.size.height = fret_ui::element::Length::Fill;
                let scrollbar = cx.scrollbar(fret_ui::element::ScrollbarProps {
                    layout: scrollbar_fill,
                    axis: fret_ui::element::ScrollbarAxis::Vertical,
                    scroll_target: Some(scroll.id),
                    scroll_handle: scroll_handle.clone(),
                    style: fret_ui::element::ScrollbarStyle::default(),
                });

                let mut stack_layout = fret_ui::element::LayoutStyle::default();
                stack_layout.size.width = fret_ui::element::Length::Px(Px(240.0));
                stack_layout.size.height = fret_ui::element::Length::Px(Px(200.0));

                let stack = cx.stack_props(
                    fret_ui::element::StackProps {
                        layout: stack_layout,
                    },
                    |_cx| vec![scroll, scrollbar],
                );

                vec![
                    ui::v_flex(|_cx| [toggle, stack])
                        .gap(Space::N2)
                        .into_element(cx)
                        .test_id("ui-gallery-scroll-area-expand-at-bottom-root"),
                ]
            },
        )
    })
}
// endregion: example
