use super::*;
use crate::core::{Graph, GraphId};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ui::canvas::widget::low_level_adapter::{
    CanvasHandledCx, CanvasPaintInvalidationCx, CanvasRedrawCx,
};
use fret_core::Px;
use fret_runtime::ModelStore;

struct StubHost;

#[derive(Default)]
struct StubCx {
    stopped: bool,
    redraws: usize,
    paint_invalidations: usize,
    activation_calls: usize,
    activated_item_label: Option<String>,
}

impl CanvasRedrawCx<StubHost> for StubCx {
    fn request_redraw(&mut self) {
        self.redraws += 1;
    }
}

impl CanvasPaintInvalidationCx<StubHost> for StubCx {
    fn invalidate_paint(&mut self) {
        self.paint_invalidations += 1;
    }
}

impl CanvasHandledCx<StubHost> for StubCx {
    fn stop_propagation(&mut self) {
        self.stopped = true;
    }
}

impl ContextMenuSelectionActivationCx<StubHost, NoopNodeGraphCanvasMiddleware> for StubCx {
    fn activate_context_menu_item(
        &mut self,
        _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
        _target: &ContextMenuTarget,
        _invoked_at: Point,
        item: NodeGraphContextMenuItem,
        _menu_candidates: &[InsertNodeCandidate],
    ) {
        self.activation_calls += 1;
        self.activated_item_label = Some(item.label.to_string());
    }
}

fn test_canvas() -> NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware> {
    let mut models = ModelStore::default();
    let graph = models.insert(Graph::new(GraphId::new()));
    let view = models.insert(NodeGraphViewState::default());
    let editor_config = models.insert(NodeGraphEditorConfig::default());
    NodeGraphCanvasWith::new_with_middleware(
        graph,
        view,
        editor_config,
        NoopNodeGraphCanvasMiddleware,
    )
}

fn menu(items: Vec<NodeGraphContextMenuItem>) -> ContextMenuState {
    ContextMenuState {
        origin: Point::new(Px(0.0), Px(0.0)),
        invoked_at: Point::new(Px(4.0), Px(8.0)),
        target: ContextMenuTarget::Background,
        items,
        candidates: Vec::new(),
        hovered_item: None,
        active_item: 0,
        typeahead: String::new(),
    }
}

fn item(label: &str, enabled: bool) -> NodeGraphContextMenuItem {
    NodeGraphContextMenuItem {
        label: Arc::<str>::from(label),
        enabled,
        action: NodeGraphContextMenuAction::Custom(7),
    }
}

fn row_position(
    canvas: &NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
    row_ix: usize,
) -> Point {
    let menu = canvas
        .interaction
        .context_menu
        .as_ref()
        .expect("context menu should be installed");
    let pad = canvas.style.paint.context_menu_padding;
    let item_h = canvas.style.paint.context_menu_item_height;
    Point::new(
        Px(menu.origin.x.0 + pad + 1.0),
        Px(menu.origin.y.0 + pad + (row_ix as f32 + 0.5) * item_h),
    )
}

fn assert_finished(cx: &StubCx) {
    assert!(cx.stopped);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
}

#[test]
fn pointer_down_without_context_menu_is_side_effect_free() {
    let mut canvas = test_canvas();
    let mut cx = StubCx::default();

    let handled = handle_context_menu_pointer_down_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        Point::new(Px(0.0), Px(0.0)),
        MouseButton::Left,
        1.0,
    );

    assert!(!handled);
    assert!(!cx.stopped);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
    assert_eq!(cx.activation_calls, 0);
}

#[test]
fn pointer_down_left_inside_enabled_item_activates_and_closes_menu() {
    let mut canvas = test_canvas();
    canvas.interaction.context_menu = Some(menu(vec![item("Alpha", true), item("Beta", true)]));
    let position = row_position(&canvas, 1);
    let mut cx = StubCx::default();

    let handled = handle_context_menu_pointer_down_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        position,
        MouseButton::Left,
        1.0,
    );

    assert!(handled);
    assert!(canvas.interaction.context_menu.is_none());
    assert_eq!(cx.activation_calls, 1);
    assert_eq!(cx.activated_item_label.as_deref(), Some("Beta"));
    assert_finished(&cx);
}

#[test]
fn pointer_down_left_disabled_item_restores_menu_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.context_menu = Some(menu(vec![item("Disabled", false)]));
    let position = row_position(&canvas, 0);
    let mut cx = StubCx::default();

    let handled = handle_context_menu_pointer_down_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        position,
        MouseButton::Left,
        1.0,
    );

    assert!(handled);
    assert!(canvas.interaction.context_menu.is_some());
    assert_eq!(cx.activation_calls, 0);
    assert_eq!(cx.activated_item_label, None);
    assert_finished(&cx);
}

#[test]
fn pointer_down_left_outside_menu_closes_menu_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.context_menu = Some(menu(vec![item("Alpha", true)]));
    let mut cx = StubCx::default();

    let handled = handle_context_menu_pointer_down_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        Point::new(Px(10_000.0), Px(10_000.0)),
        MouseButton::Left,
        1.0,
    );

    assert!(handled);
    assert!(canvas.interaction.context_menu.is_none());
    assert_eq!(cx.activation_calls, 0);
    assert_finished(&cx);
}

#[test]
fn pointer_down_right_button_leaves_menu_taken_and_unfinished() {
    let mut canvas = test_canvas();
    canvas.interaction.context_menu = Some(menu(vec![item("Alpha", true)]));
    let position = row_position(&canvas, 0);
    let mut cx = StubCx::default();

    let handled = handle_context_menu_pointer_down_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        position,
        MouseButton::Right,
        1.0,
    );

    assert!(!handled);
    assert!(canvas.interaction.context_menu.is_none());
    assert!(!cx.stopped);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
    assert_eq!(cx.activation_calls, 0);
}
