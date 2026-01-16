use std::sync::Arc;

use fret_core::{Color, Corners, Edges, InternalDragKind, Px};
use fret_runtime::Model;
use fret_ui::action::{OnInternalDrag, OnPointerDown};
use fret_ui::element::{
    AnyElement, ContainerProps, InternalDragRegionProps, LayoutStyle, Length, PointerRegionProps,
    ResizablePanelGroupProps, ViewCacheProps,
};
use fret_ui::{ElementContext, Invalidation, ResizablePanelGroupStyle, Theme, UiHost};

use crate::commands::{pane_activate_command, pane_move_active_tab_to_command};
use crate::layout::{WorkspacePaneLayout, WorkspacePaneTree, WorkspaceWindowLayout};
use crate::tab_drag::{DRAG_KIND_WORKSPACE_TAB, WorkspaceTabDragState};

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn split_container_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn pane_container_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn pane_border_color(theme: &Theme, is_active: bool) -> Option<Color> {
    if is_active {
        theme
            .color_by_key("workspace.pane.active_border")
            .or_else(|| theme.color_by_key("ring"))
            .or_else(|| theme.color_by_key("border"))
    } else {
        theme.color_by_key("border")
    }
}

fn pane_border_width(is_active: bool) -> Edges {
    let _ = is_active;
    Edges::all(Px(1.0))
}

fn pane_corner_radius(theme: &Theme) -> Corners {
    let r = theme
        .metric_by_key("workspace.pane.radius")
        .unwrap_or(Px(0.0));
    Corners::all(Px(r.0.max(0.0)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SplitChild {
    A,
    B,
}

fn split_key(path: &[SplitChild]) -> u64 {
    let mut bits: u64 = 1;
    for step in path {
        bits = bits.wrapping_shl(1)
            | match step {
                SplitChild::A => 0,
                SplitChild::B => 1,
            };
    }
    bits
}

fn clamp_fraction(fraction: f32) -> f32 {
    fraction.clamp(0.05, 0.95)
}

fn set_split_fraction(tree: &mut WorkspacePaneTree, path: &[SplitChild], fraction: f32) -> bool {
    let fraction = clamp_fraction(fraction);
    if path.is_empty() {
        let WorkspacePaneTree::Split {
            axis: _,
            fraction: f,
            a: _,
            b: _,
        } = tree
        else {
            return false;
        };

        if (*f - fraction).abs() <= 0.0001 {
            return false;
        }

        *f = fraction;
        return true;
    }

    let WorkspacePaneTree::Split {
        axis: _,
        fraction: _,
        a,
        b,
    } = tree
    else {
        return false;
    };

    match path[0] {
        SplitChild::A => set_split_fraction(a.as_mut(), &path[1..], fraction),
        SplitChild::B => set_split_fraction(b.as_mut(), &path[1..], fraction),
    }
}

#[derive(Debug, Default)]
struct SplitResizeModelState {
    fractions: Option<Model<Vec<f32>>>,
    last_model_fraction: Option<f32>,
    last_window_fraction: Option<f32>,
}

#[derive(Debug, Default)]
struct WorkspaceTabDragModelState {
    model: Option<Model<WorkspaceTabDragState>>,
}

fn get_tab_drag_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<WorkspaceTabDragState> {
    let existing = cx.with_state(WorkspaceTabDragModelState::default, |st| st.model.clone());
    if let Some(m) = existing {
        return m;
    }

    let model = cx.app.models_mut().insert(WorkspaceTabDragState::default());
    cx.with_state(WorkspaceTabDragModelState::default, |st| {
        st.model = Some(model.clone());
    });
    model
}

pub fn workspace_pane_tree_element_with_resize<H: UiHost, F>(
    cx: &mut ElementContext<'_, H>,
    window: Model<WorkspaceWindowLayout>,
    render_pane: &mut F,
) -> AnyElement
where
    F: FnMut(
        &mut ElementContext<'_, H>,
        &WorkspacePaneLayout,
        bool,
        Model<WorkspaceTabDragState>,
    ) -> AnyElement,
{
    let tab_drag = get_tab_drag_model(cx);

    let Some(window_snapshot) = cx.get_model_cloned(&window, Invalidation::Layout) else {
        return cx.container(
            ContainerProps {
                layout: fill_layout(),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );
    };

    let active_pane = window_snapshot.active_pane_id().map(|id| id.as_ref());
    render_node_with_resize(
        cx,
        &window,
        &window_snapshot.pane_tree,
        &[],
        active_pane,
        &tab_drag,
        render_pane,
    )
}

fn render_node_with_resize<H: UiHost, F>(
    cx: &mut ElementContext<'_, H>,
    window: &Model<WorkspaceWindowLayout>,
    node: &WorkspacePaneTree,
    split_path: &[SplitChild],
    active_pane: Option<&str>,
    tab_drag: &Model<WorkspaceTabDragState>,
    render_pane: &mut F,
) -> AnyElement
where
    F: FnMut(
        &mut ElementContext<'_, H>,
        &WorkspacePaneLayout,
        bool,
        Model<WorkspaceTabDragState>,
    ) -> AnyElement,
{
    match node {
        WorkspacePaneTree::Leaf(pane) => render_leaf(cx, pane, active_pane, tab_drag, render_pane),
        WorkspacePaneTree::Split {
            axis,
            fraction,
            a,
            b,
        } => {
            let axis = *axis;
            let fraction = *fraction;

            let mut path_a = split_path.to_vec();
            path_a.push(SplitChild::A);
            let mut path_b = split_path.to_vec();
            path_b.push(SplitChild::B);

            let key = split_key(split_path);
            let window = window.clone();
            let a = a.clone();
            let b = b.clone();
            let tab_drag = tab_drag.clone();

            cx.keyed(key, |cx| {
                let chrome = {
                    let theme = Theme::global(cx.app);
                    ResizablePanelGroupStyle::from_theme(theme)
                };

                let (fractions_model, last_model_fraction, last_window_fraction) = {
                    let (model, last_model_fraction, last_window_fraction) =
                        cx.with_state(SplitResizeModelState::default, |state| {
                            (
                                state.fractions.clone(),
                                state.last_model_fraction,
                                state.last_window_fraction,
                            )
                        });

                    let model = match model {
                        Some(model) => model,
                        None => {
                            let model = cx.app.models_mut().insert(vec![fraction, 1.0 - fraction]);
                            cx.with_state(SplitResizeModelState::default, |state| {
                                state.fractions = Some(model.clone());
                            });
                            model
                        }
                    };

                    (model, last_model_fraction, last_window_fraction)
                };

                let fractions_now = cx
                    .get_model_cloned(&fractions_model, Invalidation::Layout)
                    .unwrap_or_else(|| vec![fraction, 1.0 - fraction]);
                let sum = fractions_now.iter().sum::<f32>().max(0.0001);
                let model_fraction =
                    (fractions_now.get(0).copied().unwrap_or(0.5) / sum).clamp(0.0, 1.0);

                let model_changed =
                    last_model_fraction.is_none_or(|last| (last - model_fraction).abs() > 0.0001);
                let window_changed =
                    last_window_fraction.is_none_or(|last| (last - fraction).abs() > 0.0001);

                let mut next_window_fraction = fraction;
                let mut next_model_fraction = model_fraction;

                if window_changed && !model_changed {
                    let _ = cx.app.models_mut().update(&fractions_model, |v| {
                        v.clear();
                        v.push(fraction);
                        v.push(1.0 - fraction);
                    });
                    next_model_fraction = fraction;
                } else if model_changed
                    && !window_changed
                    && (model_fraction - fraction).abs() > 0.0001
                {
                    let split_path = split_path.to_vec();
                    let _ = cx.app.models_mut().update(&window, |w| {
                        if set_split_fraction(&mut w.pane_tree, &split_path, model_fraction) {
                            next_window_fraction = model_fraction;
                        }
                    });
                }

                cx.with_state(SplitResizeModelState::default, |state| {
                    state.last_model_fraction = Some(next_model_fraction);
                    state.last_window_fraction = Some(next_window_fraction);
                });

                cx.resizable_panel_group(
                    ResizablePanelGroupProps {
                        layout: split_container_layout(),
                        axis,
                        model: fractions_model,
                        min_px: vec![Px(120.0), Px(120.0)],
                        enabled: true,
                        chrome,
                    },
                    |cx| {
                        vec![
                            render_node_with_resize(
                                cx,
                                &window,
                                a.as_ref(),
                                &path_a,
                                active_pane,
                                &tab_drag,
                                render_pane,
                            ),
                            render_node_with_resize(
                                cx,
                                &window,
                                b.as_ref(),
                                &path_b,
                                active_pane,
                                &tab_drag,
                                render_pane,
                            ),
                        ]
                    },
                )
            })
        }
    }
}

fn render_leaf<H: UiHost, F>(
    cx: &mut ElementContext<'_, H>,
    pane: &WorkspacePaneLayout,
    active_pane: Option<&str>,
    tab_drag: &Model<WorkspaceTabDragState>,
    render_pane: &mut F,
) -> AnyElement
where
    F: FnMut(
        &mut ElementContext<'_, H>,
        &WorkspacePaneLayout,
        bool,
        Model<WorkspaceTabDragState>,
    ) -> AnyElement,
{
    let is_active = active_pane.is_some_and(|id| pane.id.as_ref() == id);

    let is_drop_target = cx
        .get_model_cloned(tab_drag, Invalidation::Paint)
        .is_some_and(|st| {
            st.pointer.is_some()
                && st
                    .hovered_pane
                    .as_deref()
                    .is_some_and(|p| p == pane.id.as_ref())
                && st
                    .source_pane
                    .as_deref()
                    .is_some_and(|p| p != pane.id.as_ref())
        });

    let (border, border_color, corner_radii, background) = {
        let theme = Theme::global(cx.app);
        let background = theme.color_by_key("workspace.pane.bg");
        (
            if is_drop_target {
                Edges::all(Px(2.0))
            } else {
                pane_border_width(is_active)
            },
            if is_drop_target {
                theme
                    .color_by_key("ring")
                    .or_else(|| theme.color_by_key("accent"))
                    .or_else(|| pane_border_color(theme, is_active))
            } else {
                pane_border_color(theme, is_active)
            },
            pane_corner_radius(theme),
            background,
        )
    };

    let pane_id = pane.id.clone();
    let activate_cmd = pane_activate_command(pane_id.as_ref());
    let move_tab_cmd = pane_move_active_tab_to_command(pane_id.as_ref());

    let tab_drag_model = tab_drag.clone();
    let on_internal_drag: OnInternalDrag = {
        let pane_id = pane_id.clone();
        Arc::new(move |host, acx, drag| {
            let Some(session) = host.drag(drag.pointer_id) else {
                return false;
            };
            if session.kind != DRAG_KIND_WORKSPACE_TAB || !session.dragging {
                return false;
            }

            match drag.kind {
                InternalDragKind::Enter | InternalDragKind::Over => {
                    let mut handled = false;
                    let _ = host.models_mut().update(&tab_drag_model, |st| {
                        if st.pointer != Some(drag.pointer_id) {
                            return;
                        }
                        if st.hovered_pane.as_deref() != Some(pane_id.as_ref()) {
                            st.hovered_pane = Some(pane_id.clone());
                            handled = true;
                        }
                    });
                    if handled {
                        host.request_redraw(acx.window);
                    }
                    handled
                }
                InternalDragKind::Leave => {
                    let mut cleared = false;
                    let _ = host.models_mut().update(&tab_drag_model, |st| {
                        if st.pointer != Some(drag.pointer_id) {
                            return;
                        }
                        if st.hovered_pane.as_deref() == Some(pane_id.as_ref()) {
                            st.hovered_pane = None;
                            cleared = true;
                        }
                    });
                    if cleared {
                        host.request_redraw(acx.window);
                    }
                    cleared
                }
                InternalDragKind::Cancel => {
                    let mut did_clear = false;
                    let _ = host.models_mut().update(&tab_drag_model, |st| {
                        if st.pointer != Some(drag.pointer_id) {
                            return;
                        }
                        *st = WorkspaceTabDragState::default();
                        did_clear = true;
                    });
                    if did_clear {
                        host.request_redraw(acx.window);
                    }
                    did_clear
                }
                InternalDragKind::Drop => {
                    let mut should_move = false;
                    let _ = host.models_mut().update(&tab_drag_model, |st| {
                        if st.pointer != Some(drag.pointer_id) {
                            return;
                        }
                        should_move = st
                            .source_pane
                            .as_deref()
                            .is_some_and(|p| p != pane_id.as_ref())
                            && st.hovered_pane.as_deref() == Some(pane_id.as_ref());
                        *st = WorkspaceTabDragState::default();
                    });

                    if should_move {
                        if let Some(cmd) = pane_activate_command(pane_id.as_ref()) {
                            host.dispatch_command(Some(acx.window), cmd);
                        }
                        if let Some(cmd) = move_tab_cmd.clone() {
                            host.dispatch_command(Some(acx.window), cmd);
                        }
                        host.request_redraw(acx.window);
                    }

                    should_move
                }
            }
        })
    };

    cx.pointer_region(
        PointerRegionProps {
            layout: pane_container_layout(),
            enabled: true,
        },
        |cx| {
            if let Some(cmd) = activate_cmd {
                let cmd = cmd.clone();
                let handler: OnPointerDown = Arc::new(move |host, acx, _down| {
                    host.dispatch_command(Some(acx.window), cmd.clone());
                    false
                });
                cx.pointer_region_add_on_pointer_down(handler);
            }

            let inner = cx.container(
                ContainerProps {
                    layout: fill_layout(),
                    background,
                    border,
                    border_color,
                    corner_radii,
                    ..Default::default()
                },
                |cx| {
                    vec![cx.view_cache(
                        ViewCacheProps {
                            layout: fill_layout(),
                            contained_layout: true,
                        },
                        |cx| vec![render_pane(cx, pane, is_active, tab_drag.clone())],
                    )]
                },
            );

            vec![cx.internal_drag_region(
                InternalDragRegionProps {
                    layout: fill_layout(),
                    enabled: true,
                },
                |cx| {
                    cx.internal_drag_region_on_internal_drag(on_internal_drag.clone());
                    vec![inner]
                },
            )]
        },
    )
}
