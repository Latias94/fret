use fret_core::{Event, KeyCode, MouseButton, NodeId, Rect, UiServices};
use fret_runtime::CommandId;
use fret_ui::widget::EventCx;
use fret_ui::{
    UiHost,
    tree::{UiLayerId, UiTree},
};

#[derive(Debug, Clone, Copy)]
pub enum OverlayFocusTarget {
    None,
    Root,
    Node(NodeId),
    FirstFocusableDescendant { root: NodeId, fallback: NodeId },
}

impl OverlayFocusTarget {
    fn resolve<H: UiHost>(&self, ui: &UiTree<H>, portal_root: NodeId) -> Option<NodeId> {
        match *self {
            Self::None => None,
            Self::Root => Some(portal_root),
            Self::Node(id) => Some(id),
            Self::FirstFocusableDescendant { root, fallback } => {
                ui.first_focusable_descendant(root).or(Some(fallback))
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ModalFocusScope {
    root: NodeId,
    owned_portals: Vec<NodeId>,
}

impl ModalFocusScope {
    pub fn new(root: NodeId) -> Self {
        Self {
            root,
            owned_portals: Vec::new(),
        }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn allow_portal(&mut self, root: NodeId) {
        if root == self.root {
            return;
        }
        if self.owned_portals.contains(&root) {
            return;
        }
        self.owned_portals.push(root);
    }

    pub fn disallow_portal(&mut self, root: NodeId) {
        self.owned_portals.retain(|r| *r != root);
    }

    pub fn traversal_roots_in_ui<H: UiHost>(&self, ui: &UiTree<H>) -> Vec<NodeId> {
        let mut out = Vec::with_capacity(1 + self.owned_portals.len());
        out.push(Self::effective_traversal_root(ui, self.root));
        out.extend(
            self.owned_portals
                .iter()
                .copied()
                .map(|r| Self::effective_traversal_root(ui, r)),
        );
        out
    }

    fn effective_traversal_root<H: UiHost>(ui: &UiTree<H>, root: NodeId) -> NodeId {
        ui.children(root).first().copied().unwrap_or(root)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OverlayPortal {
    pub layer: UiLayerId,
    pub root: NodeId,
    pub cleanup_on_close: bool,
    previous_focus: Option<NodeId>,
}

impl OverlayPortal {
    pub fn install<H: UiHost>(
        ui: &mut UiTree<H>,
        widget: impl fret_ui::widget::Widget<H> + 'static,
        blocks_underlay_input: bool,
        hit_testable: bool,
        visible: bool,
    ) -> Self {
        let root = ui.create_node(widget);
        let layer = ui.push_overlay_root_ex(root, blocks_underlay_input, hit_testable);
        ui.set_layer_visible(layer, visible);
        Self {
            layer,
            root,
            cleanup_on_close: true,
            previous_focus: None,
        }
    }

    pub fn is_visible<H: UiHost>(&self, ui: &UiTree<H>) -> bool {
        ui.is_layer_visible(self.layer)
    }

    pub fn show<H: UiHost>(&mut self, ui: &mut UiTree<H>) {
        if self.is_visible(ui) {
            return;
        }
        self.previous_focus = ui.focus();
        ui.set_layer_visible(self.layer, true);
    }

    pub fn show_with_focus<H: UiHost>(&mut self, ui: &mut UiTree<H>, focus: OverlayFocusTarget) {
        let was_visible = self.is_visible(ui);
        self.show(ui);
        if was_visible {
            return;
        }

        if let Some(focus) = focus.resolve(ui, self.root) {
            ui.set_focus(Some(focus));
        }
    }

    pub fn hide<H: UiHost>(&mut self, ui: &mut UiTree<H>, services: &mut dyn UiServices) {
        if !self.is_visible(ui) {
            return;
        }

        if self.cleanup_on_close {
            ui.cleanup_subtree(services, self.root);
        }
        ui.set_layer_visible(self.layer, false);

        // Only restore focus if it was inside this overlay (or missing). If focus already moved
        // elsewhere (e.g. click-through outside press), do not override it.
        let should_restore = match ui.focus() {
            None => true,
            Some(focus) => ui.node_layer(focus) == Some(self.layer),
        };

        if should_restore && let Some(prev) = self.previous_focus.take() {
            ui.set_focus(Some(prev));
        }
    }

    pub fn clear_previous_focus(&mut self) {
        self.previous_focus = None;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeDismissModifiers {
    /// Always dismiss on Escape regardless of modifiers.
    Any,
    /// Only dismiss when no modifiers are pressed.
    None,
    /// Dismiss when `ctrl/meta/alt/alt_gr` are not pressed (Shift is allowed).
    NoCtrlAltMeta,
}

#[derive(Debug, Clone)]
pub struct DismissOnEscapeAndClickOutside {
    pub close_command: CommandId,
    pub escape_modifiers: EscapeDismissModifiers,
    pub stop_propagation: bool,
}

impl DismissOnEscapeAndClickOutside {
    pub fn new(close_command: CommandId) -> Self {
        Self {
            close_command,
            escape_modifiers: EscapeDismissModifiers::NoCtrlAltMeta,
            stop_propagation: true,
        }
    }

    pub fn escape_modifiers(mut self, rule: EscapeDismissModifiers) -> Self {
        self.escape_modifiers = rule;
        self
    }

    pub fn stop_propagation(mut self, stop: bool) -> Self {
        self.stop_propagation = stop;
        self
    }

    pub fn handle_event<H: UiHost>(
        &self,
        cx: &mut EventCx<'_, H>,
        event: &Event,
        panel_bounds: Rect,
        close_on_escape: bool,
        close_on_click_outside: bool,
    ) -> bool {
        if !self.should_dismiss(event, panel_bounds, close_on_escape, close_on_click_outside) {
            return false;
        }

        cx.dispatch_command(self.close_command.clone());
        if self.stop_propagation {
            cx.stop_propagation();
        }
        true
    }

    pub fn should_dismiss(
        &self,
        event: &Event,
        panel_bounds: Rect,
        close_on_escape: bool,
        close_on_click_outside: bool,
    ) -> bool {
        match event {
            Event::KeyDown { key, modifiers, .. } => {
                if !close_on_escape || *key != KeyCode::Escape {
                    return false;
                }

                let allow = match self.escape_modifiers {
                    EscapeDismissModifiers::Any => true,
                    EscapeDismissModifiers::None => {
                        !modifiers.shift
                            && !modifiers.ctrl
                            && !modifiers.alt
                            && !modifiers.alt_gr
                            && !modifiers.meta
                    }
                    EscapeDismissModifiers::NoCtrlAltMeta => {
                        !modifiers.ctrl && !modifiers.alt && !modifiers.alt_gr && !modifiers.meta
                    }
                };

                if !allow {
                    return false;
                }

                true
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position, button, ..
            }) => {
                if !close_on_click_outside || *button != MouseButton::Left {
                    return false;
                }
                if panel_bounds.contains(*position) {
                    return false;
                }
                true
            }
            _ => false,
        }
    }
}
