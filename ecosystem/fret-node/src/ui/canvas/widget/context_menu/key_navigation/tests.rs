use super::*;
use crate::core::{Graph, GraphId};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ui::canvas::widget::widget_tail::{
    WidgetHandledCx, WidgetPaintInvalidationCx, WidgetRedrawCx,
};
use fret_core::KeyCode;
use fret_core::Px;
use fret_runtime::ModelStore;

struct StubHost;

struct StubCx {
    stopped: bool,
    redraws: usize,
    paint_invalidations: usize,
    activation_calls: usize,
    activated_menu_active_item: Option<usize>,
    activation_outcome: super::super::selection_activation::ContextMenuSelectionActivationOutcome,
}

impl Default for StubCx {
    fn default() -> Self {
        Self {
            stopped: false,
            redraws: 0,
            paint_invalidations: 0,
            activation_calls: 0,
            activated_menu_active_item: None,
            activation_outcome:
                super::super::selection_activation::ContextMenuSelectionActivationOutcome::Activated,
        }
    }
}

impl WidgetRedrawCx<StubHost> for StubCx {
    fn request_redraw(&mut self) {
        self.redraws += 1;
    }
}

impl WidgetPaintInvalidationCx<StubHost> for StubCx {
    fn invalidate_paint(&mut self) {
        self.paint_invalidations += 1;
    }
}

impl WidgetHandledCx<StubHost> for StubCx {
    fn stop_propagation(&mut self) {
        self.stopped = true;
    }
}

impl ContextMenuKeyDownCx<StubHost, NoopNodeGraphCanvasMiddleware> for StubCx {
    fn activate_context_menu_active_selection(
        &mut self,
        _canvas: &mut NodeGraphCanvasWith<NoopNodeGraphCanvasMiddleware>,
        menu: &ContextMenuState,
    ) -> super::super::selection_activation::ContextMenuSelectionActivationOutcome {
        self.activation_calls += 1;
        self.activated_menu_active_item = Some(menu.active_item);
        self.activation_outcome
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

fn assert_finished(cx: &StubCx) {
    assert!(cx.stopped);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
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

#[test]
fn advance_active_item_skips_disabled_entries() {
    let mut menu = super::test_support::menu(
        vec![
            super::test_support::item("first", false),
            super::test_support::item("second", true),
            super::test_support::item("third", false),
        ],
        0,
    );

    active_item::advance_context_menu_active_item(&mut menu, false);

    assert_eq!(menu.active_item, 1);
}

#[test]
fn advance_active_item_wraps_backwards() {
    let mut menu = super::test_support::menu(
        vec![
            super::test_support::item("first", true),
            super::test_support::item("second", false),
            super::test_support::item("third", true),
        ],
        0,
    );

    active_item::advance_context_menu_active_item(&mut menu, true);

    assert_eq!(menu.active_item, 2);
}

#[test]
fn typeahead_falls_back_to_single_character_match() {
    let mut menu = super::test_support::menu(
        vec![
            super::test_support::item("Alpha", true),
            super::test_support::item("Beta", true),
        ],
        0,
    );
    menu.typeahead.push('a');

    typeahead::apply_context_menu_typeahead(&mut menu, 'b');

    assert_eq!(menu.typeahead, "b");
    assert_eq!(menu.active_item, 1);
}

#[test]
fn pop_typeahead_reports_whether_anything_changed() {
    let mut menu = super::test_support::menu(vec![super::test_support::item("Alpha", true)], 0);
    assert!(!typeahead::pop_context_menu_typeahead(&mut menu));

    menu.typeahead.push('a');
    assert!(typeahead::pop_context_menu_typeahead(&mut menu));
    assert!(menu.typeahead.is_empty());
}

#[test]
fn sync_hovered_item_promotes_enabled_item_and_clears_typeahead() {
    let mut menu = super::test_support::menu(
        vec![
            super::test_support::item("Alpha", true),
            super::test_support::item("Beta", true),
        ],
        0,
    );
    menu.typeahead.push('a');

    assert!(hover::sync_context_menu_hovered_item(&mut menu, Some(1)));
    assert_eq!(menu.hovered_item, Some(1));
    assert_eq!(menu.active_item, 1);
    assert!(menu.typeahead.is_empty());
}

#[test]
fn sync_hovered_item_keeps_active_for_disabled_item() {
    let mut menu = super::test_support::menu(
        vec![
            super::test_support::item("Alpha", true),
            super::test_support::item("Beta", false),
        ],
        0,
    );

    assert!(hover::sync_context_menu_hovered_item(&mut menu, Some(1)));
    assert_eq!(menu.hovered_item, Some(1));
    assert_eq!(menu.active_item, 0);
}

#[test]
fn pointer_move_without_context_menu_is_side_effect_free() {
    let mut canvas = test_canvas();
    let mut cx = StubCx::default();

    let handled = pointer_move::handle_context_menu_pointer_move_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        Point::new(Px(0.0), Px(0.0)),
        1.0,
    );

    assert!(!handled);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
}

#[test]
fn pointer_move_updates_hover_and_invalidates_paint() {
    let mut canvas = test_canvas();
    canvas.interaction.context_menu = Some(super::test_support::menu(
        vec![
            super::test_support::item("Alpha", true),
            super::test_support::item("Beta", true),
        ],
        0,
    ));
    let mut cx = StubCx::default();
    let position = row_position(&canvas, 1);

    let handled = pointer_move::handle_context_menu_pointer_move_event::<StubHost, _>(
        &mut canvas,
        &mut cx,
        position,
        1.0,
    );

    assert!(handled);
    let menu = canvas.interaction.context_menu.as_ref().unwrap();
    assert_eq!(menu.hovered_item, Some(1));
    assert_eq!(menu.active_item, 1);
    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
}

#[test]
fn pointer_move_same_hover_does_not_invalidate_paint_again() {
    let mut canvas = test_canvas();
    canvas.interaction.context_menu = Some(super::test_support::menu(
        vec![
            super::test_support::item("Alpha", true),
            super::test_support::item("Beta", true),
        ],
        0,
    ));
    let mut cx = StubCx::default();
    let position = row_position(&canvas, 0);

    assert!(pointer_move::handle_context_menu_pointer_move_event::<
        StubHost,
        _,
    >(&mut canvas, &mut cx, position, 1.0));
    assert!(pointer_move::handle_context_menu_pointer_move_event::<
        StubHost,
        _,
    >(&mut canvas, &mut cx, position, 1.0));

    assert_eq!(cx.redraws, 1);
    assert_eq!(cx.paint_invalidations, 1);
}

#[test]
fn key_down_without_context_menu_is_side_effect_free() {
    let mut canvas = test_canvas();
    let mut cx = StubCx::default();

    let handled =
        handle_context_menu_key_down_event::<StubHost, _>(&mut canvas, &mut cx, KeyCode::ArrowDown);

    assert!(!handled);
    assert!(canvas.interaction.context_menu.is_none());
    assert!(!cx.stopped);
    assert_eq!(cx.redraws, 0);
    assert_eq!(cx.paint_invalidations, 0);
    assert_eq!(cx.activation_calls, 0);
}

#[test]
fn key_down_arrow_down_advances_active_item_and_finishes() {
    let mut canvas = test_canvas();
    let mut menu = super::test_support::menu(
        vec![
            super::test_support::item("Alpha", true),
            super::test_support::item("Beta", true),
        ],
        0,
    );
    menu.typeahead.push('a');
    canvas.interaction.context_menu = Some(menu);
    let mut cx = StubCx::default();

    let handled =
        handle_context_menu_key_down_event::<StubHost, _>(&mut canvas, &mut cx, KeyCode::ArrowDown);

    assert!(handled);
    let menu = canvas.interaction.context_menu.as_ref().unwrap();
    assert_eq!(menu.active_item, 1);
    assert!(menu.typeahead.is_empty());
    assert_eq!(cx.activation_calls, 0);
    assert_finished(&cx);
}

#[test]
fn key_down_enter_activates_active_item_and_closes_menu() {
    let mut canvas = test_canvas();
    canvas.interaction.context_menu = Some(super::test_support::menu(
        vec![
            super::test_support::item("Alpha", true),
            super::test_support::item("Beta", true),
        ],
        1,
    ));
    let mut cx = StubCx::default();

    let handled =
        handle_context_menu_key_down_event::<StubHost, _>(&mut canvas, &mut cx, KeyCode::Enter);

    assert!(handled);
    assert!(canvas.interaction.context_menu.is_none());
    assert_eq!(cx.activation_calls, 1);
    assert_eq!(cx.activated_menu_active_item, Some(1));
    assert_finished(&cx);
}

#[test]
fn key_down_enter_keep_open_restores_menu_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.context_menu = Some(super::test_support::menu(
        vec![super::test_support::item("Alpha", true)],
        0,
    ));
    let mut cx = StubCx {
        activation_outcome:
            super::super::selection_activation::ContextMenuSelectionActivationOutcome::KeepOpen,
        ..StubCx::default()
    };

    let handled =
        handle_context_menu_key_down_event::<StubHost, _>(&mut canvas, &mut cx, KeyCode::Enter);

    assert!(handled);
    assert!(canvas.interaction.context_menu.is_some());
    assert_eq!(cx.activation_calls, 1);
    assert_eq!(cx.activated_menu_active_item, Some(0));
    assert_finished(&cx);
}

#[test]
fn key_down_typeahead_updates_active_item_and_finishes() {
    let mut canvas = test_canvas();
    canvas.interaction.context_menu = Some(super::test_support::menu(
        vec![
            super::test_support::item("Alpha", true),
            super::test_support::item("Beta", true),
        ],
        0,
    ));
    let mut cx = StubCx::default();

    let handled =
        handle_context_menu_key_down_event::<StubHost, _>(&mut canvas, &mut cx, KeyCode::KeyB);

    assert!(handled);
    let menu = canvas.interaction.context_menu.as_ref().unwrap();
    assert_eq!(menu.active_item, 1);
    assert_eq!(menu.typeahead, "b");
    assert_eq!(cx.activation_calls, 0);
    assert_finished(&cx);
}

#[test]
fn key_down_backspace_pops_typeahead_and_finishes() {
    let mut canvas = test_canvas();
    let mut menu = super::test_support::menu(vec![super::test_support::item("Alpha", true)], 0);
    menu.typeahead.push_str("al");
    canvas.interaction.context_menu = Some(menu);
    let mut cx = StubCx::default();

    let handled =
        handle_context_menu_key_down_event::<StubHost, _>(&mut canvas, &mut cx, KeyCode::Backspace);

    assert!(handled);
    let menu = canvas.interaction.context_menu.as_ref().unwrap();
    assert_eq!(menu.typeahead, "a");
    assert_eq!(cx.activation_calls, 0);
    assert_finished(&cx);
}
