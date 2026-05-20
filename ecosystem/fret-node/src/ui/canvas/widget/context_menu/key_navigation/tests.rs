use super::*;
use crate::core::{Graph, GraphId};
use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ui::canvas::widget::widget_tail::{WidgetPaintInvalidationCx, WidgetRedrawCx};
use fret_core::Px;
use fret_runtime::ModelStore;

struct StubHost;

#[derive(Default)]
struct StubCx {
    redraws: usize,
    paint_invalidations: usize,
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
