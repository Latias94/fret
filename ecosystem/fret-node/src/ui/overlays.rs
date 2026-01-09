//! Node-graph editor overlays (UI-only).
//!
//! Overlays are transient, screen-space affordances that should not be serialized into the graph
//! asset. They are hosted outside the canvas render transform (ADR 0135) so they can use regular
//! `fret-ui` widgets (focus, IME, clipboard, semantics).

use fret_core::{KeyCode, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::{UiHost, retained_bridge::*};

use super::edit_queue::NodeGraphEditQueue;
use super::style::NodeGraphStyle;
use crate::core::GroupId;
use crate::ops::{GraphOp, GraphTransaction};

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

    fn clamp_overlay_rect(&self, desired_origin: Point, bounds: Rect) -> Rect {
        let w = self.style.context_menu_width.max(40.0);
        let h = (self.style.context_menu_item_height.max(20.0)
            + 2.0 * self.style.context_menu_padding)
            .max(24.0);

        let min_x = bounds.origin.x.0;
        let min_y = bounds.origin.y.0;
        let max_x = bounds.origin.x.0 + (bounds.size.width.0 - w).max(0.0);
        let max_y = bounds.origin.y.0 + (bounds.size.height.0 - h).max(0.0);

        let x = desired_origin.x.0.clamp(min_x, max_x);
        let y = desired_origin.y.0.clamp(min_y, max_y);
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
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

            let rect = self.clamp_overlay_rect(rename.invoked_at_window, cx.bounds);
            self.group_rename_bounds = Some(rect);
            if let Some(child) = child {
                cx.layout_in(child, rect);
            }
        } else {
            self.last_opened_group = None;
            self.group_rename_bounds = None;
            if let Some(child) = child {
                cx.layout_in(
                    child,
                    Rect::new(cx.bounds.origin, Size::new(Px(0.0), Px(0.0))),
                );
                if cx.focus == Some(child) {
                    cx.tree.set_focus(Some(self.canvas_node));
                }
            }
        }

        cx.bounds.size
    }
}
