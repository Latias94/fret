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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderTargetColorRange {
    #[default]
    Unknown,
    Full,
    Limited,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderTargetColorPrimaries {
    #[default]
    Unknown,
    Bt709,
    DisplayP3,
    Bt2020,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderTargetTransferFunction {
    #[default]
    Unknown,
    Srgb,
    Linear,
    Bt1886,
    Pq,
    Hlg,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderTargetMatrixCoefficients {
    #[default]
    Unknown,
    /// Identity transform for already-RGB content.
    Rgb,
    Bt601,
    Bt709,
    Bt2020Ncl,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RenderTargetColorEncoding {
    pub primaries: RenderTargetColorPrimaries,
    pub transfer: RenderTargetTransferFunction,
    pub matrix: RenderTargetMatrixCoefficients,
    pub range: RenderTargetColorRange,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderTargetAlphaMode {
    #[default]
    Premultiplied,
    Straight,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderTargetIngestStrategy {
    #[default]
    Unknown,
    /// The target is produced on the shared device/queue without importing external handles.
    Owned,
    /// The target is sampled from an external producer without an intermediate copy.
    ExternalZeroCopy,
    /// The target is refreshed via a GPU-side copy/blit into a renderer-owned texture.
    GpuCopy,
    /// The target is refreshed via CPU bytes uploaded into a GPU texture.
    CpuUpload,
}

impl RenderTargetIngestStrategy {
    pub const COUNT: usize = 5;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderTargetRotation {
    #[default]
    R0,
    R90,
    R180,
    R270,
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
    /// Content color encoding hints for real media sources (video/camera/remote desktop).
    ///
    /// These are best-effort hints. Producers should set them when they can, and renderers must
    /// degrade deterministically when the effective ingest strategy cannot preserve them.
    #[serde(default)]
    pub color_encoding: RenderTargetColorEncoding,
    /// Requested ingestion strategy (what the caller wanted).
    ///
    /// This is a diagnostic hint for capability-gated fallback behavior. Renderers may
    /// report when `requested_ingest_strategy` differs from `ingest_strategy`.
    pub requested_ingest_strategy: RenderTargetIngestStrategy,
    pub ingest_strategy: RenderTargetIngestStrategy,

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
            color_encoding: RenderTargetColorEncoding::default(),
            requested_ingest_strategy: RenderTargetIngestStrategy::Unknown,
            ingest_strategy: RenderTargetIngestStrategy::Unknown,
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

    #[test]
    fn render_target_metadata_color_encoding_defaults_when_missing() {
        let json = r#"{
            "alpha_mode":"premultiplied",
            "orientation":{"rotation":"r0","mirror_x":false},
            "requested_ingest_strategy":"unknown",
            "ingest_strategy":"unknown",
            "frame_timestamp_ns":null
        }"#;

        let meta: RenderTargetMetadata =
            serde_json::from_str(json).expect("deserialize metadata without color_encoding");
        assert_eq!(meta.color_encoding, RenderTargetColorEncoding::default());
    }
}
