use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Modifiers, MouseButton, Point, PointerId, Px, Rect, Transform2D};
use fret_runtime::{DragKindId, Model};
use fret_ui::action::{
    OnPointerDown, OnPointerMove, OnPointerUp, PointerDownCx, PointerMoveCx, PointerUpCx,
};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PointerRegionProps, ScrollAxis,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::transition;
use fret_ui_kit::dnd::{
    self, ActivationConstraint, CollisionStrategy, DndItemId, DndScopeId, InsertionSide,
    SensorOutput, insertion_side_for_pointer,
};
use fret_ui_kit::primitives::presence;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space, ui};

use crate::ScrollArea;
use crate::test_id::attach_test_id;

const DRAG_KIND_KANBAN: DragKindId = DragKindId(101);

fn fnv1a64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn kanban_card_dnd_id(id: &str) -> DndItemId {
    DndItemId(fnv1a64(&format!("shadcn-extras.kanban.card.{id}")))
}

fn kanban_column_dnd_id(id: &str) -> DndItemId {
    DndItemId(fnv1a64(&format!("shadcn-extras.kanban.column.{id}")))
}

fn sanitize_test_id_suffix(s: &str) -> Arc<str> {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '-' {
            out.push(c.to_ascii_lowercase());
        } else if c == '_' || c.is_whitespace() {
            out.push('-');
        }
    }
    Arc::<str>::from(out)
}

#[derive(Debug, Clone)]
pub struct KanbanItem {
    pub id: Arc<str>,
    pub name: Arc<str>,
    pub column: Arc<str>,
}

impl KanbanItem {
    pub fn new(
        id: impl Into<Arc<str>>,
        name: impl Into<Arc<str>>,
        column: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            column: column.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KanbanColumn {
    pub id: Arc<str>,
    pub name: Arc<str>,
}

impl KanbanColumn {
    pub fn new(id: impl Into<Arc<str>>, name: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KanbanCardMode {
    Board,
    Overlay,
}

#[derive(Debug, Clone, Copy)]
pub struct KanbanCardCtx {
    pub mode: KanbanCardMode,
    pub dragging: bool,
    pub active: bool,
    pub over: bool,
}

#[derive(Debug, Clone)]
struct KanbanPointerState {
    active: DndItemId,
    over: Option<DndItemId>,
    over_side: Option<InsertionSide>,
    dragging: bool,
    pointer: Point,
    translation: Point,
    origin_rect: Option<fret_core::Rect>,
}

#[derive(Debug, Clone, Copy)]
struct KanbanLastDrag {
    active: DndItemId,
    translation: Point,
    origin_rect: Option<fret_core::Rect>,
}

#[derive(Debug, Default, Clone)]
struct KanbanDndState {
    pointers: HashMap<PointerId, KanbanPointerState>,
    last_drag: Option<KanbanLastDrag>,
}

#[derive(Debug, Default)]
struct KanbanDndStateModel {
    model: Option<Model<KanbanDndState>>,
}

fn get_state_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<KanbanDndState> {
    let existing = cx.with_state(KanbanDndStateModel::default, |st| st.model.clone());
    if let Some(m) = existing {
        return m;
    }

    let model = cx.app.models_mut().insert(KanbanDndState::default());
    cx.with_state(KanbanDndStateModel::default, |st| {
        st.model = Some(model.clone());
    });
    model
}

fn is_column_id(id: DndItemId, columns: &[KanbanColumn]) -> Option<Arc<str>> {
    columns
        .iter()
        .find(|c| kanban_column_dnd_id(c.id.as_ref()) == id)
        .map(|c| c.id.clone())
}

fn item_index_by_dnd_id(items: &[KanbanItem], id: DndItemId) -> Option<usize> {
    items
        .iter()
        .position(|it| kanban_card_dnd_id(it.id.as_ref()) == id)
}

fn insertion_index_for_end_of_column(items: &[KanbanItem], column: &Arc<str>) -> usize {
    items
        .iter()
        .rposition(|it| &it.column == column)
        .map(|idx| idx.saturating_add(1))
        .unwrap_or(items.len())
}

fn item_gap_for_sortable_rects(
    rects: &[Option<fret_core::Rect>],
    index: usize,
    active_index: usize,
) -> Px {
    // Ported from dnd-kit `verticalListSortingStrategy`'s `getItemGap`:
    // `repo-ref/dnd-kit/packages/sortable/src/strategies/verticalListSorting.ts`.
    let current = rects.get(index).and_then(|r| *r);
    let prev = if index > 0 {
        rects.get(index.saturating_sub(1)).and_then(|r| *r)
    } else {
        None
    };
    let next = rects.get(index.saturating_add(1)).and_then(|r| *r);

    let Some(current) = current else {
        return Px(0.0);
    };

    if active_index < index {
        if let Some(prev) = prev {
            return Px(current.origin.y.0 - (prev.origin.y.0 + prev.size.height.0));
        }
        if let Some(next) = next {
            return Px(next.origin.y.0 - (current.origin.y.0 + current.size.height.0));
        }
        return Px(0.0);
    }

    if let Some(next) = next {
        return Px(next.origin.y.0 - (current.origin.y.0 + current.size.height.0));
    }
    if let Some(prev) = prev {
        return Px(current.origin.y.0 - (prev.origin.y.0 + prev.size.height.0));
    }
    Px(0.0)
}

fn neighbor_gap(rects: &[Option<fret_core::Rect>], index: usize) -> Px {
    let current = rects.get(index).and_then(|r| *r);
    let prev = if index > 0 {
        rects.get(index.saturating_sub(1)).and_then(|r| *r)
    } else {
        None
    };
    let next = rects.get(index.saturating_add(1)).and_then(|r| *r);

    let Some(current) = current else {
        return Px(0.0);
    };

    if let Some(prev) = prev {
        return Px(current.origin.y.0 - (prev.origin.y.0 + prev.size.height.0));
    }
    if let Some(next) = next {
        return Px(next.origin.y.0 - (current.origin.y.0 + current.size.height.0));
    }
    Px(0.0)
}

#[derive(Debug, Clone)]
struct KanbanDragPlan {
    active: DndItemId,
    active_col: Arc<str>,
    active_index: usize,
    target_col: Arc<str>,
    insert_at: usize,
}

fn apply_drop_reorder(
    items: &mut Vec<KanbanItem>,
    columns: &[KanbanColumn],
    over_rect: Option<fret_core::Rect>,
    pointer: fret_core::Point,
    active: DndItemId,
    over: DndItemId,
) -> bool {
    if active == over {
        return false;
    }

    let Some(active_index) = item_index_by_dnd_id(items, active) else {
        return false;
    };

    let target_column = if let Some(col) = is_column_id(over, columns) {
        col
    } else {
        let Some(over_index) = item_index_by_dnd_id(items, over) else {
            return false;
        };
        items
            .get(over_index)
            .map(|it| it.column.clone())
            .unwrap_or_else(|| items[active_index].column.clone())
    };

    let mut insert_at = if is_column_id(over, columns).is_some() {
        insertion_index_for_end_of_column(items, &target_column)
    } else {
        let Some(over_index) = item_index_by_dnd_id(items, over) else {
            return false;
        };
        let side = over_rect
            .map(|rect| insertion_side_for_pointer(pointer, rect, dnd::Axis::Y))
            .unwrap_or(InsertionSide::Before);
        over_index.saturating_add(match side {
            InsertionSide::Before => 0,
            InsertionSide::After => 1,
        })
    };

    let mut item = items.remove(active_index);
    item.column = target_column;

    if active_index < insert_at {
        insert_at = insert_at.saturating_sub(1);
    }

    items.insert(insert_at.min(items.len()), item);
    true
}

/// A shadcn-styled Kanban board inspired by Kibo's shadcn blocks.
///
/// Upstream inspiration (MIT):
/// - `repo-ref/kibo/packages/kanban`
#[derive(Debug, Clone)]
pub struct Kanban {
    columns: Vec<KanbanColumn>,
    items: Model<Vec<KanbanItem>>,
    test_id: Option<Arc<str>>,
    activation: ActivationConstraint,
    collision_strategy: CollisionStrategy,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    column_layout: LayoutRefinement,
}

impl Kanban {
    pub fn new(
        columns: impl IntoIterator<Item = KanbanColumn>,
        items: Model<Vec<KanbanItem>>,
    ) -> Self {
        Self {
            columns: columns.into_iter().collect(),
            items,
            test_id: None,
            activation: ActivationConstraint::Distance { px: 6.0 },
            collision_strategy: CollisionStrategy::ClosestCenter,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            column_layout: LayoutRefinement::default()
                .w_px(Px(280.0))
                .min_h(Px(160.0))
                .overflow_hidden(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn activation(mut self, activation: ActivationConstraint) -> Self {
        self.activation = activation;
        self
    }

    pub fn collision_strategy(mut self, strategy: CollisionStrategy) -> Self {
        self.collision_strategy = strategy;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_column_layout(mut self, layout: LayoutRefinement) -> Self {
        self.column_layout = self.column_layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with(cx, |cx, item, _ctx| {
            ui::text(cx, item.name.clone())
                .font_medium()
                .w_full()
                .min_w_0()
                .truncate()
                .into_element(cx)
        })
    }

    pub fn into_element_with<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        render_card: impl Fn(&mut ElementContext<'_, H>, &KanbanItem, KanbanCardCtx) -> AnyElement,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let items = self.items.clone();
            let columns: Arc<[KanbanColumn]> = Arc::from(self.columns.into_boxed_slice());
            let activation = self.activation;
            let collision_strategy = self.collision_strategy;

            let dnd_svc = dnd::dnd_service_model(cx);
            let state = get_state_model(cx);
            let frame_id = cx.frame_id;
            let scope = DndScopeId(cx.root_id().0);

            let state_snapshot = cx.watch_model(&state).paint().cloned_or_default();
            let (active, over, over_side, dragging_open, translation, origin_rect) = state_snapshot
                .pointers
                .iter()
                .min_by_key(|(pointer_id, _)| pointer_id.0)
                .map(|(_, st)| {
                    (
                        Some(st.active),
                        st.over,
                        st.over_side,
                        st.dragging,
                        Some(st.translation),
                        st.origin_rect,
                    )
                })
                .unwrap_or((None, None, None, false, None, None));
            let translation = translation.unwrap_or_else(|| Point::new(Px(0.0), Px(0.0)));

            let chrome = ChromeRefinement::default().merge(self.chrome);
            let layout = LayoutRefinement::default()
                .w_full()
                .relative()
                .merge(self.layout);

            let root_props = decl_style::container_props(&theme, chrome, layout);

            let root_test_id = self
                .test_id
                .clone()
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.kanban"));

            let column_layout = self.column_layout;

            let el = cx.container(root_props, move |cx| {
                let items_snapshot = cx.watch_model(&items).layout().cloned_or_default();
                let sm_breakpoint = fret_ui_kit::declarative::viewport_width_at_least(
                    cx,
                    Invalidation::Layout,
                    fret_ui_kit::declarative::tailwind::SM,
                    fret_ui_kit::declarative::ViewportQueryHysteresis::default(),
                );
                let board_gap_x = if sm_breakpoint { Space::N4 } else { Space::N3 };
                let column_layout = if sm_breakpoint {
                    column_layout.clone()
                } else {
                    column_layout
                        .clone()
                        .merge(LayoutRefinement::default().w_px(Px(240.0)).min_w_0())
                };

                let overlay_presence =
                    presence::fade_presence_with_durations(cx, dragging_open, 1, 6);

                if state_snapshot.pointers.is_empty()
                    && state_snapshot.last_drag.is_some()
                    && !overlay_presence.present
                {
                    let _ = cx.app.models_mut().update(&state, |st| {
                        st.last_drag = None;
                    });
                }

                let overlay_active = if dragging_open {
                    active
                } else {
                    state_snapshot.last_drag.map(|d| d.active)
                };
                let overlay_translation = if dragging_open {
                    translation
                } else {
                    state_snapshot
                        .last_drag
                        .map(|d| d.translation)
                        .unwrap_or(translation)
                };
                let overlay_origin_rect = if dragging_open {
                    origin_rect
                } else {
                    state_snapshot.last_drag.and_then(|d| d.origin_rect)
                };

                let dragging = overlay_presence.present && overlay_active.is_some();

                let active_height = origin_rect.map(|r| r.size.height).unwrap_or(Px(0.0));
                let default_gap = decl_style::space(&theme, Space::N2);

                let drag_plan = (|| -> Option<KanbanDragPlan> {
                    if !dragging_open || active_height.0 <= 0.0 {
                        return None;
                    }

                    let active_id = active?;
                    let over_id = over?;
                    if active_id == over_id {
                        return None;
                    }

                    let active_item = items_snapshot
                        .iter()
                        .find(|it| kanban_card_dnd_id(it.id.as_ref()) == active_id)?;
                    let active_col = active_item.column.clone();

                    let mut active_index: usize = 0;
                    for it in items_snapshot.iter() {
                        if it.column.as_ref() != active_col.as_ref() {
                            continue;
                        }
                        if kanban_card_dnd_id(it.id.as_ref()) == active_id {
                            break;
                        }
                        active_index = active_index.saturating_add(1);
                    }

                    let (target_col, mut insert_at) =
                        if let Some(col) = is_column_id(over_id, columns.as_ref()) {
                            let target_len = items_snapshot
                                .iter()
                                .filter(|it| it.column.as_ref() == col.as_ref())
                                .count();
                            let mut insert_at = target_len;
                            if col.as_ref() == active_col.as_ref() && target_len > 0 {
                                insert_at = insert_at.saturating_sub(1);
                            }
                            (col, insert_at)
                        } else {
                            let over_item = items_snapshot
                                .iter()
                                .find(|it| kanban_card_dnd_id(it.id.as_ref()) == over_id)?;
                            let target_col = over_item.column.clone();

                            let mut over_index: usize = 0;
                            for it in items_snapshot.iter() {
                                if it.column.as_ref() != target_col.as_ref() {
                                    continue;
                                }
                                if kanban_card_dnd_id(it.id.as_ref()) == over_id {
                                    break;
                                }
                                over_index = over_index.saturating_add(1);
                            }

                            let side = over_side.unwrap_or(InsertionSide::Before);
                            let insert_at = over_index.saturating_add(match side {
                                InsertionSide::Before => 0,
                                InsertionSide::After => 1,
                            });

                            let target_len = items_snapshot
                                .iter()
                                .filter(|it| it.column.as_ref() == target_col.as_ref())
                                .count();
                            (target_col, insert_at.min(target_len))
                        };

                    if target_col.as_ref() == active_col.as_ref() && active_index < insert_at {
                        insert_at = insert_at.saturating_sub(1);
                    }

                    Some(KanbanDragPlan {
                        active: active_id,
                        active_col,
                        active_index,
                        target_col,
                        insert_at,
                    })
                })();

                let render_card = &render_card;

                let mut cols: Vec<AnyElement> = Vec::with_capacity(columns.len());
                for col in columns.iter().cloned() {
                    let dnd_for_render = dnd_svc.clone();
                    let column_layout = column_layout.clone();
                    let items = items.clone();
                    let state_on_down = state.clone();
                    let state_on_move = state.clone();
                    let state_on_up = state.clone();
                    let dnd_on_down = dnd_for_render.clone();
                    let dnd_on_move = dnd_for_render.clone();
                    let dnd_on_up = dnd_for_render.clone();

                    let col_id = col.id.clone();
                    let col_name = col.name.clone();
                    let col_dnd_id = kanban_column_dnd_id(col_id.as_ref());

                    let col_test_id = Arc::<str>::from(format!(
                        "shadcn-extras.kanban.column-{}",
                        sanitize_test_id_suffix(col_id.as_ref())
                    ));

                    let mut column_cards: Vec<KanbanItem> = items_snapshot
                        .iter()
                        .filter(|it| it.column.as_ref() == col_id.as_ref())
                        .cloned()
                        .collect();

                    let col_el = cx.keyed(col_dnd_id.0, |cx| {
                        let border_color = if over == Some(col_dnd_id) {
                            theme.color_required("primary")
                        } else {
                            theme.color_required("border")
                        };
                        let col_chrome = ChromeRefinement::default()
                            .border_1()
                            .rounded(Radius::Md)
                            .bg(ColorRef::Color(theme.color_required("secondary")))
                            .border_color(ColorRef::Color(border_color))
                            .text_color(ColorRef::Color(theme.color_required("foreground")));
                        let mut col_props =
                            decl_style::container_props(&theme, col_chrome, column_layout.clone());
                        let radius = col_props.corner_radii.top_left;
                        col_props.shadow = Some(decl_style::shadow_sm(&theme, radius));

                        let header = cx.container(
                            decl_style::container_props(
                                &theme,
                                ChromeRefinement::default().p(Space::N2),
                                LayoutRefinement::default().w_full(),
                            ),
                            |cx| {
                                vec![
                                    ui::text(cx, col_name)
                                        .font_semibold()
                                        .w_full()
                                        .min_w_0()
                                        .truncate()
                                        .into_element(cx),
                                ]
                            },
                        );
                        let header = attach_test_id(
                            header,
                            Arc::<str>::from(format!("{col_test_id}.header")),
                        );

                        let mut card_elems: Vec<AnyElement> =
                            Vec::with_capacity(column_cards.len());

                        let column_card_ids: Arc<[DndItemId]> = Arc::from(
                            column_cards
                                .iter()
                                .map(|card| kanban_card_dnd_id(card.id.as_ref()))
                                .collect::<Vec<_>>()
                                .into_boxed_slice(),
                        );
                        let column_rects: Arc<[Option<fret_core::Rect>]> = Arc::from(
                            column_card_ids
                                .iter()
                                .map(|card_id| {
                                    dnd::droppable_rect_in_scope(
                                        cx.app.models(),
                                        &dnd_for_render,
                                        cx.window,
                                        cx.frame_id,
                                        scope,
                                        *card_id,
                                    )
                                })
                                .collect::<Vec<_>>()
                                .into_boxed_slice(),
                        );
                        for (card_index, card) in column_cards.drain(..).enumerate() {
                            let card_id = card.id.clone();
                            let card = card;
                            let card_dnd_id = kanban_card_dnd_id(card_id.as_ref());

                            let card_test_id = Arc::<str>::from(format!(
                                "shadcn-extras.kanban.card-{}",
                                sanitize_test_id_suffix(card_id.as_ref())
                            ));

                            let active_card =
                                overlay_presence.present && overlay_active == Some(card_dnd_id);
                            let over_card = dragging_open && over == Some(card_dnd_id);

                            let state_on_down = state_on_down.clone();
                            let state_on_move = state_on_move.clone();
                            let state_on_up = state_on_up.clone();
                            let dnd_on_down = dnd_on_down.clone();
                            let dnd_on_move = dnd_on_move.clone();
                            let dnd_on_up = dnd_on_up.clone();
                            let items_on_up = items.clone();

                            let columns_for_move = columns.clone();
                            let columns_for_up = columns.clone();

                            let el = cx.keyed(card_dnd_id.0, |cx| {
                                let card_shift_sign = drag_plan.as_ref().map_or(0, |plan| {
                                    if plan.active == card_dnd_id {
                                        return 0;
                                    }

                                    if plan.active_col.as_ref() == plan.target_col.as_ref()
                                        && col_id.as_ref() == plan.active_col.as_ref()
                                    {
                                        if plan.active_index < plan.insert_at
                                            && card_index > plan.active_index
                                            && card_index <= plan.insert_at
                                        {
                                            return -1;
                                        }
                                        if plan.active_index > plan.insert_at
                                            && card_index >= plan.insert_at
                                            && card_index < plan.active_index
                                        {
                                            return 1;
                                        }
                                        return 0;
                                    }

                                    if col_id.as_ref() == plan.active_col.as_ref()
                                        && col_id.as_ref() != plan.target_col.as_ref()
                                        && card_index > plan.active_index
                                    {
                                        return -1;
                                    }

                                    if col_id.as_ref() == plan.target_col.as_ref()
                                        && col_id.as_ref() != plan.active_col.as_ref()
                                        && card_index >= plan.insert_at
                                    {
                                        return 1;
                                    }

                                    0
                                });

                                let shift_down =
                                    transition::drive_transition(cx, card_shift_sign > 0, 6);
                                let shift_up =
                                    transition::drive_transition(cx, card_shift_sign < 0, 6);
                                let shift_progress = shift_down.progress - shift_up.progress;

                                let shift_gap = drag_plan.as_ref().map_or(default_gap, |plan| {
                                    if plan.active_col.as_ref() == plan.target_col.as_ref()
                                        && col_id.as_ref() == plan.active_col.as_ref()
                                    {
                                        item_gap_for_sortable_rects(
                                            column_rects.as_ref(),
                                            card_index,
                                            plan.active_index,
                                        )
                                    } else if col_id.as_ref() == plan.active_col.as_ref() {
                                        item_gap_for_sortable_rects(
                                            column_rects.as_ref(),
                                            card_index,
                                            plan.active_index,
                                        )
                                    } else if col_id.as_ref() == plan.target_col.as_ref() {
                                        neighbor_gap(column_rects.as_ref(), card_index)
                                    } else {
                                        default_gap
                                    }
                                });
                                let shift_distance = Px(active_height.0 + shift_gap.0);
                                let preview_translation =
                                    Point::new(Px(0.0), Px(shift_distance.0 * shift_progress));

                                let mut pr = PointerRegionProps::default();
                                pr.layout.size.width = Length::Fill;
                                pr.layout.size.height = Length::Auto;

                                let on_down: OnPointerDown =
                                    Arc::new(move |host, action_cx, down: PointerDownCx| {
                                        if down.button != MouseButton::Left {
                                            return false;
                                        }
                                        if down.modifiers != Modifiers::default() {
                                            return false;
                                        }

                                        let _ = dnd::handle_pointer_down_in_scope(
                                            host.models_mut(),
                                            &dnd_on_down,
                                            action_cx.window,
                                            frame_id,
                                            DRAG_KIND_KANBAN,
                                            scope,
                                            down.pointer_id,
                                            down.position,
                                            down.tick_id,
                                            activation,
                                            collision_strategy,
                                            None,
                                        );

                                        let _ = host.models_mut().update(&state_on_down, |st| {
                                            st.pointers.insert(
                                                down.pointer_id,
                                                KanbanPointerState {
                                                    active: card_dnd_id,
                                                    over: Some(card_dnd_id),
                                                    over_side: None,
                                                    dragging: false,
                                                    pointer: down.position,
                                                    translation: Point::new(Px(0.0), Px(0.0)),
                                                    origin_rect: None,
                                                },
                                            );
                                        });

                                        host.request_redraw(action_cx.window);
                                        false
                                    });

                                let on_move: OnPointerMove =
                                    Arc::new(move |host, action_cx, mv: PointerMoveCx| {
                                        let mut tracked = false;
                                        let mut canceled = false;
                                        let mut became_dragging = false;
                                        let _ = host.models_mut().update(&state_on_move, |st| {
                                            if !st.pointers.contains_key(&mv.pointer_id) {
                                                return;
                                            }
                                            tracked = true;
                                            if !mv.buttons.left {
                                                st.pointers.remove(&mv.pointer_id);
                                                canceled = true;
                                            }
                                        });

                                        if !tracked {
                                            return false;
                                        }

                                        if canceled {
                                            let _ = dnd::handle_pointer_cancel_in_scope(
                                                host.models_mut(),
                                                &dnd_on_move,
                                                action_cx.window,
                                                frame_id,
                                                DRAG_KIND_KANBAN,
                                                scope,
                                                mv.pointer_id,
                                                mv.position,
                                                mv.tick_id,
                                                activation,
                                                collision_strategy,
                                                None,
                                            );
                                            host.release_pointer_capture();
                                            host.request_redraw(action_cx.window);
                                            return true;
                                        }

                                        let dnd_update = dnd::handle_pointer_move_in_scope(
                                            host.models_mut(),
                                            &dnd_on_move,
                                            action_cx.window,
                                            frame_id,
                                            DRAG_KIND_KANBAN,
                                            scope,
                                            mv.pointer_id,
                                            mv.position,
                                            mv.tick_id,
                                            activation,
                                            collision_strategy,
                                            None,
                                        );

                                        if matches!(
                                            dnd_update.sensor,
                                            SensorOutput::DragStart { .. }
                                                | SensorOutput::DragMove { .. }
                                        ) {
                                            let translation = match dnd_update.sensor {
                                                SensorOutput::DragMove { translation, .. } => {
                                                    translation
                                                }
                                                _ => Point::new(Px(0.0), Px(0.0)),
                                            };

                                            let origin_rect = dnd::droppable_rect_in_scope(
                                                host.models_mut(),
                                                &dnd_on_move,
                                                action_cx.window,
                                                frame_id,
                                                scope,
                                                card_dnd_id,
                                            );

                                            let next_over = dnd_update.over;
                                            let next_over_side = match next_over {
                                                None => None,
                                                Some(over_id)
                                                    if is_column_id(
                                                        over_id,
                                                        columns_for_move.as_ref(),
                                                    )
                                                    .is_some() =>
                                                {
                                                    None
                                                }
                                                Some(over_id) => dnd::droppable_rect_in_scope(
                                                    host.models_mut(),
                                                    &dnd_on_move,
                                                    action_cx.window,
                                                    frame_id,
                                                    scope,
                                                    over_id,
                                                )
                                                .map(|rect| {
                                                    insertion_side_for_pointer(
                                                        mv.position,
                                                        rect,
                                                        dnd::Axis::Y,
                                                    )
                                                }),
                                            };
                                            let _ =
                                                host.models_mut().update(&state_on_move, |st| {
                                                    let Some(state) =
                                                        st.pointers.get_mut(&mv.pointer_id)
                                                    else {
                                                        return;
                                                    };
                                                    if !state.dragging {
                                                        became_dragging = true;
                                                        state.origin_rect = origin_rect;
                                                    }
                                                    state.dragging = true;
                                                    state.translation = translation;
                                                    state.over = next_over;
                                                    state.over_side = next_over_side;
                                                    state.pointer = mv.position;
                                                });

                                            if became_dragging {
                                                host.capture_pointer();
                                            }
                                            host.request_redraw(action_cx.window);
                                            return true;
                                        }
                                        false
                                    });

                                let on_up: OnPointerUp =
                                    Arc::new(move |host, action_cx, up: PointerUpCx| {
                                        if up.button != MouseButton::Left {
                                            return false;
                                        }

                                        let mut reorder: Option<(DndItemId, DndItemId)> = None;
                                        let mut had_pointer = false;
                                        let mut was_dragging = false;

                                        let _ = host.models_mut().update(&state_on_up, |st| {
                                            let Some(state) = st.pointers.remove(&up.pointer_id)
                                            else {
                                                return;
                                            };
                                            had_pointer = true;
                                            if state.dragging {
                                                was_dragging = true;
                                                st.last_drag = Some(KanbanLastDrag {
                                                    active: state.active,
                                                    translation: state.translation,
                                                    origin_rect: state.origin_rect,
                                                });
                                                let Some(over) = state.over else {
                                                    return;
                                                };
                                                if state.active != over {
                                                    reorder = Some((state.active, over));
                                                }
                                            }
                                        });

                                        if !had_pointer {
                                            return false;
                                        }

                                        let _ = dnd::handle_pointer_up_in_scope(
                                            host.models_mut(),
                                            &dnd_on_up,
                                            action_cx.window,
                                            frame_id,
                                            DRAG_KIND_KANBAN,
                                            scope,
                                            up.pointer_id,
                                            up.position,
                                            up.tick_id,
                                            activation,
                                            collision_strategy,
                                            None,
                                        );
                                        if was_dragging {
                                            host.release_pointer_capture();
                                        }

                                        if let Some((active, over)) = reorder {
                                            let over_rect = dnd::droppable_rect_in_scope(
                                                host.models_mut(),
                                                &dnd_on_up,
                                                action_cx.window,
                                                frame_id,
                                                scope,
                                                over,
                                            );
                                            let _ =
                                                host.models_mut().update(&items_on_up, |items| {
                                                    let _ = apply_drop_reorder(
                                                        items,
                                                        columns_for_up.as_ref(),
                                                        over_rect,
                                                        up.position,
                                                        active,
                                                        over,
                                                    );
                                                });
                                        }

                                        if was_dragging {
                                            host.request_redraw(action_cx.window);
                                        }
                                        was_dragging
                                    });

                                let el = cx.pointer_region(pr, |cx| {
                                    cx.pointer_region_on_pointer_down(on_down);
                                    cx.pointer_region_on_pointer_move(on_move);
                                    cx.pointer_region_on_pointer_up(on_up);

                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Auto;

                                    let border_color = if over_card {
                                        theme.color_required("primary")
                                    } else {
                                        theme.color_required("border")
                                    };

                                    let card_chrome = ChromeRefinement::default()
                                        .border_1()
                                        .rounded(Radius::Md)
                                        .bg(ColorRef::Color(theme.color_required("card")))
                                        .border_color(ColorRef::Color(border_color))
                                        .text_color(ColorRef::Color(
                                            theme.color_required("card-foreground"),
                                        ))
                                        .p(Space::N3);
                                    let card_layout = LayoutRefinement::default().w_full();

                                    let mut card_props = decl_style::container_props(
                                        &theme,
                                        card_chrome,
                                        card_layout,
                                    );
                                    let radius = card_props.corner_radii.top_left;
                                    card_props.shadow = Some(if active_card {
                                        decl_style::shadow_md(&theme, radius)
                                    } else {
                                        decl_style::shadow_sm(&theme, radius)
                                    });

                                    let card = cx.container(card_props, |cx| {
                                        if !active_card {
                                            let element = cx.root_id();
                                            if let Some(rect) = cx.last_bounds_for_element(element)
                                            {
                                                let rect = Rect::new(
                                                    Point::new(
                                                        Px(rect.origin.x.0
                                                            + preview_translation.x.0),
                                                        Px(rect.origin.y.0
                                                            + preview_translation.y.0),
                                                    ),
                                                    rect.size,
                                                );
                                                dnd::register_droppable_rect_in_scope(
                                                    cx.app.models_mut(),
                                                    &dnd_for_render,
                                                    cx.window,
                                                    cx.frame_id,
                                                    scope,
                                                    card_dnd_id,
                                                    rect,
                                                    1,
                                                    false,
                                                );
                                            }
                                        }

                                        vec![render_card(
                                            cx,
                                            &card,
                                            KanbanCardCtx {
                                                mode: KanbanCardMode::Board,
                                                dragging,
                                                active: active_card,
                                                over: over_card,
                                            },
                                        )]
                                    });

                                    let card = if active_card {
                                        cx.opacity(0.25, |_cx| [card])
                                    } else {
                                        card
                                    };

                                    // Keep the outer wrapper's size stable for pointer capture.
                                    let wrapper = cx.container(
                                        ContainerProps {
                                            layout,
                                            ..Default::default()
                                        },
                                        |_cx| [card],
                                    );

                                    let wrap_transform = dragging
                                        || shift_down.animating
                                        || shift_up.animating
                                        || preview_translation.y.0.abs() > 0.0
                                        || preview_translation.x.0.abs() > 0.0;

                                    if wrap_transform {
                                        vec![cx.render_transform(
                                            Transform2D::translation(preview_translation),
                                            |_cx| [wrapper],
                                        )]
                                    } else {
                                        vec![wrapper]
                                    }
                                });
                                attach_test_id(el, card_test_id.clone())
                            });

                            card_elems.push(el);
                        }

                        let cards = stack::vstack(
                            cx,
                            stack::VStackProps::default()
                                .gap(Space::N2)
                                .layout(LayoutRefinement::default().w_full()),
                            |_cx| card_elems,
                        );
                        let cards =
                            attach_test_id(cards, Arc::<str>::from(format!("{col_test_id}.cards")));

                        let content = cx.container(
                            decl_style::container_props(
                                &theme,
                                ChromeRefinement::default().p(Space::N2),
                                LayoutRefinement::default().w_full().overflow_hidden(),
                            ),
                            |_cx| vec![cards],
                        );

                        let header = header;
                        let content = content;

                        let el = cx.container(col_props, move |cx| {
                            let element = cx.root_id();
                            if let Some(rect) = cx.last_bounds_for_element(element) {
                                dnd::register_droppable_rect_in_scope(
                                    cx.app.models_mut(),
                                    &dnd_for_render,
                                    cx.window,
                                    cx.frame_id,
                                    scope,
                                    col_dnd_id,
                                    rect,
                                    0,
                                    false,
                                );
                            }

                            vec![header, content]
                        });

                        attach_test_id(el, col_test_id)
                    });

                    cols.push(col_el);
                }

                let board = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap_x(board_gap_x)
                        .items_start()
                        .layout(LayoutRefinement::default()),
                    |_cx| cols,
                );
                let board = ScrollArea::new([board])
                    .axis(ScrollAxis::X)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx);

                let overlay = if dragging {
                    (|| -> Option<AnyElement> {
                        let active_id = overlay_active?;
                        let active_item = items_snapshot
                            .iter()
                            .find(|it| kanban_card_dnd_id(it.id.as_ref()) == active_id)?;
                        let active_rect = overlay_origin_rect.or_else(|| {
                            dnd::droppable_rect_in_scope(
                                cx.app.models(),
                                &dnd_svc,
                                cx.window,
                                cx.frame_id,
                                scope,
                                active_id,
                            )
                        })?;
                        let root_rect = cx.last_bounds_for_element(cx.root_id())?;

                        let local_origin = Point::new(
                            Px(active_rect.origin.x.0 - root_rect.origin.x.0),
                            Px(active_rect.origin.y.0 - root_rect.origin.y.0),
                        );

                        let mut overlay_layout = LayoutStyle::default();
                        overlay_layout.position = fret_ui::element::PositionStyle::Absolute;
                        overlay_layout.inset.left = Some(local_origin.x);
                        overlay_layout.inset.top = Some(local_origin.y);
                        overlay_layout.size.width = Length::Px(active_rect.size.width);
                        overlay_layout.size.height = Length::Px(active_rect.size.height);

                        let overlay_props = ContainerProps {
                            layout: overlay_layout,
                            ..Default::default()
                        };

                        let overlay_test_id = Arc::<str>::from(format!(
                            "shadcn-extras.kanban.card-{}.overlay",
                            sanitize_test_id_suffix(active_item.id.as_ref())
                        ));

                        let overlay_el = cx.container(overlay_props, |cx| {
                            let card_border_color = theme.color_required("primary");

                            let card_chrome = ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("card")))
                                .border_color(ColorRef::Color(card_border_color))
                                .text_color(ColorRef::Color(
                                    theme.color_required("card-foreground"),
                                ))
                                .p(Space::N3);
                            let card_layout = LayoutRefinement::default().size_full();

                            let mut card_props =
                                decl_style::container_props(&theme, card_chrome, card_layout);
                            let radius = card_props.corner_radii.top_left;
                            card_props.shadow = Some(decl_style::shadow_md(&theme, radius));

                            let card = cx.container(card_props, |cx| {
                                vec![render_card(
                                    cx,
                                    active_item,
                                    KanbanCardCtx {
                                        mode: KanbanCardMode::Overlay,
                                        dragging,
                                        active: true,
                                        over: false,
                                    },
                                )]
                            });

                            vec![cx.opacity(overlay_presence.opacity, |cx| {
                                vec![cx.render_transform(
                                    Transform2D::translation(overlay_translation),
                                    |_cx| [card],
                                )]
                            })]
                        });

                        Some(attach_test_id(overlay_el, overlay_test_id))
                    })()
                } else {
                    None
                };

                let mut out = vec![board];
                if let Some(overlay) = overlay {
                    out.push(overlay);
                }
                out
            });

            attach_test_id(el, root_test_id)
        })
    }
}
