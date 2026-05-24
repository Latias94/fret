use fret_core::{Event, Modifiers, MouseButtons, Point, PointerEvent, PointerType, Px, Rect, Size};
use fret_runtime::Effect;
use fret_ui::Widget as _;

use crate::core::CanvasPoint;
use crate::ui::canvas::state::NodeDrag;

use super::{
    NullServices, TestUiHostImpl, event_cx, insert_graph_view_editor_config_with,
    make_test_graph_two_nodes,
};

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn pointer_move_auto_pan_timer_starts_for_node_drag_near_viewport_edge() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _b) = make_test_graph_two_nodes();
    let (graph, view, editor_config) =
        insert_graph_view_editor_config_with(&mut host, graph_value, |state| {
            state.interaction.auto_pan.on_node_drag = true;
        });
    let mut canvas = new_canvas!(host, graph, view, editor_config);

    canvas.interaction.last_pos = Some(Point::new(Px(799.0), Px(300.0)));
    canvas.interaction.node_drag = Some(NodeDrag {
        primary: a,
        node_ids: vec![a],
        nodes: vec![(a, CanvasPoint { x: 0.0, y: 0.0 })],
        current_nodes: vec![(a, CanvasPoint { x: 0.0, y: 0.0 })],
        current_groups: Vec::new(),
        preview_rev: 0,
        grab_offset: Point::new(Px(0.0), Px(0.0)),
        start_pos: Point::new(Px(0.0), Px(0.0)),
    });

    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    {
        let mut cx = event_cx(
            &mut host,
            &mut services,
            test_bounds(),
            &mut prevented_default_actions,
        );
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Move {
                pointer_id: fret_core::PointerId::default(),
                position: Point::new(Px(799.0), Px(300.0)),
                buttons: MouseButtons {
                    left: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    assert!(canvas.interaction.auto_pan_timer.is_some());
    assert!(
        host.effects.iter().any(|effect| matches!(
            effect,
            Effect::SetTimer {
                repeat: Some(_),
                ..
            }
        )),
        "expected pointer-move auto-pan timer sync to schedule a repeating timer"
    );
}
