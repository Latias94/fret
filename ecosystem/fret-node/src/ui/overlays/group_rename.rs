//! Node graph overlay host state (UI-only).

use fret_core::{KeyCode, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::{UiHost, retained_bridge::*};

use crate::core::GroupId;
use crate::ops::{GraphOp, GraphTransaction};
use crate::ui::{NodeGraphEditQueue, NodeGraphStyle};

use super::{clamp_rect_to_bounds, layout_hidden_child_and_release_focus};

fn group_rename_size_at(style: &NodeGraphStyle) -> Size {
    let w = style.context_menu_width.max(40.0);
    let h = (style.context_menu_item_height.max(20.0) + 2.0 * style.context_menu_padding).max(24.0);
    Size::new(Px(w), Px(h))
}

fn group_rename_rect_at(style: &NodeGraphStyle, desired_origin: Point, bounds: Rect) -> Rect {
    clamp_rect_to_bounds(
        Rect::new(desired_origin, group_rename_size_at(style)),
        bounds,
    )
}

/// UI-only overlay state for a node graph editor instance.
#[derive(Debug, Default, Clone)]
pub struct NodeGraphOverlayState {
    pub group_rename: Option<GroupRenameOverlay>,
}

#[derive(Debug, Clone)]
pub struct GroupRenameOverlay {
    pub group: GroupId,
    pub invoked_at_window: Point,
}

/// Overlay host that provides a TextInput-backed group rename UI.
///
/// Expected children:
/// - child 0: a `BoundTextInput` bound to `group_rename_text`.
pub struct NodeGraphOverlayHost {
    graph: Model<crate::Graph>,
    edits: Model<NodeGraphEditQueue>,
    overlays: Model<NodeGraphOverlayState>,
    group_rename_text: Model<String>,
    canvas_node: fret_core::NodeId,
    style: NodeGraphStyle,

    last_opened_group: Option<GroupId>,
    group_rename_bounds: Option<Rect>,
    active: bool,
}

impl NodeGraphOverlayHost {
    pub fn new(
        graph: Model<crate::Graph>,
        edits: Model<NodeGraphEditQueue>,
        overlays: Model<NodeGraphOverlayState>,
        group_rename_text: Model<String>,
        canvas_node: fret_core::NodeId,
        style: NodeGraphStyle,
    ) -> Self {
        Self {
            graph,
            edits,
            overlays,
            group_rename_text,
            canvas_node,
            style,
            last_opened_group: None,
            group_rename_bounds: None,
            active: false,
        }
    }

    fn current_group_rename<H: UiHost>(&self, host: &H) -> Option<GroupRenameOverlay> {
        self.overlays
            .read_ref(host, |s| s.group_rename.clone())
            .ok()
            .flatten()
    }

    fn close_group_rename<H: UiHost>(&mut self, host: &mut H) {
        let _ = self.overlays.update(host, |s, _cx| {
            s.group_rename = None;
        });
    }

    fn commit_group_rename<H: UiHost>(&mut self, host: &mut H, group: GroupId) {
        let to = self
            .group_rename_text
            .read_ref(host, |t| t.clone())
            .ok()
            .unwrap_or_default();
        let from = self
            .graph
            .read_ref(host, |g| g.groups.get(&group).map(|gg| gg.title.clone()))
            .ok()
            .flatten()
            .unwrap_or_default();

        if from == to {
            return;
        }

        let tx = GraphTransaction {
            label: Some("Rename Group".to_string()),
            ops: vec![GraphOp::SetGroupTitle {
                id: group,
                from,
                to,
            }],
        };
        let _ = self.edits.update(host, |q, _cx| {
            q.push(tx);
        });
    }
}

impl<H: UiHost> Widget<H> for NodeGraphOverlayHost {
    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        self.active
            && self
                .group_rename_bounds
                .is_some_and(|r| r.contains(position))
    }

    fn semantics_present(&self) -> bool {
        self.active
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &fret_core::Event) {
        let Some(rename) = self.current_group_rename(&*cx.app) else {
            return;
        };

        match event {
            fret_core::Event::KeyDown { key, .. } => match *key {
                KeyCode::Escape => {
                    self.close_group_rename(cx.app);
                    cx.request_focus(self.canvas_node);
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Layout);
                }
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    self.commit_group_rename(cx.app, rename.group);
                    self.close_group_rename(cx.app);
                    cx.request_focus(self.canvas_node);
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Layout);
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.overlays, Invalidation::Layout);
        cx.observe_model(&self.graph, Invalidation::Layout);
        cx.observe_model(&self.group_rename_text, Invalidation::Layout);

        let child = cx.children.get(0).copied();
        let rename = self.current_group_rename(&*cx.app);
        self.active = rename.is_some();

        if let Some(rename) = rename {
            if self.last_opened_group != Some(rename.group) {
                self.last_opened_group = Some(rename.group);
                let title = self
                    .graph
                    .read_ref(cx.app, |g| {
                        g.groups.get(&rename.group).map(|gg| gg.title.clone())
                    })
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                let _ = self.group_rename_text.update(cx.app, |t, _cx| {
                    *t = title;
                });
                if let Some(child) = child {
                    cx.tree.set_focus(Some(child));
                }
            }

            let rect = group_rename_rect_at(&self.style, rename.invoked_at_window, cx.bounds);
            self.group_rename_bounds = Some(rect);
            if let Some(child) = child {
                cx.layout_in(child, rect);
            }
        } else {
            self.last_opened_group = None;
            self.group_rename_bounds = None;
            if let Some(child) = child {
                layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
            }
        }

        cx.bounds.size
    }
}
