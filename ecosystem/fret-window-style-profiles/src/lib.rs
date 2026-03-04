//! Ecosystem-level window style profiles (recipes).
//!
//! These helpers are intentionally *not* part of the portable contract surface. They provide a
//! convenient way to apply consistent `WindowStyleRequest` presets in apps while keeping the
//! underlying window-style facets orthogonal and diagnosable.

use fret_runtime::runner_window_style_diagnostics::clamp_background_material_request;
use fret_runtime::{
    ActivationPolicy, PlatformCapabilities, RunnerWindowStyleDiagnosticsStore, TaskbarVisibility,
    WindowBackgroundMaterialRequest, WindowDecorationsRequest, WindowHitTestRequestV1,
    WindowOpacity, WindowStyleRequest, WindowZLevel,
};

#[derive(Debug, Clone, PartialEq)]
pub struct WindowStyleProfileExpectationsV1 {
    /// Expected effective/clamped hit-test policy when `style.hit_test` is set.
    pub hit_test: Option<WindowHitTestRequestV1>,
    /// Stable fingerprint for effective `PassthroughRegions`, if any.
    pub hit_test_regions_fingerprint64: Option<u64>,
    /// Expected effective/clamped background material when `style.background_material` is set.
    pub background_material: Option<WindowBackgroundMaterialRequest>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledWindowStyleProfileV1 {
    pub style: WindowStyleRequest,
    pub expectations: WindowStyleProfileExpectationsV1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HudOverlayOptionsV1 {
    /// Prefer a frameless window by default for overlay-style surfaces.
    pub decorations: WindowDecorationsRequest,
    /// Request non-activating behavior (best-effort; capability-gated).
    pub non_activating: bool,
    /// Request always-on-top (best-effort; capability-gated).
    pub always_on_top: bool,
    /// Preferred OS background materials in priority order.
    ///
    /// Notes:
    /// - This only selects the `background_material` facet.
    /// - Whether the window is visually transparent remains a renderer/content decision; use
    ///   `transparent` explicitly if you want a composited surface regardless of material.
    pub background_material_preference: Vec<WindowBackgroundMaterialRequest>,
    /// Optional global window opacity hint.
    pub opacity: Option<WindowOpacity>,
}

impl Default for HudOverlayOptionsV1 {
    fn default() -> Self {
        Self {
            decorations: WindowDecorationsRequest::None,
            non_activating: true,
            always_on_top: true,
            background_material_preference: vec![
                WindowBackgroundMaterialRequest::SystemDefault,
                WindowBackgroundMaterialRequest::Mica,
                WindowBackgroundMaterialRequest::Acrylic,
                WindowBackgroundMaterialRequest::Vibrancy,
            ],
            opacity: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClickThroughOverlayOptionsV1 {
    pub hud: HudOverlayOptionsV1,
    /// Regions (client logical pixels) that should remain interactive.
    pub interactive_regions: Vec<fret_runtime::WindowHitTestRegionV1>,
}

fn pick_background_material(
    preference: &[WindowBackgroundMaterialRequest],
    caps: &PlatformCapabilities,
) -> Option<WindowBackgroundMaterialRequest> {
    for &want in preference {
        let clamped = clamp_background_material_request(want, caps);
        if clamped != WindowBackgroundMaterialRequest::None {
            return Some(clamped);
        }
    }
    None
}

fn expectations_from_style(
    style: &WindowStyleRequest,
    caps: &PlatformCapabilities,
) -> WindowStyleProfileExpectationsV1 {
    let (hit_test, hit_test_regions_fingerprint64) = match style.hit_test.clone() {
        None => (None, None),
        Some(req) => {
            let (effective, _reason) =
                RunnerWindowStyleDiagnosticsStore::clamp_hit_test_request(req, caps);
            let fp = match &effective {
                WindowHitTestRequestV1::PassthroughRegions { regions } => Some(
                    fret_runtime::hit_test_regions_signature_v1(regions)
                        .1
                        .fingerprint64,
                ),
                _ => None,
            };
            (Some(effective), fp)
        }
    };

    let background_material = style
        .background_material
        .map(|m| clamp_background_material_request(m, caps));

    WindowStyleProfileExpectationsV1 {
        hit_test,
        hit_test_regions_fingerprint64,
        background_material,
    }
}

/// Standard, interactive app window.
pub fn app_window_profile_v1(caps: &PlatformCapabilities) -> CompiledWindowStyleProfileV1 {
    let style = WindowStyleRequest {
        decorations: Some(WindowDecorationsRequest::System),
        activation: Some(ActivationPolicy::Activates),
        taskbar: Some(TaskbarVisibility::Show),
        z_level: Some(WindowZLevel::Normal),
        hit_test: Some(WindowHitTestRequestV1::Normal),
        ..Default::default()
    };
    let expectations = expectations_from_style(&style, caps);
    CompiledWindowStyleProfileV1 {
        style,
        expectations,
    }
}

/// Auxiliary/tool window posture (best-effort). Intended for panels and secondary windows.
pub fn tool_window_profile_v1(caps: &PlatformCapabilities) -> CompiledWindowStyleProfileV1 {
    let style = WindowStyleRequest {
        taskbar: Some(TaskbarVisibility::Hide),
        activation: Some(ActivationPolicy::Activates),
        ..Default::default()
    };
    let expectations = expectations_from_style(&style, caps);
    CompiledWindowStyleProfileV1 {
        style,
        expectations,
    }
}

/// HUD-style overlay posture (best-effort, capability-gated).
pub fn hud_overlay_profile_v1(
    caps: &PlatformCapabilities,
    options: HudOverlayOptionsV1,
) -> CompiledWindowStyleProfileV1 {
    let material = pick_background_material(&options.background_material_preference, caps);

    let style = WindowStyleRequest {
        decorations: Some(options.decorations),
        activation: Some(if options.non_activating {
            ActivationPolicy::NonActivating
        } else {
            ActivationPolicy::Activates
        }),
        z_level: Some(if options.always_on_top {
            WindowZLevel::AlwaysOnTop
        } else {
            WindowZLevel::Normal
        }),
        background_material: material,
        opacity: options.opacity,
        ..Default::default()
    };
    let expectations = expectations_from_style(&style, caps);
    CompiledWindowStyleProfileV1 {
        style,
        expectations,
    }
}

/// Click-through overlay window:
/// - passthrough by default,
/// - interactive within the provided regions union (ADR 0313).
pub fn click_through_overlay_profile_v1(
    caps: &PlatformCapabilities,
    options: ClickThroughOverlayOptionsV1,
) -> CompiledWindowStyleProfileV1 {
    let mut base = hud_overlay_profile_v1(caps, options.hud);
    base.style.hit_test = Some(WindowHitTestRequestV1::PassthroughRegions {
        regions: options.interactive_regions,
    });
    base.expectations = expectations_from_style(&base.style, caps);
    base
}
