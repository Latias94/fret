//! Explicit advanced renderer/material helpers for `fret-ui-magic`.
//!
//! This module exists to keep `fret-ui-magic` recipes on the component surface while still
//! providing an explicit escape hatch for app/driver code that has access to renderer-backed
//! services.
//!
//! These helpers are not part of the default app-author path: they belong in runner or driver
//! hooks that can reach `MaterialService`.

use fret_app::App;
use fret_core::{MaterialDescriptor, MaterialKind, MaterialService};
use fret_ui_kit::recipes::catalog::VisualCatalog;

/// Ensure the baseline `fret-ui-magic` materials exist in the app-owned `VisualCatalog`.
///
/// Call this from an explicit runner/driver hook that has access to the renderer-backed
/// `MaterialService` (for example `WinitAppDriver::gpu_ready`).
pub fn ensure_materials(app: &mut App, material_service: &mut dyn MaterialService) {
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
