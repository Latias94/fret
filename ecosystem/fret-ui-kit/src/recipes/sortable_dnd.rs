//! Sortable/reorder recipe built on the headless `fret-dnd` toolbox.
//!
//! This is intentionally not a "full component": it focuses on the DnD policy wiring and keeps
//! visuals/content fully caller-owned.

use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Modifiers, MouseButton, PointerId, Px};
use fret_runtime::{DragKindId, Model};
use fret_ui::action::{
    OnPointerCancel, OnPointerDown, OnPointerMove, OnPointerUp, PointerCancelCx, PointerDownCx,
    PointerMoveCx, PointerUpCx,
};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, PointerRegionProps};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::model_watch::ModelWatchExt as _;
use crate::dnd;
use crate::dnd::{
    ActivationConstraint, Axis, CollisionStrategy, DndItemId, DndPointerForwarders,
    DndPointerForwardersConfig, DndScopeId, DndUpdate, InsertionSide, SensorOutput,
    insertion_side_for_pointer,
};
use crate::ui;
use crate::{IntoUiElement, Space, collect_children};

const DRAG_KIND_SORTABLE_REORDER: DragKindId = DragKindId(100);

#[derive(Debug, Clone, Copy)]
pub struct SortableReorderListProps {
    pub row_height: Px,
    pub activation: ActivationConstraint,
    pub collision_strategy: CollisionStrategy,
}

impl Default for SortableReorderListProps {
    fn default() -> Self {
        Self {
            row_height: Px(32.0),
            activation: ActivationConstraint::Distance { px: 6.0 },
            collision_strategy: CollisionStrategy::ClosestCenter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SortablePointerState {
    active: DndItemId,
    over: DndItemId,
    dragging: bool,
}

#[derive(Debug, Default, Clone)]
struct SortableDndState {
    pointers: HashMap<PointerId, SortablePointerState>,
}

fn get_state_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<SortableDndState> {
    cx.local_model(SortableDndState::default)
}

/// Sortable/reorder helper:
/// - renders a list driven by `items` (a `Vec<DndItemId>`),
/// - routes pointer events through `DndPointerForwarders`,
/// - on drop, mutates `items` by moving the active id to the `over` position.
///
/// Notes:
/// - This is a minimal MVP intended to validate `fret-dnd` policy wiring.
/// - Geometry is sourced from `last_bounds_for_element` (prev-bounds snapshot), so the first frame may not have
///   droppable rects yet. Most use-sites will naturally render continuously during interactions.
#[allow(clippy::too_many_arguments)]
pub fn sortable_reorder_list<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<DndItemId>>,
    props: SortableReorderListProps,
    mut row_contents: impl FnMut(&mut ElementContext<'_, H>, DndItemId) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    let SortableReorderListProps {
        row_height,
        activation,
        collision_strategy,
    } = props;

    let ids = cx.watch_model(&items).layout().cloned_or_default();
    let state = get_state_model(cx);
    let dnd = dnd::dnd_service_model(cx);
    let frame_id = cx.frame_id;
    let scope = DndScopeId(cx.root_id().0);

    let theme = Theme::global(&*cx.app);
    let list_bg = theme
        .color_by_key("list.background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_token("card"));
    let row_hover = theme
        .color_by_key("list.hover.background")
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));
    let row_active = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));

    let state_snapshot = cx.watch_model(&state).paint().cloned_or_default();
    let (active, over) = state_snapshot
        .pointers
        .iter()
        .min_by_key(|(pointer_id, _)| pointer_id.0)
        .map(|(_, st)| (Some(st.active), Some(st.over)))
        .unwrap_or((None, None));

    let mut children: Vec<AnyElement> = Vec::new();

    for id in ids {
        let state_on_down = state.clone();
        let state_on_move = state.clone();
        let state_on_up = state.clone();
        let state_on_cancel = state.clone();
        let items_on_up = items.clone();
        let dnd_on_up = dnd.clone();
        let on_update_state = state.clone();
        let on_update = Arc::new(
            move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  update: &DndUpdate| {
                let (pointer_id, over) = match update.sensor {
                    SensorOutput::DragStart { pointer_id, .. }
                    | SensorOutput::DragMove { pointer_id, .. } => (pointer_id, update.over),
                    _ => return,
                };

                let mut changed = false;
                let _ = host.models_mut().update(&on_update_state, |st| {
                    let Some(state) = st.pointers.get_mut(&pointer_id) else {
                        return;
                    };
                    state.dragging = true;
                    if let Some(over) = over {
                        state.over = over;
                    }
                    changed = true;
                });

                if changed {
                    host.request_redraw(action_cx.window);
                }
            },
        );

        let el = cx.keyed(id.0, |cx| {
            let mut pr = PointerRegionProps::default();
            pr.layout.size.width = Length::Fill;
            pr.layout.size.height = Length::Px(row_height);

            let forwarders = DndPointerForwarders::new(
                dnd.clone(),
                frame_id,
                DndPointerForwardersConfig::for_kind(DRAG_KIND_SORTABLE_REORDER)
                    .scope(scope)
                    .activation_constraint(activation)
                    .collision_strategy(collision_strategy)
                    .consume_events(false)
                    .on_update(on_update.clone()),
            );
            let down_forwarder = forwarders.on_pointer_down();
            let move_forwarder = forwarders.on_pointer_move();
            let up_forwarder = forwarders.on_pointer_up();
            let cancel_forwarder = forwarders.on_pointer_cancel();
            let cancel_forwarder_on_move = cancel_forwarder.clone();

            let on_down: OnPointerDown = Arc::new(move |host, action_cx, down: PointerDownCx| {
                if down.button != MouseButton::Left {
                    return false;
                }
                if down.modifiers != Modifiers::default() {
                    // MVP policy: avoid claiming modified clicks until we have clearer interaction
                    // arbitration with selection/multi-select semantics.
                    return false;
                }

                let window = action_cx.window;
                let pointer_id = down.pointer_id;
                let _ = down_forwarder(host, action_cx, down);

                let inserted = host
                    .models_mut()
                    .update(&state_on_down, |st| {
                        st.pointers.insert(
                            pointer_id,
                            SortablePointerState {
                                active: id,
                                over: id,
                                dragging: false,
                            },
                        );
                    })
                    .is_ok();
                if inserted {
                    host.request_redraw(window);
                }
                inserted
            });

            let on_move: OnPointerMove = Arc::new(move |host, action_cx, mv: PointerMoveCx| {
                let window = action_cx.window;
                let pointer_id = mv.pointer_id;
                let mut tracked = false;
                let mut canceled = false;
                let _ = host.models_mut().update(&state_on_move, |st| {
                    if !st.pointers.contains_key(&pointer_id) {
                        return;
                    }
                    tracked = true;
                    if !mv.buttons.left {
                        st.pointers.remove(&pointer_id);
                        canceled = true;
                    }
                });

                if !tracked {
                    return false;
                }

                if canceled {
                    let _ = cancel_forwarder_on_move(
                        host,
                        action_cx,
                        PointerCancelCx {
                            pointer_id,
                            position: Some(mv.position),
                            position_local: Some(mv.position_local),
                            position_window: mv.position_window,
                            buttons: mv.buttons,
                            modifiers: mv.modifiers,
                            pointer_type: mv.pointer_type,
                            tick_id: mv.tick_id,
                            pixels_per_point: mv.pixels_per_point,
                            reason: fret_core::PointerCancelReason::LeftWindow,
                        },
                    );
                    host.request_redraw(window);
                    return true;
                }

                let _ = move_forwarder(host, action_cx, mv);

                host.models_mut()
                    .read(&state_on_move, |st| {
                        st.pointers
                            .get(&pointer_id)
                            .is_some_and(|state| state.dragging)
                    })
                    .unwrap_or(false)
            });

            let on_up: OnPointerUp = Arc::new(move |host, action_cx, up: PointerUpCx| {
                if up.button != MouseButton::Left {
                    return false;
                }

                let window = action_cx.window;
                let pointer_id = up.pointer_id;
                let up_position = up.position;
                let mut moved = false;
                let mut reorder: Option<(DndItemId, DndItemId)> = None;
                let mut had_pointer = false;

                let _ = host.models_mut().update(&state_on_up, |st| {
                    let Some(state) = st.pointers.remove(&pointer_id) else {
                        return;
                    };
                    had_pointer = true;
                    if state.dragging && state.active != state.over {
                        reorder = Some((state.active, state.over));
                    }
                });

                if !had_pointer {
                    return false;
                }

                let _ = up_forwarder(host, action_cx, up);

                if let Some((active, over)) = reorder {
                    let over_rect = dnd::droppable_rect_in_scope(
                        host.models_mut(),
                        &dnd_on_up,
                        window,
                        frame_id,
                        scope,
                        over,
                    );
                    let side = over_rect
                        .map(|rect| insertion_side_for_pointer(up_position, rect, Axis::Y))
                        .unwrap_or(InsertionSide::Before);

                    let _ = host.models_mut().update(&items_on_up, |ids| {
                        let Some(active_index) = ids.iter().position(|&v| v == active) else {
                            return;
                        };
                        let Some(over_index) = ids.iter().position(|&v| v == over) else {
                            return;
                        };

                        let mut insert_at = over_index.saturating_add(match side {
                            InsertionSide::Before => 0,
                            InsertionSide::After => 1,
                        });
                        if active_index < insert_at {
                            insert_at = insert_at.saturating_sub(1);
                        }

                        let item = ids.remove(active_index);
                        ids.insert(insert_at.min(ids.len()), item);
                        moved = true;
                    });
                }

                host.request_redraw(window);
                moved
            });

            let on_cancel: OnPointerCancel =
                Arc::new(move |host, action_cx, cancel: PointerCancelCx| {
                    let window = action_cx.window;
                    let mut had_pointer = false;
                    let _ = host.models_mut().update(&state_on_cancel, |st| {
                        had_pointer = st.pointers.remove(&cancel.pointer_id).is_some();
                    });

                    if !had_pointer {
                        return false;
                    }

                    let _ = cancel_forwarder(host, action_cx, cancel);
                    host.request_redraw(window);
                    true
                });

            let bg = if active == Some(id) {
                Some(row_active)
            } else if over == Some(id) {
                Some(row_hover)
            } else {
                None
            };

            cx.pointer_region(pr, |cx| {
                cx.pointer_region_on_pointer_down(on_down);
                cx.pointer_region_on_pointer_move(on_move);
                cx.pointer_region_on_pointer_up(on_up);
                cx.pointer_region_on_pointer_cancel(on_cancel);

                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;

                vec![cx.container(
                    ContainerProps {
                        layout,
                        background: bg.or(Some(list_bg)),
                        ..Default::default()
                    },
                    |cx| {
                        let element = cx.root_id();
                        if let Some(rect) = cx.last_bounds_for_element(element) {
                            dnd::register_droppable_rect_in_scope(
                                cx.app.models_mut(),
                                &dnd,
                                cx.window,
                                cx.frame_id,
                                scope,
                                id,
                                rect,
                                0,
                                false,
                            );
                        }
                        let items = row_contents(cx, id);
                        collect_children(cx, items)
                    },
                )]
            })
        });

        children.push(el);
    }

    ui::v_stack(|_cx| children)
        .gap(Space::N0)
        .justify_start()
        .items_stretch()
        .w_full()
        .into_element(cx)
}

#[cfg(test)]
mod tests;
