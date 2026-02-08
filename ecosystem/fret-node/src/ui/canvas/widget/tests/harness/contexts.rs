use fret_core::Rect;

use super::{NullServices, TestUiHostImpl};

pub(crate) fn event_cx<'a>(
    host: &'a mut TestUiHostImpl,
    services: &'a mut NullServices,
    bounds: Rect,
    prevented_default_actions: &'a mut fret_runtime::DefaultActionSet,
) -> fret_ui::retained_bridge::EventCx<'a, TestUiHostImpl> {
    fret_ui::retained_bridge::EventCx {
        app: host,
        services,
        node: fret_core::NodeId::default(),
        layer_root: None,
        window: None,
        input_ctx: fret_runtime::InputContext::default(),
        pointer_id: None,
        prevented_default_actions,
        children: &[],
        focus: None,
        captured: None,
        bounds,
        invalidations: Vec::new(),
        requested_focus: None,
        requested_capture: None,
        requested_cursor: None,
        notify_requested: false,
        notify_requested_location: None,
        stop_propagation: false,
    }
}

pub(crate) fn command_cx<'a>(
    host: &'a mut TestUiHostImpl,
    services: &'a mut NullServices,
    tree: &'a mut fret_ui::UiTree<TestUiHostImpl>,
) -> fret_ui::retained_bridge::CommandCx<'a, TestUiHostImpl> {
    fret_ui::retained_bridge::CommandCx {
        app: host,
        services,
        tree,
        node: fret_core::NodeId::default(),
        window: None,
        input_ctx: fret_runtime::InputContext::default(),
        focus: None,
        invalidations: Vec::new(),
        requested_focus: None,
        stop_propagation: false,
    }
}
