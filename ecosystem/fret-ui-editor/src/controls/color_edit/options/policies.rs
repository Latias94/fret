/// Alpha preview policy for `ColorEdit` swatches.
///
/// Dear ImGui exposes this as `AlphaOpaque`, `AlphaNoBg`, and `AlphaPreviewHalf` flags on
/// `ColorButton` / `ColorEdit`. Fret keeps it as explicit per-control editor policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorEditAlphaPreview {
    /// Show transparent colors over a checkerboard background.
    Checkerboard,
    /// Show the current RGB channels as fully opaque in preview only.
    Opaque,
    /// Show the color with its real alpha without a checkerboard background.
    NoBackground,
    /// Split the preview between opaque RGB and transparent checkerboard-backed RGB.
    Half,
}

impl Default for ColorEditAlphaPreview {
    fn default() -> Self {
        Self::Checkerboard
    }
}

/// Per-control color drag/drop policy for editor `ColorEdit`.
///
/// Dear ImGui enables color drag/drop by default and uses `NoDragDrop` as the opt-out flag. Fret
/// keeps the same default for local editor payloads while making cross-window routing explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEditDragDropOptions {
    pub enabled: bool,
    pub cross_window: bool,
}

impl Default for ColorEditDragDropOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            cross_window: false,
        }
    }
}

/// Hover tooltip policy for editor `ColorEdit` preview swatches.
///
/// Dear ImGui exposes this as `ImGuiColorEditFlags_NoTooltip`. Fret keeps it as explicit
/// per-control editor policy and avoids global color-edit option state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEditTooltipOptions {
    pub enabled: bool,
}

impl Default for ColorEditTooltipOptions {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Context-menu copy policy for editor `ColorEdit` preview swatches.
///
/// Dear ImGui exposes `Copy as..` inside `ColorEditOptionsPopup()`. Fret keeps the behavior local
/// to the editor control and writes through the existing clipboard effect boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEditCopyOptions {
    pub enabled: bool,
}

impl Default for ColorEditCopyOptions {
    fn default() -> Self {
        Self { enabled: true }
    }
}
