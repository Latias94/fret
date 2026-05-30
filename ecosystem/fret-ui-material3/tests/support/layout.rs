use fret_core::{Edges, NodeId, Px, Rect};
use fret_ui::{
    UiTree,
    element::{AnyElement, ContainerProps},
};

use super::host::TestHost;

pub(crate) fn paint_alpha(paint: &fret_core::Paint) -> f32 {
    match paint {
        fret_core::Paint::Solid(c) => c.a,
        _ => 1.0,
    }
}

pub(crate) fn find_first_bounds_with_size(
    ui: &UiTree<TestHost>,
    root: NodeId,
    width: f32,
    height: f32,
) -> Option<Rect> {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let Some(r) = ui.debug_node_visual_bounds(node)
            && (r.size.width.0 - width).abs() < 0.1
            && (r.size.height.0 - height).abs() < 0.1
        {
            return Some(r);
        }

        for child in ui.children(node) {
            stack.push(child);
        }
    }
    None
}

pub(crate) fn semantics_node_id_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Option<NodeId> {
    ui.semantics_snapshot().and_then(|snapshot| {
        snapshot
            .nodes
            .iter()
            .find_map(|node| (node.test_id.as_deref() == Some(test_id)).then_some(node.id))
    })
}

pub(crate) fn with_padding<'a, H: fret_ui::UiHost>(
    cx: &mut fret_ui::elements::ElementContext<'a, H>,
    padding: Px,
    child: AnyElement,
) -> AnyElement {
    cx.container(
        ContainerProps {
            padding: Edges::all(padding).into(),
            ..Default::default()
        },
        move |_cx| vec![child],
    )
}
