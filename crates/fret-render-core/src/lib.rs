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
}
