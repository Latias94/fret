use fret_core::{Modifiers, Px};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockingInteractionSettings {
    pub drag_inversion: DockDragInversionSettings,
    /// Drag activation threshold for docking tab drags (window-local logical pixels).
    pub tab_drag_threshold: Px,
    /// Drag distance threshold for suppressing viewport right-click context menus (screen px).
    ///
    /// Docking forwards viewport pointer input via `Effect::ViewportInput` and uses pointer capture
    /// to keep delivering events when the cursor leaves the viewport bounds.
    ///
    /// For editor-like viewports, it is common to support both:
    /// - right-click context menus (when the pointer does not move), and
    /// - right-drag navigation (orbit/pan) without a context menu on release.
    ///
    /// When this value is exceeded during a right-button viewport capture, docking will suppress
    /// bubbling on the matching `PointerUp` so context-menu primitives upstream do not trigger.
    pub viewport_context_menu_drag_threshold: Px,
    /// When true, suppress bubbling of secondary right-click events while a viewport capture
    /// session is active (e.g. during a left-drag marquee).
    pub suppress_context_menu_during_viewport_capture: bool,
}

impl Default for DockingInteractionSettings {
    fn default() -> Self {
        Self {
            drag_inversion: DockDragInversionSettings::default(),
            tab_drag_threshold: Px(6.0),
            viewport_context_menu_drag_threshold: Px(6.0),
            suppress_context_menu_during_viewport_capture: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockDragInversionSettings {
    pub modifier: DockDragInversionModifier,
    pub policy: DockDragInversionPolicy,
}

impl Default for DockDragInversionSettings {
    fn default() -> Self {
        Self {
            modifier: DockDragInversionModifier::Shift,
            policy: DockDragInversionPolicy::DockByDefault,
        }
    }
}

impl DockDragInversionSettings {
    pub fn wants_dock_previews(self, modifiers: Modifiers) -> bool {
        let modifier_down = self.modifier.is_down(modifiers);
        match self.policy {
            DockDragInversionPolicy::DockByDefault => !modifier_down,
            DockDragInversionPolicy::DockOnlyWhenModifier => modifier_down,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockDragInversionModifier {
    None,
    Shift,
    Ctrl,
    Alt,
    AltGr,
    Meta,
}

impl DockDragInversionModifier {
    pub fn is_down(self, modifiers: Modifiers) -> bool {
        match self {
            Self::None => false,
            Self::Shift => modifiers.shift,
            Self::Ctrl => modifiers.ctrl,
            Self::Alt => modifiers.alt,
            Self::AltGr => modifiers.alt_gr,
            Self::Meta => modifiers.meta,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockDragInversionPolicy {
    DockByDefault,
    DockOnlyWhenModifier,
}
