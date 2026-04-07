//! Node graph overlay host state (UI-only).

use fret_core::{KeyCode, Point, Rect, Size};
use fret_runtime::Model;
use fret_ui::{UiHost, retained_bridge::*};

use crate::core::{GroupId, SymbolId};
use crate::ops::GraphTransaction;
use crate::ui::compat_transport::NodeGraphEditQueue;
use crate::ui::controller::NodeGraphController;
use crate::ui::style::NodeGraphStyle;

use super::layout_hidden_child_and_release_focus;
use super::rename_policy::{
    RenameOverlaySession, RenameOverlaySessionKey, active_rename_session,
    build_rename_commit_transaction, clear_rename_sessions, rename_overlay_rect_at,
    rename_overlay_should_cancel_on_focus_loss, rename_session_seed_text,
};

/// UI-only overlay state for a node graph editor instance.
#[derive(Debug, Default, Clone)]
pub struct NodeGraphOverlayState {
    pub group_rename: Option<GroupRenameOverlay>,
    pub symbol_rename: Option<SymbolRenameOverlay>,
}

#[derive(Debug, Clone)]
pub struct GroupRenameOverlay {
    pub group: GroupId,
    pub invoked_at_window: Point,
}

#[derive(Debug, Clone)]
pub struct SymbolRenameOverlay {
    pub symbol: SymbolId,
    pub invoked_at_window: Point,
}

/// Overlay host that provides a TextInput-backed inline rename UI.
///
/// Expected children:
/// - child 0: a `BoundTextInput` bound to `group_rename_text`.
pub struct NodeGraphOverlayHost {
    graph: Model<crate::Graph>,
    edits: Option<Model<NodeGraphEditQueue>>,
    controller: Option<NodeGraphController>,
    overlays: Model<NodeGraphOverlayState>,
    group_rename_text: Model<String>,
    canvas_node: fret_core::NodeId,
    style: NodeGraphStyle,

    last_opened_session: Option<RenameOverlaySessionKey>,
    rename_bounds: Option<Rect>,
    active: bool,
}

impl NodeGraphOverlayHost {
    pub fn new(
        graph: Model<crate::Graph>,
        overlays: Model<NodeGraphOverlayState>,
        group_rename_text: Model<String>,
        canvas_node: fret_core::NodeId,
        style: NodeGraphStyle,
    ) -> Self {
        Self {
            graph,
            edits: None,
            controller: None,
            overlays,
            group_rename_text,
            canvas_node,
            style,
            last_opened_session: None,
            rename_bounds: None,
            active: false,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn with_edit_queue(mut self, edits: Model<NodeGraphEditQueue>) -> Self {
        self.edits = Some(edits);
        self
    }

    /// Routes retained rename commits through a store-backed controller.
    ///
    /// This is the public advanced retained seam. Raw edit-queue fallback remains
    /// crate-internal compatibility plumbing for focused retained tests and temporary
    /// migration harnesses.
    pub fn with_controller(mut self, controller: NodeGraphController) -> Self {
        self.controller = Some(controller);
        self
    }

    fn submit_transaction<H: UiHost>(&self, host: &mut H, tx: &GraphTransaction) {
        crate::ui::retained_submit::submit_graph_transaction(
            host,
            self.controller.as_ref(),
            self.edits.as_ref(),
            &self.graph,
            tx,
        );
    }

    fn active_rename_session<H: UiHost>(&self, host: &H) -> Option<RenameOverlaySession> {
        self.overlays
            .read_ref(host, active_rename_session)
            .ok()
            .flatten()
    }

    fn close_rename_sessions<H: UiHost>(&mut self, host: &mut H) {
        let _ = self.overlays.update(host, |s, _cx| {
            clear_rename_sessions(s);
        });
    }

    fn commit_rename_session<H: UiHost>(&mut self, host: &mut H, session: &RenameOverlaySession) {
        let to = self
            .group_rename_text
            .read_ref(host, |t| t.clone())
            .ok()
            .unwrap_or_default();
        let tx = self
            .graph
            .read_ref(host, |g| build_rename_commit_transaction(g, session, &to))
            .ok()
            .flatten();
        if let Some(tx) = tx {
            self.submit_transaction(host, &tx);
        }
    }
}

impl<H: UiHost> Widget<H> for NodeGraphOverlayHost {
    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        self.active && self.rename_bounds.is_some_and(|r| r.contains(position))
    }

    fn semantics_present(&self) -> bool {
        self.active
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &fret_core::Event) {
        let Some(session) = self.active_rename_session(&*cx.app) else {
            return;
        };

        match event {
            fret_core::Event::KeyDown { key, .. } => match *key {
                KeyCode::Escape => {
                    self.close_rename_sessions(cx.app);
                    cx.request_focus(self.canvas_node);
                    cx.stop_propagation();
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Layout);
                }
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    self.commit_rename_session(cx.app, &session);
                    self.close_rename_sessions(cx.app);
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
        let session = self.active_rename_session(&*cx.app);
        self.active = session.is_some();

        if let Some(session) = session {
            let session_key = session.key();
            let just_opened = self.last_opened_session != Some(session_key);
            if rename_overlay_should_cancel_on_focus_loss(child, cx.focus, just_opened) {
                self.close_rename_sessions(cx.app);
                self.last_opened_session = None;
                self.rename_bounds = None;
                self.active = false;
                if let Some(child) = child {
                    layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
                }
                return cx.bounds.size;
            }
            if just_opened {
                self.last_opened_session = Some(session_key);
                let seed_text = self
                    .graph
                    .read_ref(cx.app, |g| rename_session_seed_text(g, &session))
                    .ok()
                    .unwrap_or_default();
                let _ = self.group_rename_text.update(cx.app, |t, _cx| {
                    *t = seed_text;
                });
                if let Some(child) = child {
                    cx.tree.set_focus(Some(child));
                }
            }

            let rect = rename_overlay_rect_at(&self.style, session.invoked_at_window(), cx.bounds);
            self.rename_bounds = Some(rect);
            if let Some(child) = child {
                cx.layout_in(child, rect);
            }
        } else {
            self.last_opened_session = None;
            self.rename_bounds = None;
            if let Some(child) = child {
                layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
            }
        }

        cx.bounds.size
    }
}
