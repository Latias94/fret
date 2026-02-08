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
