use fret_core::Rect;

use super::{NullServices, TestUiHostImpl};

pub(crate) fn event_cx<'a>(
    host: &'a mut TestUiHostImpl,
    services: &'a mut NullServices,
    bounds: Rect,
    prevented_default_actions: &'a mut fret_runtime::DefaultActionSet,
) -> fret_ui::retained_bridge::EventCx<'a, TestUiHostImpl> {
    fret_ui::retained_bridge::EventCx::new(
        host,
        services,
        fret_core::NodeId::default(),
        None,
        None,
        fret_runtime::InputContext::default(),
        None,
        1.0,
        None,
        None,
        false,
        false,
        None,
        false,
        prevented_default_actions,
        &[],
        None,
        None,
        bounds,
    )
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
        notify_requested: false,
        notify_requested_location: None,
        stop_propagation: false,
    }
}
