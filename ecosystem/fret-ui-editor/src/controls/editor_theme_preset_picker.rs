//! Editor theme preset picker.
//!
//! This is an editor-policy control, not a runtime theme mechanism. It switches between
//! editor-owned preset patches and keeps Dear ImGui-style tuning in the ecosystem layer.

mod options;
mod render;
mod state;
#[cfg(test)]
mod tests;

pub use options::EditorThemePresetPickerOptions;
pub use state::EditorThemePresetPicker;
