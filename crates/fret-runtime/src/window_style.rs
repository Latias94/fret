use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowRole {
    Main,
    #[default]
    Auxiliary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskbarVisibility {
    Show,
    Hide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationPolicy {
    Activates,
    NonActivating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowZLevel {
    Normal,
    AlwaysOnTop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MousePolicy {
    Normal,
    /// Request click-through / mouse passthrough behavior for the OS window (best-effort).
    Passthrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowHitTestRequestV1 {
    /// Normal OS hit testing (default).
    Normal,
    /// Window ignores pointer hit testing (click-through).
    PassthroughAll,
}

/// Global window opacity hint (best-effort).
///
/// This is not per-pixel transparency. The value is expressed as an 8-bit alpha where:
/// - `0` = fully transparent (may be treated as hidden on some platforms),
/// - `255` = fully opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowOpacity(pub u8);

impl WindowOpacity {
    pub fn from_f32(opacity: f32) -> Self {
        let a = opacity.clamp(0.0, 1.0);
        let byte = (255.0 * a).round().clamp(0.0, 255.0) as u8;
        Self(byte)
    }

    pub fn as_f32(self) -> f32 {
        (self.0 as f32) / 255.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowDecorationsRequest {
    /// Platform default decorations.
    System,
    /// Request a frameless window (client-drawn).
    None,
    /// Request server-side decorations (Wayland only; best-effort).
    Server,
    /// Request client-side decorations (Wayland only; best-effort).
    Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowBackgroundMaterialRequest {
    /// Explicitly disable OS-provided background materials (opaque/default backdrop).
    None,
    /// Request platform default material for a utility window class, if any.
    SystemDefault,
    /// Windows 11-style Mica (best-effort).
    Mica,
    /// Acrylic/blurred translucent backdrop (best-effort).
    Acrylic,
    /// macOS vibrancy-style backdrop (best-effort).
    Vibrancy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowStyleRequest {
    pub taskbar: Option<TaskbarVisibility>,
    pub activation: Option<ActivationPolicy>,
    pub z_level: Option<WindowZLevel>,
    pub decorations: Option<WindowDecorationsRequest>,
    pub resizable: Option<bool>,
    /// Requests a transparent composited window background (best-effort).
    pub transparent: Option<bool>,
    /// Optional request for OS-provided background materials (best-effort).
    pub background_material: Option<WindowBackgroundMaterialRequest>,
    /// Optional request for window-level pointer hit testing (best-effort).
    pub hit_test: Option<WindowHitTestRequestV1>,
    /// Request click-through / mouse passthrough behavior for the OS window (best-effort).
    pub mouse: Option<MousePolicy>,
    /// Request global window opacity (not per-pixel transparency), best-effort.
    pub opacity: Option<WindowOpacity>,
}
