//! Portable render-facing contract types.
//!
//! This crate intentionally contains only small, backend-agnostic data types that are useful for
//! multiple renderer implementations (wgpu today; future backends later).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderTargetColorSpace {
    Srgb,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderTargetAlphaMode {
    Premultiplied,
    Straight,
}

impl Default for RenderTargetAlphaMode {
    fn default() -> Self {
        Self::Premultiplied
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderTargetRotation {
    R0,
    R90,
    R180,
    R270,
}

impl Default for RenderTargetRotation {
    fn default() -> Self {
        Self::R0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RenderTargetOrientation {
    pub rotation: RenderTargetRotation,
    pub mirror_x: bool,
}

impl Default for RenderTargetOrientation {
    fn default() -> Self {
        Self {
            rotation: RenderTargetRotation::R0,
            mirror_x: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RenderTargetMetadata {
    pub alpha_mode: RenderTargetAlphaMode,
    pub orientation: RenderTargetOrientation,

    /// Optional frame timestamp hint for diagnostics/telemetry, in monotonic nanoseconds.
    ///
    /// This should not be used for UI logic. It exists to help debug real import paths
    /// (e.g. platform video frames) where timing information is useful to attribute stutter.
    pub frame_timestamp_ns: Option<u64>,
}

impl Default for RenderTargetMetadata {
    fn default() -> Self {
        Self {
            alpha_mode: RenderTargetAlphaMode::Premultiplied,
            orientation: RenderTargetOrientation::default(),
            frame_timestamp_ns: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_target_color_space_serializes_as_snake_case() {
        let srgb = serde_json::to_string(&RenderTargetColorSpace::Srgb).expect("serialize srgb");
        let linear =
            serde_json::to_string(&RenderTargetColorSpace::Linear).expect("serialize linear");

        assert_eq!(srgb, "\"srgb\"");
        assert_eq!(linear, "\"linear\"");
    }

    #[test]
    fn render_target_alpha_mode_serializes_as_snake_case() {
        let premul =
            serde_json::to_string(&RenderTargetAlphaMode::Premultiplied).expect("serialize premul");
        let straight =
            serde_json::to_string(&RenderTargetAlphaMode::Straight).expect("serialize straight");

        assert_eq!(premul, "\"premultiplied\"");
        assert_eq!(straight, "\"straight\"");
    }

    #[test]
    fn render_target_orientation_is_defaultable_and_serializable() {
        let o = RenderTargetOrientation::default();
        assert_eq!(o.rotation, RenderTargetRotation::R0);
        assert!(!o.mirror_x);

        let json = serde_json::to_string(&o).expect("serialize orientation");
        assert_eq!(json, "{\"rotation\":\"r0\",\"mirror_x\":false}");
    }
}
