use super::super::super::super::*;

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
