use std::cell::Cell;
use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, InternalDragKind, Point, Px};
use fret_runtime::Model;
use fret_ui::action::{OnInternalDrag, OnPointerDown};
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, InsetStyle, InternalDragRegionProps, LayoutStyle,
    Length, PointerRegionProps, PositionStyle, ResizablePanelGroupProps, ViewCacheProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Invalidation, ResizablePanelGroupStyle, Theme, UiHost};

use fret_dnd::{EdgeDropZone, compute_edge_drop_zone};

use crate::commands::{
    pane_activate_command, pane_move_active_tab_to_command, pane_split_command,
    tab_activate_command, tab_move_active_after_command, tab_move_active_before_command,
};
use crate::layout::{WorkspacePaneLayout, WorkspacePaneTree, WorkspaceWindowLayout};
use crate::tab_drag::{
    DRAG_KIND_WORKSPACE_TAB, WorkspacePaneDragGeometry, WorkspaceTabDragState,
    WorkspaceTabDropIntent, WorkspaceTabDropZone, WorkspaceTabInsertionSide,
    resolve_workspace_tab_drop_intent,
};

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn absolute_fill_layout() -> LayoutStyle {
    let mut layout = fill_layout();
    layout.position = PositionStyle::Absolute;
    layout.inset = InsetStyle {
        top: Some(Px(0.0)).into(),
        right: Some(Px(0.0)).into(),
        bottom: Some(Px(0.0)).into(),
        left: Some(Px(0.0)).into(),
    };
    layout
}

// Note: edge drop zones are computed from pane geometry (nearest-edge-with-margin), rather than
// via overlapping edge regions. See `WorkspacePaneDragGeometry`.

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

fn pane_stack_layout() -> LayoutStyle {
    let mut layout = fill_layout();
    layout.position = PositionStyle::Relative;
    layout
}

fn flex_grow_fill_layout(grow: f32) -> LayoutStyle {
    let mut layout = fill_layout();
    layout.flex.grow = grow;
    layout
}

fn with_alpha(mut color: Color, alpha: f32) -> Color {
    color.a = alpha;
    color
}

fn drop_preview_fill(theme: &Theme) -> Option<Color> {
    theme
        .color_by_key("workspace.pane.drop_preview_fill")
        .or_else(|| theme.color_by_key("ring").map(|c| with_alpha(c, 0.14)))
}

fn drop_preview_border(theme: &Theme) -> Option<Color> {
    theme
        .color_by_key("workspace.pane.drop_preview_border")
        .or_else(|| theme.color_by_key("ring"))
}

fn edge_for_drop_zone(zone: WorkspaceTabDropZone) -> Option<EdgeDropZone> {
    match zone {
        WorkspaceTabDropZone::Left => Some(EdgeDropZone::Left),
        WorkspaceTabDropZone::Right => Some(EdgeDropZone::Right),
        WorkspaceTabDropZone::Up => Some(EdgeDropZone::Up),
        WorkspaceTabDropZone::Down => Some(EdgeDropZone::Down),
        WorkspaceTabDropZone::Center => None,
    }
}

fn drop_zone_for_edge(edge: EdgeDropZone) -> WorkspaceTabDropZone {
    match edge {
        EdgeDropZone::Left => WorkspaceTabDropZone::Left,
        EdgeDropZone::Right => WorkspaceTabDropZone::Right,
        EdgeDropZone::Up => WorkspaceTabDropZone::Up,
        EdgeDropZone::Down => WorkspaceTabDropZone::Down,
    }
}

fn compute_drop_zone_for_position(
    geom: WorkspacePaneDragGeometry,
    position: Point,
    previous: Option<WorkspaceTabDropZone>,
) -> WorkspaceTabDropZone {
    let prev_edge = previous.and_then(edge_for_drop_zone);
    compute_edge_drop_zone(
        geom.bounds,
        position,
        geom.edge_margin,
        prev_edge,
        geom.edge_hysteresis,
    )
    .map(drop_zone_for_edge)
    .unwrap_or(WorkspaceTabDropZone::Center)
}

fn drop_preview_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    zone: WorkspaceTabDropZone,
    fill: Option<Color>,
    border_color: Option<Color>,
    corner_radii: Corners,
) -> AnyElement {
    let border = Edges::all(Px(1.0));

    match zone {
        WorkspaceTabDropZone::Center => cx.container(
            ContainerProps {
                layout: absolute_fill_layout(),
                background: fill,
                border,
                border_color,
                corner_radii,
                ..Default::default()
            },
            |_cx| Vec::new(),
        ),
        WorkspaceTabDropZone::Left | WorkspaceTabDropZone::Right => {
            let preview_first = zone == WorkspaceTabDropZone::Left;
            cx.flex(
                FlexProps {
                    layout: absolute_fill_layout(),
                    direction: Axis::Horizontal,
                    ..Default::default()
                },
                |cx| {
                    let preview = cx.container(
                        ContainerProps {
                            layout: flex_grow_fill_layout(1.0),
                            background: fill,
                            border,
                            border_color,
                            corner_radii,
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    );
                    let empty = cx.container(
                        ContainerProps {
                            layout: flex_grow_fill_layout(1.0),
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    );

                    if preview_first {
                        vec![preview, empty]
                    } else {
                        vec![empty, preview]
                    }
                },
            )
        }
        WorkspaceTabDropZone::Up | WorkspaceTabDropZone::Down => {
            let preview_first = zone == WorkspaceTabDropZone::Up;
            cx.flex(
                FlexProps {
                    layout: absolute_fill_layout(),
                    direction: Axis::Vertical,
                    ..Default::default()
                },
                |cx| {
                    let preview = cx.container(
                        ContainerProps {
                            layout: flex_grow_fill_layout(1.0),
                            background: fill,
                            border,
                            border_color,
                            corner_radii,
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    );
                    let empty = cx.container(
                        ContainerProps {
                            layout: flex_grow_fill_layout(1.0),
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    );

                    if preview_first {
                        vec![preview, empty]
                    } else {
                        vec![empty, preview]
                    }
                },
            )
        }
    }
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

fn drop_border_width(zone: WorkspaceTabDropZone) -> Edges {
    match zone {
        WorkspaceTabDropZone::Center => Edges::all(Px(2.0)),
        WorkspaceTabDropZone::Left => Edges {
            left: Px(2.0),
            ..Edges::all(Px(1.0))
        },
        WorkspaceTabDropZone::Right => Edges {
            right: Px(2.0),
            ..Edges::all(Px(1.0))
        },
        WorkspaceTabDropZone::Up => Edges {
            top: Px(2.0),
            ..Edges::all(Px(1.0))
        },
        WorkspaceTabDropZone::Down => Edges {
            bottom: Px(2.0),
            ..Edges::all(Px(1.0))
        },
    }
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

    let tab_drag_for_root = tab_drag.clone();
    let clear_hover: OnInternalDrag = Arc::new(move |host, acx, drag| {
        let Some(session) = host.drag(drag.pointer_id) else {
            return false;
        };
        if session.kind != DRAG_KIND_WORKSPACE_TAB || !session.dragging {
            return false;
        }
        if session.current_window != acx.window {
            return false;
        }
        let session_source_window = session.source_window;

        match drag.kind {
            InternalDragKind::Over | InternalDragKind::Enter => {
                let mut did_clear = false;
                let _ = host.models_mut().update(&tab_drag_for_root, |st| {
                    if st.pointer != Some(drag.pointer_id)
                        || st.source_window != Some(session_source_window)
                    {
                        return;
                    }
                    if st.hovered_pane.is_some() || st.hovered_zone.is_some() {
                        st.hovered_pane = None;
                        st.hovered_zone = None;
                        st.hovered_tab = None;
                        st.hovered_tab_side = None;
                        st.hovered_pane_tab_rects = Vec::new();
                        did_clear = true;
                    }
                });
                if did_clear {
                    host.request_redraw(acx.window);
                }
                did_clear
            }
            InternalDragKind::Leave | InternalDragKind::Cancel => {
                let mut did_clear = false;
                let _ = host.models_mut().update(&tab_drag_for_root, |st| {
                    if st.pointer != Some(drag.pointer_id)
                        || st.source_window != Some(session_source_window)
                    {
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
            InternalDragKind::Drop => false,
        }
    });

    cx.internal_drag_region(
        InternalDragRegionProps {
            layout: fill_layout(),
            enabled: true,
        },
        |cx| {
            cx.internal_drag_region_on_internal_drag(clear_hover.clone());
            vec![render_node_with_resize(
                cx,
                &window,
                &window_snapshot.pane_tree,
                &[],
                active_pane,
                &tab_drag,
                render_pane,
            )]
        },
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
        WorkspacePaneTree::Leaf(pane) => {
            render_leaf(cx, window, pane, active_pane, tab_drag, render_pane)
        }
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
    window: &Model<WorkspaceWindowLayout>,
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

    let (drop_zone, can_drop) = cx
        .get_model_cloned(tab_drag, Invalidation::Paint)
        .map(|st| {
            let hovered = st.pointer.is_some()
                && st.dragged_tab.is_some()
                && st
                    .hovered_pane
                    .as_deref()
                    .is_some_and(|p| p == pane.id.as_ref());
            if !hovered {
                return (None, false);
            }

            let zone = st.hovered_zone.unwrap_or(WorkspaceTabDropZone::Center);
            let can_drop = match zone {
                WorkspaceTabDropZone::Center => st
                    .source_pane
                    .as_deref()
                    .is_some_and(|p| p != pane.id.as_ref()),
                WorkspaceTabDropZone::Left
                | WorkspaceTabDropZone::Right
                | WorkspaceTabDropZone::Up
                | WorkspaceTabDropZone::Down => true,
            };
            (Some(zone), can_drop)
        })
        .unwrap_or((None, false));

    let (border, border_color, corner_radii, background) = {
        let theme = Theme::global(cx.app);
        let background = theme.color_by_key("workspace.pane.bg");
        (
            if can_drop && drop_zone.is_some() {
                drop_border_width(drop_zone.unwrap_or(WorkspaceTabDropZone::Center))
            } else {
                pane_border_width(is_active)
            },
            if can_drop && drop_zone.is_some() {
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

    let window_model = window.clone();
    let tab_drag_model = tab_drag.clone();

    let drag_handler: OnInternalDrag = {
        let pane_id = pane_id.clone();
        let tab_drag_model = tab_drag_model.clone();
        let window_model = window_model.clone();
        let move_tab_cmd = move_tab_cmd.clone();

        Arc::new(move |host, acx, drag| {
            let Some(session) = host.drag(drag.pointer_id) else {
                return false;
            };
            if session.kind != DRAG_KIND_WORKSPACE_TAB || !session.dragging {
                return false;
            }
            if session.current_window != acx.window {
                return false;
            }
            let session_source_window = session.source_window;

            match drag.kind {
                InternalDragKind::Over | InternalDragKind::Enter => {
                    let mut handled = false;
                    let _ = host.models_mut().update(&tab_drag_model, |st| {
                        if st.pointer != Some(drag.pointer_id)
                            || st.source_window != Some(session_source_window)
                        {
                            return;
                        }
                        let geom = st
                            .pane_geometry
                            .iter()
                            .find(|(id, _)| id.as_ref() == pane_id.as_ref())
                            .map(|(_, g)| *g);
                        let next_zone = geom
                            .map(|geom| {
                                compute_drop_zone_for_position(geom, drag.position, st.hovered_zone)
                            })
                            .unwrap_or(WorkspaceTabDropZone::Center);

                        if st.hovered_pane.as_deref() != Some(pane_id.as_ref())
                            || st.hovered_zone != Some(next_zone)
                        {
                            st.hovered_pane = Some(pane_id.clone());
                            st.hovered_zone = Some(next_zone);
                            if next_zone != WorkspaceTabDropZone::Center {
                                st.hovered_tab = None;
                                st.hovered_tab_side = None;
                                st.hovered_pane_tab_rects = Vec::new();
                            }
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
                        if st.pointer != Some(drag.pointer_id)
                            || st.source_window != Some(session_source_window)
                        {
                            return;
                        }
                        if st.hovered_pane.as_deref() == Some(pane_id.as_ref()) {
                            st.hovered_pane = None;
                            st.hovered_zone = None;
                            st.hovered_tab = None;
                            st.hovered_tab_side = None;
                            st.hovered_pane_tab_rects = Vec::new();
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
                        if st.pointer != Some(drag.pointer_id)
                            || st.source_window != Some(session_source_window)
                        {
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
                    let mut intent: WorkspaceTabDropIntent = WorkspaceTabDropIntent::None;
                    let _ = host.models_mut().update(&tab_drag_model, |st| {
                        if st.pointer != Some(drag.pointer_id)
                            || st.source_window != Some(session_source_window)
                        {
                            return;
                        }
                        if st.hovered_pane.as_deref() != Some(pane_id.as_ref()) {
                            return;
                        }

                        let geom = st
                            .pane_geometry
                            .iter()
                            .find(|(id, _)| id.as_ref() == pane_id.as_ref())
                            .map(|(_, g)| *g);
                        let zone = geom
                            .map(|geom| {
                                compute_drop_zone_for_position(geom, drag.position, st.hovered_zone)
                            })
                            .unwrap_or_else(|| {
                                st.hovered_zone.unwrap_or(WorkspaceTabDropZone::Center)
                            });

                        intent = resolve_workspace_tab_drop_intent(st, &pane_id, zone);

                        *st = WorkspaceTabDragState::default();
                    });

                    match intent {
                        WorkspaceTabDropIntent::None => false,
                        WorkspaceTabDropIntent::MoveToPane {
                            source,
                            dragged_tab,
                            target,
                        } => {
                            if let Some(cmd) = pane_activate_command(source.as_ref()) {
                                host.dispatch_command(Some(acx.window), cmd);
                            }
                            if let Some(cmd) = tab_activate_command(dragged_tab.as_ref()) {
                                host.dispatch_command(Some(acx.window), cmd);
                            }
                            if let Some(cmd) = pane_move_active_tab_to_command(target.as_ref()) {
                                host.dispatch_command(Some(acx.window), cmd);
                            } else if let Some(cmd) = move_tab_cmd.clone() {
                                host.dispatch_command(Some(acx.window), cmd);
                            }
                            host.request_redraw(acx.window);
                            true
                        }
                        WorkspaceTabDropIntent::InsertToPane {
                            source,
                            dragged_tab,
                            target,
                            target_tab,
                            side,
                        } => {
                            if let Some(cmd) = pane_activate_command(source.as_ref()) {
                                host.dispatch_command(Some(acx.window), cmd);
                            }
                            if let Some(cmd) = tab_activate_command(dragged_tab.as_ref()) {
                                host.dispatch_command(Some(acx.window), cmd);
                            }
                            if let Some(cmd) = pane_move_active_tab_to_command(target.as_ref()) {
                                host.dispatch_command(Some(acx.window), cmd);
                            } else if let Some(cmd) = move_tab_cmd.clone() {
                                host.dispatch_command(Some(acx.window), cmd);
                            }

                            let cmd = match side {
                                WorkspaceTabInsertionSide::Before => {
                                    tab_move_active_before_command(target_tab.as_ref())
                                }
                                WorkspaceTabInsertionSide::After => {
                                    tab_move_active_after_command(target_tab.as_ref())
                                }
                            };
                            if let Some(cmd) = cmd {
                                host.dispatch_command(Some(acx.window), cmd);
                            }

                            host.request_redraw(acx.window);
                            true
                        }
                        WorkspaceTabDropIntent::SplitAndMove {
                            source,
                            dragged_tab,
                            target,
                            axis,
                            side,
                        } => {
                            let new_pane_id = host
                                .models_mut()
                                .read(&window_model, |w| w.generate_next_pane_id())
                                .ok();

                            let Some(new_pane_id) = new_pane_id else {
                                return false;
                            };

                            if let Some(cmd) = pane_activate_command(target.as_ref()) {
                                host.dispatch_command(Some(acx.window), cmd);
                            }
                            if let Some(cmd) = pane_split_command(axis, side, new_pane_id.as_ref())
                            {
                                host.dispatch_command(Some(acx.window), cmd);
                            }

                            if let Some(cmd) = pane_activate_command(source.as_ref()) {
                                host.dispatch_command(Some(acx.window), cmd);
                            }
                            if let Some(cmd) = tab_activate_command(dragged_tab.as_ref()) {
                                host.dispatch_command(Some(acx.window), cmd);
                            }
                            if let Some(cmd) = pane_move_active_tab_to_command(new_pane_id.as_ref())
                            {
                                host.dispatch_command(Some(acx.window), cmd);
                            }

                            host.request_redraw(acx.window);
                            true
                        }
                    }
                }
            }
        })
    };

    cx.pointer_region(
        PointerRegionProps {
            layout: pane_container_layout(),
            enabled: true,
            ..Default::default()
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

            let pane_bounds_element = Cell::<Option<GlobalElementId>>::new(None);

            vec![cx.internal_drag_region(
                InternalDragRegionProps {
                    layout: pane_stack_layout(),
                    enabled: true,
                },
                |cx| {
                    cx.internal_drag_region_on_internal_drag(drag_handler.clone());

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
                                    ..Default::default()
                                },
                                |cx| vec![render_pane(cx, pane, is_active, tab_drag.clone())],
                            )]
                        },
                    );
                    pane_bounds_element.set(Some(inner.id));

                    let preview = if can_drop {
                        drop_zone.map(|zone| {
                            let theme = Theme::global(cx.app);
                            drop_preview_element(
                                cx,
                                zone,
                                drop_preview_fill(theme),
                                drop_preview_border(theme),
                                corner_radii,
                            )
                        })
                    } else {
                        None
                    };

                    let mut children = vec![inner];
                    if let Some(preview) = preview {
                        children.push(preview);
                    }

                    if let Some(bounds) = pane_bounds_element
                        .get()
                        .and_then(|id| cx.last_bounds_for_element(id))
                    {
                        let theme = Theme::global(cx.app);
                        let max_px = theme
                            .metric_by_key("workspace.pane.drop_edge_px")
                            .unwrap_or(Px(24.0))
                            .0;
                        let ratio = theme
                            .number_by_key("workspace.pane.drop_edge_ratio")
                            .filter(|r| r.is_finite() && *r > 0.0);
                        let min_px = theme
                            .metric_by_key("workspace.pane.drop_edge_min_px")
                            .unwrap_or(Px(18.0))
                            .0;
                        let hysteresis_px = theme
                            .metric_by_key("workspace.pane.drop_edge_hysteresis_px")
                            .unwrap_or(Px(0.0));

                        let edge_margin = if let Some(ratio) = ratio {
                            let base = bounds.size.width.0.min(bounds.size.height.0) * ratio;
                            Px(base.clamp(min_px, max_px.max(min_px)))
                        } else {
                            Px(max_px.max(0.0))
                        };

                        let geom = WorkspacePaneDragGeometry {
                            bounds,
                            edge_margin,
                            edge_hysteresis: hysteresis_px,
                        };

                        let pane_id_for_geom = pane_id.clone();
                        let tab_drag_for_geom = tab_drag.clone();
                        let _ = cx.app.models_mut().update(&tab_drag_for_geom, |st| {
                            if st.pointer.is_none() {
                                return;
                            }
                            if let Some(existing) = st
                                .pane_geometry
                                .iter_mut()
                                .find(|(id, _)| id.as_ref() == pane_id_for_geom.as_ref())
                            {
                                if existing.1 != geom {
                                    existing.1 = geom;
                                }
                            } else {
                                st.pane_geometry.push((pane_id_for_geom.clone(), geom));
                            }
                        });
                    }

                    children
                },
            )]
        },
    )
}
