//! Optional `fret-app` integration helpers for `fret-ui-magic`.
//!
//! This module exists to keep `fret-ui-magic` components ecosystem-only while still providing an
//! ergonomic way for `fret-app`-based binaries to:
//! - register the required renderer-controlled materials (Tier B),
//! - cache the resulting `MaterialId`s in an app-owned catalog (ADR 0245),
//! - and keep component authoring surfaces free of backend handles.

use fret_app::App;
use fret_core::{MaterialDescriptor, MaterialKind, MaterialService};
use fret_ui_kit::recipes::catalog::VisualCatalog;

/// Register the baseline materials used by `fret-ui-magic` Phase 0 and cache them in the app-owned
/// `VisualCatalog`.
///
/// This should be called from a runner/driver hook that has access to the renderer (e.g.
/// `WinitAppDriver::gpu_ready`).
pub fn ensure_magic_materials(app: &mut App, material_service: &mut dyn MaterialService) {
    app.with_global_mut_untracked(VisualCatalog::default, |cat, _app| {
        // Patterns (Phase 0).
        let _ = cat.materials.get_or_register(
            material_service,
            MaterialDescriptor::new(MaterialKind::DotGrid),
        );
        let _ = cat.materials.get_or_register(
            material_service,
            MaterialDescriptor::new(MaterialKind::Grid),
        );
        let _ = cat.materials.get_or_register(
            material_service,
            MaterialDescriptor::new(MaterialKind::Stripe),
        );
        let _ = cat.materials.get_or_register(
            material_service,
            MaterialDescriptor::new(MaterialKind::Checkerboard),
        );

        // Future Phase 0 targets.
        let _ = cat.materials.get_or_register(
            material_service,
            MaterialDescriptor::new(MaterialKind::Noise),
        );
        let _ = cat.materials.get_or_register(
            material_service,
            MaterialDescriptor::new(MaterialKind::Sparkle),
        );
    });
}
