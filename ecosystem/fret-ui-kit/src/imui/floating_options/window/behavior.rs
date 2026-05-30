/// Behavior flags for in-window floating windows.
///
/// This is an ecosystem-level facade surface (not a mechanism-layer contract). The goal is to
/// provide ImGui-like control over common floating window behavior without introducing a parallel
/// runtime or duplicating canonical policy.
#[derive(Debug, Clone, Copy)]
pub struct FloatingWindowOptions {
    /// When true, the window can be moved by dragging the title bar.
    pub movable: bool,
    /// When true, resize handles are active when the window is rendered with an initial size.
    pub resizable: bool,
    /// When true, title-bar double click toggles collapse/expand.
    pub collapsible: bool,
    /// When true and an `open` model is provided, the close button and `Escape`-to-close are enabled.
    pub closable: bool,
    /// When true, pointer down inside the window requests focus for the surface (even if
    /// activation is disabled).
    ///
    /// This is useful to model ImGui's `NoBringToFrontOnFocus` behavior: you may want a window to
    /// take focus when clicked without also being activated for z-order.
    pub focus_on_click: bool,
    /// When true, pointer down anywhere in the window activates it for z-order (when nested under
    /// `floating_layer(...)`).
    pub activate_on_click: bool,
    /// When false, the window is rendered but pointer interactions are blocked (no activation,
    /// drag, resize, or child clicks).
    pub inputs_enabled: bool,
    /// When true, the window is rendered but is inert for pointer and keyboard navigation:
    /// it does not participate in pointer hit-testing and is skipped by focus traversal.
    ///
    /// This is intended to model Dear ImGui's `NoInputs` window flag, which implies mouse
    /// pass-through and disables nav/focus participation.
    ///
    /// Note: `no_inputs=true` is different from `inputs_enabled=false`:
    /// - `inputs_enabled=false` blocks pointer hits (not click-through) but still participates
    ///   in focus traversal.
    /// - `no_inputs=true` is click-through and is skipped by focus traversal.
    pub no_inputs: bool,
    /// When true, the floating window is hit-test transparent (pointer events pass through to
    /// underlay content).
    ///
    /// This is intended to model Dear ImGui's "mouse pass-through" style behavior (`NoMouseInputs`
    /// for in-window floating surfaces. In Fret's current facade, this is pointer pass-through
    /// only: the subtree remains present for focus traversal / keyboard navigation.
    ///
    /// Note: `inputs_enabled=false` is *not* click-through; it is "non-interactive but blocks
    /// pointer hits". Use `pointer_passthrough=true` when you explicitly want click-through.
    pub pointer_passthrough: bool,
}

impl Default for FloatingWindowOptions {
    fn default() -> Self {
        Self {
            movable: true,
            resizable: true,
            collapsible: true,
            closable: true,
            focus_on_click: true,
            activate_on_click: true,
            inputs_enabled: true,
            no_inputs: false,
            pointer_passthrough: false,
        }
    }
}
