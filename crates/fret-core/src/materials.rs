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

/// Renderer-owned catalog textures that sampled materials may bind (ADR 0242).
///
/// These are framework-controlled identifiers, not backend handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialCatalogTextureKind {
    /// A small baked noise texture (nominally “blue noise”).
    BlueNoise64x64R8,
    /// A repeated 8x8 Bayer dither matrix.
    Bayer8x8R8,
}

/// Fixed and versioned material binding shapes.
///
/// v1 materials are params-only; v2 introduces a renderer-owned catalog texture bind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialBindingShape {
    /// v1: fixed-size `MaterialParams` only.
    ParamsOnly,
    /// v2: `MaterialParams` + one renderer-owned catalog texture + one sampler (ADR 0242).
    ParamsPlusCatalogTexture { texture: MaterialCatalogTextureKind },
}

impl Default for MaterialBindingShape {
    fn default() -> Self {
        Self::ParamsOnly
    }
}

/// Backend-agnostic descriptor used to register a material pipeline.
///
/// v1: descriptors are intentionally small and fixed to keep the surface controlled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialDescriptor {
    pub kind: MaterialKind,
    #[serde(default)]
    pub binding: MaterialBindingShape,
}

impl MaterialDescriptor {
    pub const fn new(kind: MaterialKind) -> Self {
        Self {
            kind,
            binding: MaterialBindingShape::ParamsOnly,
        }
    }

    pub const fn sampled_with_catalog_texture(
        kind: MaterialKind,
        texture: MaterialCatalogTextureKind,
    ) -> Self {
        Self {
            kind,
            binding: MaterialBindingShape::ParamsPlusCatalogTexture { texture },
        }
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
