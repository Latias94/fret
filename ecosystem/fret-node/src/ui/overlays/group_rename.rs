//! Node graph overlay host state (UI-only).

use fret_core::{Point, Rect, Size};
use fret_runtime::Model;
use fret_ui::{UiHost, retained_bridge::*};

use crate::core::{GroupId, SymbolId};
use crate::ui::compat_transport::NodeGraphEditQueue;
use crate::ui::controller::NodeGraphController;
use crate::ui::style::NodeGraphStyle;

use super::layout_hidden_child_and_release_focus;
use super::rename_host_event::{
    apply_rename_host_key_decision, close_rename_host_sessions, decide_rename_host_key,
};
use super::rename_host_layout::{RenameHostLayoutPlan, plan_rename_host_layout};
use super::rename_policy::{
    RenameOverlaySession, RenameOverlaySessionKey, active_rename_session, rename_session_seed_text,
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

    fn active_rename_session<H: UiHost>(&self, host: &H) -> Option<RenameOverlaySession> {
        self.overlays
            .read_ref(host, active_rename_session)
            .ok()
            .flatten()
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

        let fret_core::Event::KeyDown { key, .. } = event else {
            return;
        };

        let decision = decide_rename_host_key(*key);
        if apply_rename_host_key_decision(
            cx.app,
            decision,
            &self.graph,
            &self.group_rename_text,
            &self.overlays,
            &session,
            self.controller.as_ref(),
            self.edits.as_ref(),
        ) {
            crate::ui::retained_event_tail::focus_canvas_and_finish_layout_event(
                cx,
                self.canvas_node,
            );
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.overlays, Invalidation::Layout);
        cx.observe_model(&self.graph, Invalidation::Layout);
        cx.observe_model(&self.group_rename_text, Invalidation::Layout);

        let child = cx.children.get(0).copied();
        let session = self.active_rename_session(&*cx.app);
        self.active = session.is_some();

        match plan_rename_host_layout(
            &self.style,
            cx.bounds,
            session.as_ref(),
            child,
            cx.focus,
            self.last_opened_session,
        ) {
            RenameHostLayoutPlan::CancelActiveSession => {
                close_rename_host_sessions(cx.app, &self.overlays);
                self.last_opened_session = None;
                self.rename_bounds = None;
                self.active = false;
                if let Some(child) = child {
                    layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
                }
            }
            RenameHostLayoutPlan::Active {
                rect,
                session_key,
                just_opened,
            } => {
                let session = session.expect("active rename session for active rename layout plan");
                self.active = true;
                self.last_opened_session = Some(session_key);
                if just_opened {
                    let seed_text = self
                        .graph
                        .read_ref(cx.app, |g| rename_session_seed_text(g, &session))
                        .ok()
                        .unwrap_or_default();
                    let _ = self.group_rename_text.update(cx.app, |t, _cx| {
                        *t = seed_text;
                    });
                }
                if let Some(child) = child {
                    if just_opened {
                        cx.tree.set_focus(Some(child));
                    }
                    cx.layout_in(child, rect);
                }
                self.rename_bounds = Some(rect);
            }
            RenameHostLayoutPlan::Hidden => {
                self.last_opened_session = None;
                self.rename_bounds = None;
                self.active = false;
                if let Some(child) = child {
                    layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
                }
            }
        }

        cx.bounds.size
    }
}
