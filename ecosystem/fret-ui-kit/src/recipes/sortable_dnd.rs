//! Sortable/reorder recipe built on the headless `fret-dnd` toolbox.
//!
//! This is intentionally not a "full component": it focuses on the DnD policy wiring and keeps
//! visuals/content fully caller-owned.

use std::sync::Arc;

use fret_core::{Modifiers, MouseButton, Point, PointerId, Px};
use fret_dnd::{
    ActivationConstraint, CollisionStrategy, DndItemId, Droppable, PointerSensor, RegistrySnapshot,
    SensorEvent, SensorOutput, closest_center_collisions, pointer_within_collisions,
};
use fret_runtime::Model;
use fret_ui::action::{
    OnPointerDown, OnPointerMove, OnPointerUp, PointerDownCx, PointerMoveCx, PointerUpCx,
};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, PointerRegionProps};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::model_watch::ModelWatchExt as _;
use crate::declarative::stack;
use crate::{Items, Justify, LayoutRefinement, Space};

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

#[derive(Debug, Default, Clone)]
struct SortableDndState {
    sensor: PointerSensor,
    droppables: Vec<Droppable>,
    pointer_id: Option<PointerId>,
    active: Option<DndItemId>,
    over: Option<DndItemId>,
    dragging: bool,
}

#[derive(Debug, Default)]
struct SortableDndStateModel {
    model: Option<Model<SortableDndState>>,
}

fn get_state_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<SortableDndState> {
    let existing = cx.with_state(SortableDndStateModel::default, |st| st.model.clone());
    if let Some(m) = existing {
        return m;
    }

    let model = cx.app.models_mut().insert(SortableDndState::default());
    cx.with_state(SortableDndStateModel::default, |st| {
        st.model = Some(model.clone());
    });
    model
}

fn collisions_for(
    strategy: CollisionStrategy,
    droppables: &[Droppable],
    pointer: Point,
) -> Vec<fret_dnd::DndCollision> {
    let snapshot = RegistrySnapshot {
        draggables: Vec::new(),
        droppables: droppables.to_vec(),
    };
    match strategy {
        CollisionStrategy::PointerWithin => pointer_within_collisions(&snapshot, pointer),
        CollisionStrategy::ClosestCenter => closest_center_collisions(&snapshot, pointer),
    }
}

/// Sortable/reorder helper:
/// - renders a list driven by `items` (a `Vec<DndItemId>`),
/// - captures pointer and tracks a per-pointer `PointerSensor`,
/// - on drop, mutates `items` by moving the active id to the `over` position.
///
/// Notes:
/// - This is a minimal MVP intended to validate `fret-dnd` policy wiring.
/// - Geometry is sourced from `last_visual_bounds_for_element`, so the first frame may not have
///   droppable rects yet. Most use-sites will naturally render continuously during interactions.
#[allow(clippy::too_many_arguments)]
pub fn sortable_reorder_list<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<DndItemId>>,
    props: SortableReorderListProps,
    mut row_contents: impl FnMut(&mut ElementContext<'_, H>, DndItemId) -> Vec<AnyElement>,
) -> AnyElement {
    let SortableReorderListProps {
        row_height,
        activation,
        collision_strategy,
    } = props;

    let ids = cx.watch_model(&items).layout().cloned().unwrap_or_default();
    let state = get_state_model(cx);

    let theme = Theme::global(&*cx.app);
    let list_bg = theme
        .color_by_key("list.background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_required("card"));
    let row_hover = theme
        .color_by_key("list.hover.background")
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_required("accent"));
    let row_active = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_required("accent"));

    let state_snapshot = cx.watch_model(&state).paint().cloned().unwrap_or_default();
    let active = state_snapshot.active;
    let over = state_snapshot.over;

    let mut droppables: Vec<Droppable> = Vec::new();
    let mut children: Vec<AnyElement> = Vec::new();

    for id in ids {
        let state_on_down = state.clone();
        let state_on_move = state.clone();
        let state_on_up = state.clone();
        let items_on_up = items.clone();

        let el = cx.keyed(id.0, |cx| {
            let mut pr = PointerRegionProps::default();
            pr.layout.size.width = Length::Fill;
            pr.layout.size.height = Length::Px(row_height);

            let on_down: OnPointerDown = Arc::new(move |host, action_cx, down: PointerDownCx| {
                if down.button != MouseButton::Left {
                    return false;
                }
                if down.modifiers != Modifiers::default() {
                    // MVP policy: avoid claiming modified clicks until we have clearer interaction
                    // arbitration with selection/multi-select semantics.
                    return false;
                }

                host.capture_pointer();

                let _ = host.models_mut().update(&state_on_down, |st| {
                    st.sensor.set_constraint(activation);
                    let _ = st.sensor.handle(SensorEvent::Down {
                        pointer_id: down.pointer_id,
                        position: down.position,
                        tick: down.tick_id.0,
                    });
                    st.pointer_id = Some(down.pointer_id);
                    st.active = Some(id);
                    st.over = Some(id);
                    st.dragging = false;
                });
                host.request_redraw(action_cx.window);
                true
            });

            let on_move: OnPointerMove = Arc::new(move |host, _action_cx, mv: PointerMoveCx| {
                enum MoveOutcome {
                    Ignore,
                    Canceled,
                    Updated,
                }

                let mut outcome = MoveOutcome::Ignore;
                let _ = host.models_mut().update(&state_on_move, |st| {
                    if st.pointer_id != Some(mv.pointer_id) {
                        return;
                    }

                    if !mv.buttons.left {
                        st.pointer_id = None;
                        st.active = None;
                        st.over = None;
                        st.dragging = false;
                        st.sensor.clear_pointer(mv.pointer_id);
                        outcome = MoveOutcome::Canceled;
                        return;
                    }

                    st.sensor.set_constraint(activation);
                    let sensor = st.sensor.handle(SensorEvent::Move {
                        pointer_id: mv.pointer_id,
                        position: mv.position,
                        tick: mv.tick_id.0,
                    });

                    if matches!(
                        sensor,
                        SensorOutput::DragStart { .. } | SensorOutput::DragMove { .. }
                    ) {
                        st.dragging = true;
                    }

                    if st.dragging {
                        let cols = collisions_for(collision_strategy, &st.droppables, mv.position);
                        st.over = cols.first().map(|c| c.id);
                        outcome = MoveOutcome::Updated;
                    }
                });

                match outcome {
                    MoveOutcome::Ignore => false,
                    MoveOutcome::Canceled => {
                        host.release_pointer_capture();
                        host.request_redraw(_action_cx.window);
                        true
                    }
                    MoveOutcome::Updated => {
                        host.request_redraw(_action_cx.window);
                        true
                    }
                }
            });

            let on_up: OnPointerUp = Arc::new(move |host, action_cx, up: PointerUpCx| {
                if up.button != MouseButton::Left {
                    return false;
                }

                let mut moved = false;
                let mut reorder: Option<(DndItemId, DndItemId)> = None;

                let _ = host.models_mut().update(&state_on_up, |st| {
                    if st.pointer_id != Some(up.pointer_id) {
                        return;
                    }

                    if st.dragging {
                        if let (Some(active), Some(over)) = (st.active, st.over)
                            && active != over
                        {
                            reorder = Some((active, over));
                        }
                    }

                    st.pointer_id = None;
                    st.active = None;
                    st.over = None;
                    st.dragging = false;
                    st.sensor.clear_pointer(up.pointer_id);
                });

                host.release_pointer_capture();

                if let Some((active, over)) = reorder {
                    let _ = host.models_mut().update(&items_on_up, |ids| {
                        let Some(active_index) = ids.iter().position(|&v| v == active) else {
                            return;
                        };
                        let Some(over_index) = ids.iter().position(|&v| v == over) else {
                            return;
                        };
                        let item = ids.remove(active_index);
                        let insert_at = over_index.min(ids.len());
                        ids.insert(insert_at, item);
                        moved = true;
                    });
                }

                host.request_redraw(action_cx.window);
                moved
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
                            droppables.push(Droppable {
                                id,
                                rect,
                                disabled: false,
                                z_index: 0,
                            });
                        }
                        row_contents(cx, id)
                    },
                )]
            })
        });

        children.push(el);
    }

    let _ = cx.app.models_mut().update(&state, |st| {
        st.sensor.set_constraint(activation);
        st.droppables = droppables;
    });

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap_y(Space::N0)
            .justify(Justify::Start)
            .items(Items::Stretch)
            .layout(LayoutRefinement::default().w_full()),
        |_cx| children,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::rc::Rc;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, MouseButtons, PathCommand, PathConstraints, PathId, PathMetrics,
        PathService, PathStyle, PointerType, Rect, Size, SvgId, SvgService, TextBlobId,
        TextConstraints, TextInput, TextMetrics, TextService,
    };
    use fret_runtime::{FrameId, TickId};
    use fret_ui::ThemeConfig;
    use fret_ui::{Theme, UiTree};

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    fn bump(app: &mut App) {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn sortable_reorder_moves_item_to_over_index() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let items = app
            .models_mut()
            .insert(vec![DndItemId(1), DndItemId(2), DndItemId(3)]);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices::default();

        let row_ids: Rc<RefCell<Vec<fret_ui::GlobalElementId>>> = Rc::new(RefCell::new(Vec::new()));

        fn render(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut FakeServices,
            window: AppWindowId,
            bounds: Rect,
            items: Model<Vec<DndItemId>>,
            row_ids: Rc<RefCell<Vec<fret_ui::GlobalElementId>>>,
        ) -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "sortable", |cx| {
                row_ids.borrow_mut().clear();
                let el = sortable_reorder_list(
                    cx,
                    items,
                    SortableReorderListProps {
                        row_height: Px(32.0),
                        activation: ActivationConstraint::Distance { px: 6.0 },
                        collision_strategy: CollisionStrategy::ClosestCenter,
                    },
                    |cx, id| {
                        row_ids.borrow_mut().push(cx.root_id());
                        vec![cx.text(format!("Item {}", id.0))]
                    },
                );
                vec![el]
            })
        }

        // Needs two frames: geometry comes from `last_bounds_for_element` (prev-bounds snapshot).
        for _ in 0..2 {
            bump(&mut app);
            let root = render(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                items.clone(),
                row_ids.clone(),
            );
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let elements = row_ids.borrow().clone();
        assert_eq!(elements.len(), 3);

        let nodes = elements
            .iter()
            .map(|&el| fret_ui::elements::node_for_element(&mut app, window, el).expect("node"))
            .collect::<Vec<_>>();
        let rects = nodes
            .iter()
            .map(|&n| ui.debug_node_bounds(n).expect("bounds"))
            .collect::<Vec<_>>();

        assert!(
            rects[0].size.width.0 > 0.0 && rects[0].size.height.0 > 0.0,
            "expected non-empty row bounds"
        );
        assert!(
            rects[0].origin.y.0 < rects[1].origin.y.0 && rects[1].origin.y.0 < rects[2].origin.y.0,
            "expected stacked rows to have increasing y origins"
        );

        let center = |r: Rect| {
            Point::new(
                Px(r.origin.x.0 + r.size.width.0 * 0.5),
                Px(r.origin.y.0 + r.size.height.0 * 0.5),
            )
        };

        let start = center(rects[0]);
        let target = center(rects[2]);

        bump(&mut app);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: start,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        assert!(
            ui.captured_for(PointerId(0)).is_some(),
            "expected pointer to be captured after down"
        );

        bump(&mut app);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: target,
                buttons: MouseButtons {
                    left: true,
                    ..Default::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        assert!(
            ui.captured_for(PointerId(0)).is_some(),
            "expected pointer to remain captured during move"
        );

        bump(&mut app);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: target,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        assert!(
            ui.captured_for(PointerId(0)).is_none(),
            "expected pointer capture to be released after up"
        );

        bump(&mut app);
        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            items.clone(),
            row_ids.clone(),
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let after = app.models().get_cloned(&items).unwrap_or_default();
        assert_eq!(after, vec![DndItemId(2), DndItemId(3), DndItemId(1)]);
    }
}
