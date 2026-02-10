use crate::MaterialId;
use serde::{Deserialize, Serialize};

/// Framework-controlled material kinds intended for lightweight stylization (Tier B).
///
/// These are portable and backend-agnostic. Renderers may capability-gate registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialKind {
    DotGrid,
    Grid,
    Checkerboard,
    Stripe,
    Noise,
    Beam,
    Sparkle,
    ConicSweep,
}

/// Backend-agnostic descriptor used to register a material pipeline.
///
/// v1: descriptors are intentionally small and fixed to keep the surface controlled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialDescriptor {
    pub kind: MaterialKind,
}

impl MaterialDescriptor {
    pub const fn new(kind: MaterialKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialRegistrationError {
    Unsupported,
}

/// Renderer-owned registry for framework-controlled materials.
///
/// This is exposed as a runtime service so components can obtain `MaterialId` handles without
/// receiving backend handles or shader code.
pub trait MaterialService {
    fn register_material(
        &mut self,
        desc: MaterialDescriptor,
    ) -> Result<MaterialId, MaterialRegistrationError>;

    fn unregister_material(&mut self, id: MaterialId) -> bool;
}
