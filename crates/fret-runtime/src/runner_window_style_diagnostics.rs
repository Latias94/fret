use std::collections::HashMap;

use fret_core::AppWindowId;

use crate::PlatformCapabilities;
use crate::window_style::{
    ActivationPolicy, MousePolicy, TaskbarVisibility, WindowBackgroundMaterialRequest,
    WindowDecorationsRequest, WindowHitTestRequestV1, WindowStyleRequest, WindowZLevel,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerWindowCompositedAlphaSourceV1 {
    /// The runner does not support composited alpha windows.
    Unavailable,
    /// The caller explicitly requested `transparent=true`.
    ExplicitTrue,
    /// The caller explicitly requested `transparent=false`.
    ExplicitFalse,
    /// The window was created composited because a non-None backdrop material was requested.
    ImpliedByMaterialCreateTime,
    /// The caller omitted `transparent` and no implied material required composition.
    DefaultOpaque,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerWindowAppearanceV1 {
    /// The OS window is not composited with alpha.
    Opaque,
    /// The OS window surface is composited with alpha, but no OS backdrop material is enabled.
    CompositedNoBackdrop,
    /// The OS window surface is composited with alpha and an OS backdrop material is enabled.
    CompositedBackdrop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerWindowHitTestSourceV1 {
    /// No explicit request (defaults apply).
    Default,
    /// The caller explicitly requested `hit_test=...`.
    HitTestFacet,
    /// Legacy request via `mouse=Passthrough`.
    LegacyMousePolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerWindowHitTestClampReasonV1 {
    None,
    MissingPassthroughAllCapability,
    MissingPassthroughRegionsCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunnerWindowStyleEffectiveSnapshotV1 {
    pub decorations: WindowDecorationsRequest,
    pub resizable: bool,
    /// Whether the OS window surface is composited with alpha (create-time; may be sticky).
    pub surface_composited_alpha: bool,
    /// Why the surface is (or is not) composited.
    pub surface_composited_alpha_source: RunnerWindowCompositedAlphaSourceV1,
    /// Whether the runner will preserve alpha by default (clear alpha = 0) for this window.
    ///
    /// This is a visual policy decision used by the runner+renderer. It is intentionally
    /// separated from `surface_composited_alpha` to avoid conflating "window can be composited"
    /// with "window is visually transparent".
    pub visual_transparent: bool,
    /// A derived, user-facing summary of window background appearance facets.
    pub appearance: RunnerWindowAppearanceV1,
    pub background_material: WindowBackgroundMaterialRequest,
    /// Effective window hit test policy (pointer passthrough).
    pub hit_test: WindowHitTestRequestV1,
    /// Last requested hit test policy (pre-clamp), if any.
    pub hit_test_requested: Option<WindowHitTestRequestV1>,
    pub hit_test_source: RunnerWindowHitTestSourceV1,
    pub hit_test_clamp_reason: RunnerWindowHitTestClampReasonV1,
    pub taskbar: TaskbarVisibility,
    pub activation: ActivationPolicy,
    pub z_level: WindowZLevel,
    pub mouse: MousePolicy,
}

impl Default for RunnerWindowStyleEffectiveSnapshotV1 {
    fn default() -> Self {
        Self {
            decorations: WindowDecorationsRequest::System,
            resizable: true,
            surface_composited_alpha: false,
            surface_composited_alpha_source: RunnerWindowCompositedAlphaSourceV1::DefaultOpaque,
            visual_transparent: false,
            appearance: RunnerWindowAppearanceV1::Opaque,
            background_material: WindowBackgroundMaterialRequest::None,
            hit_test: WindowHitTestRequestV1::Normal,
            hit_test_requested: None,
            hit_test_source: RunnerWindowHitTestSourceV1::Default,
            hit_test_clamp_reason: RunnerWindowHitTestClampReasonV1::None,
            taskbar: TaskbarVisibility::Show,
            activation: ActivationPolicy::Activates,
            z_level: WindowZLevel::Normal,
            mouse: MousePolicy::Normal,
        }
    }
}

#[derive(Debug, Default)]
pub struct RunnerWindowStyleDiagnosticsStore {
    effective: HashMap<AppWindowId, RunnerWindowStyleEffectiveSnapshotV1>,
    transparent_explicit: HashMap<AppWindowId, Option<bool>>,
    transparent_implied_by_material_create_time: HashMap<AppWindowId, bool>,
}

impl RunnerWindowStyleDiagnosticsStore {
    fn derive_appearance(
        surface_composited_alpha: bool,
        background_material: WindowBackgroundMaterialRequest,
    ) -> RunnerWindowAppearanceV1 {
        if !surface_composited_alpha {
            return RunnerWindowAppearanceV1::Opaque;
        }
        if background_material != WindowBackgroundMaterialRequest::None {
            return RunnerWindowAppearanceV1::CompositedBackdrop;
        }
        RunnerWindowAppearanceV1::CompositedNoBackdrop
    }

    pub fn clamp_hit_test_request(
        requested: WindowHitTestRequestV1,
        caps: &PlatformCapabilities,
    ) -> (WindowHitTestRequestV1, RunnerWindowHitTestClampReasonV1) {
        match requested {
            WindowHitTestRequestV1::Normal => (
                WindowHitTestRequestV1::Normal,
                RunnerWindowHitTestClampReasonV1::None,
            ),
            WindowHitTestRequestV1::PassthroughAll if caps.ui.window_hit_test_passthrough_all => (
                WindowHitTestRequestV1::PassthroughAll,
                RunnerWindowHitTestClampReasonV1::None,
            ),
            WindowHitTestRequestV1::PassthroughAll => (
                WindowHitTestRequestV1::Normal,
                RunnerWindowHitTestClampReasonV1::MissingPassthroughAllCapability,
            ),
        }
    }

    fn requested_hit_test_from_request(
        requested: WindowStyleRequest,
    ) -> Option<(WindowHitTestRequestV1, RunnerWindowHitTestSourceV1)> {
        if let Some(hit_test) = requested.hit_test {
            return Some((hit_test, RunnerWindowHitTestSourceV1::HitTestFacet));
        }
        if matches!(requested.mouse, Some(MousePolicy::Passthrough)) {
            // Back-compat mapping: window-level mouse passthrough is treated as hit-test passthrough.
            return Some((
                WindowHitTestRequestV1::PassthroughAll,
                RunnerWindowHitTestSourceV1::LegacyMousePolicy,
            ));
        }
        None
    }

    pub fn effective_snapshot(
        &self,
        window: AppWindowId,
    ) -> Option<RunnerWindowStyleEffectiveSnapshotV1> {
        self.effective.get(&window).copied()
    }

    pub fn record_window_open(
        &mut self,
        window: AppWindowId,
        requested: WindowStyleRequest,
        caps: &PlatformCapabilities,
    ) {
        let mut next = RunnerWindowStyleEffectiveSnapshotV1::default();
        self.transparent_explicit
            .insert(window, requested.transparent);
        self.transparent_implied_by_material_create_time
            .insert(window, false);

        if caps.ui.window_decorations {
            if let Some(decorations) = requested.decorations {
                next.decorations = decorations;
            }
        }
        if caps.ui.window_resizable {
            if let Some(resizable) = requested.resizable {
                next.resizable = resizable;
            }
        }
        if let Some(material) = requested.background_material {
            let clamped = clamp_background_material_request(material, caps);
            next.background_material = clamped;
        }

        // Determine whether the surface is composited with alpha (create-time semantics).
        if !caps.ui.window_transparent {
            next.surface_composited_alpha = false;
            next.surface_composited_alpha_source = RunnerWindowCompositedAlphaSourceV1::Unavailable;
        } else if let Some(transparent) = requested.transparent {
            next.surface_composited_alpha = transparent;
            next.surface_composited_alpha_source = if transparent {
                RunnerWindowCompositedAlphaSourceV1::ExplicitTrue
            } else {
                RunnerWindowCompositedAlphaSourceV1::ExplicitFalse
            };
        } else if next.background_material != WindowBackgroundMaterialRequest::None {
            // Background materials may require a composited alpha surface (ADR 0310). If the
            // caller did not explicitly request `transparent`, treat it as implied once a
            // non-None material is effectively applied.
            next.surface_composited_alpha = true;
            next.surface_composited_alpha_source =
                RunnerWindowCompositedAlphaSourceV1::ImpliedByMaterialCreateTime;
            self.transparent_implied_by_material_create_time
                .insert(window, true);
        } else {
            next.surface_composited_alpha = false;
            next.surface_composited_alpha_source =
                RunnerWindowCompositedAlphaSourceV1::DefaultOpaque;
        }

        // Background materials generally require a composited alpha surface. If the window is not
        // composited, degrade any material request to None so the effective snapshot remains
        // achievable.
        if !next.surface_composited_alpha {
            next.background_material = WindowBackgroundMaterialRequest::None;
        }

        // Visual transparency default: preserve alpha when a backdrop material is enabled, or when
        // the caller explicitly requested a composited surface for visual transparency.
        next.visual_transparent = next.background_material != WindowBackgroundMaterialRequest::None
            || matches!(requested.transparent, Some(true));
        next.appearance =
            Self::derive_appearance(next.surface_composited_alpha, next.background_material);

        if let Some((hit_test, source)) = Self::requested_hit_test_from_request(requested) {
            next.hit_test_requested = Some(hit_test);
            let (effective, clamp_reason) = Self::clamp_hit_test_request(hit_test, caps);
            next.hit_test = effective;
            next.hit_test_source = source;
            next.hit_test_clamp_reason = clamp_reason;
        }

        if let Some(taskbar) = requested.taskbar {
            next.taskbar = if taskbar == TaskbarVisibility::Hide && !caps.ui.window_skip_taskbar {
                TaskbarVisibility::Show
            } else {
                taskbar
            };
        }
        if let Some(activation) = requested.activation {
            next.activation = if activation == ActivationPolicy::NonActivating
                && !caps.ui.window_non_activating
            {
                ActivationPolicy::Activates
            } else {
                activation
            };
        }
        if let Some(z_level) = requested.z_level {
            next.z_level = if z_level == WindowZLevel::AlwaysOnTop
                && matches!(caps.ui.window_z_level, crate::WindowZLevelQuality::None)
            {
                WindowZLevel::Normal
            } else {
                z_level
            };
        }
        if let Some(mouse) = requested.mouse {
            next.mouse = if mouse == MousePolicy::Passthrough && !caps.ui.window_mouse_passthrough {
                MousePolicy::Normal
            } else {
                mouse
            };
        }

        self.effective.insert(window, next);
    }

    pub fn record_window_close(&mut self, window: AppWindowId) {
        self.effective.remove(&window);
        self.transparent_explicit.remove(&window);
        self.transparent_implied_by_material_create_time
            .remove(&window);
    }

    pub fn apply_style_patch(
        &mut self,
        window: AppWindowId,
        patch: WindowStyleRequest,
        caps: &PlatformCapabilities,
    ) {
        let Some(current) = self.effective.get_mut(&window) else {
            return;
        };

        // Create-time facets are intentionally ignored for v1 runtime patching.
        // See ADR 0139 for patchability rules.

        if let Some(material) = patch.background_material {
            let next_material = clamp_background_material_request(material, caps);

            // Background materials generally require a composited alpha window surface (ADR 0310).
            // Since composited alpha is create-time in the runner, degrade non-None material
            // requests when the window is not already composited.
            current.background_material = if next_material != WindowBackgroundMaterialRequest::None
                && !current.surface_composited_alpha
            {
                WindowBackgroundMaterialRequest::None
            } else {
                next_material
            };

            // Visual transparency default tracks the effective material, but falls back to the
            // caller's explicit create-time transparency intent.
            let explicit = self.transparent_explicit.get(&window).copied().flatten();
            current.visual_transparent = current.background_material
                != WindowBackgroundMaterialRequest::None
                || matches!(explicit, Some(true));
            current.appearance = Self::derive_appearance(
                current.surface_composited_alpha,
                current.background_material,
            );

            // Keep composited alpha create-time and sticky. If the window was created composited
            // (explicitly or implied by a create-time material request), keep it for the lifetime.
            if !caps.ui.window_transparent {
                current.surface_composited_alpha = false;
                current.surface_composited_alpha_source =
                    RunnerWindowCompositedAlphaSourceV1::Unavailable;
            } else if let Some(explicit) = explicit {
                current.surface_composited_alpha = explicit;
                current.surface_composited_alpha_source = if explicit {
                    RunnerWindowCompositedAlphaSourceV1::ExplicitTrue
                } else {
                    RunnerWindowCompositedAlphaSourceV1::ExplicitFalse
                };
            } else if self
                .transparent_implied_by_material_create_time
                .get(&window)
                .copied()
                .unwrap_or(false)
            {
                current.surface_composited_alpha = true;
                // Once implied at create-time, keep it sticky even if the material is cleared.
                current.surface_composited_alpha_source =
                    RunnerWindowCompositedAlphaSourceV1::ImpliedByMaterialCreateTime;
            } else {
                current.surface_composited_alpha = false;
                current.surface_composited_alpha_source =
                    RunnerWindowCompositedAlphaSourceV1::DefaultOpaque;
            }

            current.appearance = Self::derive_appearance(
                current.surface_composited_alpha,
                current.background_material,
            );
        }

        if let Some(hit_test) = patch.hit_test {
            current.hit_test_requested = Some(hit_test);
            let (effective, clamp_reason) = Self::clamp_hit_test_request(hit_test, caps);
            current.hit_test = effective;
            current.hit_test_source = RunnerWindowHitTestSourceV1::HitTestFacet;
            current.hit_test_clamp_reason = clamp_reason;
        } else if let Some(mouse) = patch.mouse
            && matches!(mouse, MousePolicy::Passthrough)
        {
            let requested = WindowHitTestRequestV1::PassthroughAll;
            current.hit_test_requested = Some(requested);
            let (effective, clamp_reason) = Self::clamp_hit_test_request(requested, caps);
            current.hit_test = effective;
            current.hit_test_source = RunnerWindowHitTestSourceV1::LegacyMousePolicy;
            current.hit_test_clamp_reason = clamp_reason;
        }

        if let Some(taskbar) = patch.taskbar {
            if taskbar == TaskbarVisibility::Hide && !caps.ui.window_skip_taskbar {
                // Ignore unsupported hide requests.
            } else {
                current.taskbar = taskbar;
            }
        }
        if let Some(activation) = patch.activation {
            if activation == ActivationPolicy::NonActivating && !caps.ui.window_non_activating {
                // Ignore unsupported non-activating requests.
            } else {
                current.activation = activation;
            }
        }
        if let Some(z_level) = patch.z_level {
            if z_level == WindowZLevel::AlwaysOnTop
                && matches!(caps.ui.window_z_level, crate::WindowZLevelQuality::None)
            {
                // Ignore unsupported AlwaysOnTop.
            } else {
                current.z_level = z_level;
            }
        }
        if let Some(mouse) = patch.mouse {
            if mouse == MousePolicy::Passthrough && !caps.ui.window_mouse_passthrough {
                // Ignore unsupported passthrough requests.
            } else {
                current.mouse = mouse;
            }
        }
    }
}

pub fn clamp_background_material_request(
    requested: WindowBackgroundMaterialRequest,
    caps: &PlatformCapabilities,
) -> WindowBackgroundMaterialRequest {
    use WindowBackgroundMaterialRequest::*;
    match requested {
        None => None,
        SystemDefault if caps.ui.window_background_material_system_default => SystemDefault,
        Mica if caps.ui.window_background_material_mica => Mica,
        Acrylic if caps.ui.window_background_material_acrylic => Acrylic,
        Vibrancy if caps.ui.window_background_material_vibrancy => Vibrancy,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::KeyData;

    fn window(id: u64) -> AppWindowId {
        AppWindowId::from(KeyData::from_ffi(id))
    }

    #[test]
    fn implied_transparency_stays_sticky_when_material_cleared() {
        let caps = PlatformCapabilities::default();
        let mut store = RunnerWindowStyleDiagnosticsStore::default();
        let w = window(1);

        store.record_window_open(
            w,
            WindowStyleRequest {
                background_material: Some(WindowBackgroundMaterialRequest::Mica),
                ..Default::default()
            },
            &caps,
        );
        let before = store.effective_snapshot(w).unwrap();
        assert!(before.surface_composited_alpha);
        assert_eq!(
            before.background_material,
            WindowBackgroundMaterialRequest::Mica
        );

        store.apply_style_patch(
            w,
            WindowStyleRequest {
                background_material: Some(WindowBackgroundMaterialRequest::None),
                ..Default::default()
            },
            &caps,
        );
        let after = store.effective_snapshot(w).unwrap();
        assert!(after.surface_composited_alpha);
        assert_eq!(
            after.background_material,
            WindowBackgroundMaterialRequest::None
        );
    }

    #[test]
    fn material_request_degrades_when_window_not_composited() {
        let caps = PlatformCapabilities::default();
        let mut store = RunnerWindowStyleDiagnosticsStore::default();
        let w = window(2);

        store.record_window_open(w, WindowStyleRequest::default(), &caps);
        let before = store.effective_snapshot(w).unwrap();
        assert!(!before.surface_composited_alpha);
        assert_eq!(
            before.background_material,
            WindowBackgroundMaterialRequest::None
        );

        store.apply_style_patch(
            w,
            WindowStyleRequest {
                background_material: Some(WindowBackgroundMaterialRequest::Mica),
                ..Default::default()
            },
            &caps,
        );
        let after = store.effective_snapshot(w).unwrap();
        assert!(!after.surface_composited_alpha);
        assert_eq!(
            after.background_material,
            WindowBackgroundMaterialRequest::None
        );
    }

    #[test]
    fn implied_material_transparency_clears_visually_when_material_cleared() {
        let caps = PlatformCapabilities::default();
        let mut store = RunnerWindowStyleDiagnosticsStore::default();
        let w = window(3);

        store.record_window_open(
            w,
            WindowStyleRequest {
                background_material: Some(WindowBackgroundMaterialRequest::Mica),
                ..Default::default()
            },
            &caps,
        );
        let before = store.effective_snapshot(w).unwrap();
        assert!(before.surface_composited_alpha);
        assert!(before.visual_transparent);
        assert_eq!(
            before.appearance,
            RunnerWindowAppearanceV1::CompositedBackdrop
        );

        store.apply_style_patch(
            w,
            WindowStyleRequest {
                background_material: Some(WindowBackgroundMaterialRequest::None),
                ..Default::default()
            },
            &caps,
        );
        let after = store.effective_snapshot(w).unwrap();
        assert!(after.surface_composited_alpha);
        assert!(!after.visual_transparent);
        assert_eq!(
            after.background_material,
            WindowBackgroundMaterialRequest::None
        );
        assert_eq!(
            after.appearance,
            RunnerWindowAppearanceV1::CompositedNoBackdrop
        );
    }

    #[test]
    fn explicit_transparent_window_defaults_to_visual_transparency_without_material() {
        let caps = PlatformCapabilities::default();
        let mut store = RunnerWindowStyleDiagnosticsStore::default();
        let w = window(4);

        store.record_window_open(
            w,
            WindowStyleRequest {
                transparent: Some(true),
                ..Default::default()
            },
            &caps,
        );
        let have = store.effective_snapshot(w).unwrap();
        assert!(have.surface_composited_alpha);
        assert!(have.visual_transparent);
        assert_eq!(
            have.background_material,
            WindowBackgroundMaterialRequest::None
        );
        assert_eq!(
            have.appearance,
            RunnerWindowAppearanceV1::CompositedNoBackdrop
        );
    }

    #[test]
    fn legacy_mouse_passthrough_maps_to_hit_test_passthrough_all() {
        let caps = PlatformCapabilities::default();
        let mut store = RunnerWindowStyleDiagnosticsStore::default();
        let w = window(5);

        store.record_window_open(
            w,
            WindowStyleRequest {
                mouse: Some(MousePolicy::Passthrough),
                ..Default::default()
            },
            &caps,
        );
        let have = store.effective_snapshot(w).unwrap();
        assert_eq!(have.hit_test, WindowHitTestRequestV1::PassthroughAll);
        assert_eq!(
            have.hit_test_requested,
            Some(WindowHitTestRequestV1::PassthroughAll)
        );
        assert_eq!(
            have.hit_test_source,
            RunnerWindowHitTestSourceV1::LegacyMousePolicy
        );
        assert_eq!(
            have.hit_test_clamp_reason,
            RunnerWindowHitTestClampReasonV1::None
        );
    }

    #[test]
    fn hit_test_passthrough_all_degrades_when_unsupported() {
        let mut caps = PlatformCapabilities::default();
        caps.ui.window_hit_test_passthrough_all = false;

        let mut store = RunnerWindowStyleDiagnosticsStore::default();
        let w = window(6);
        store.record_window_open(
            w,
            WindowStyleRequest {
                hit_test: Some(WindowHitTestRequestV1::PassthroughAll),
                ..Default::default()
            },
            &caps,
        );
        let have = store.effective_snapshot(w).unwrap();
        assert_eq!(have.hit_test, WindowHitTestRequestV1::Normal);
        assert_eq!(
            have.hit_test_requested,
            Some(WindowHitTestRequestV1::PassthroughAll)
        );
        assert_eq!(
            have.hit_test_source,
            RunnerWindowHitTestSourceV1::HitTestFacet
        );
        assert_eq!(
            have.hit_test_clamp_reason,
            RunnerWindowHitTestClampReasonV1::MissingPassthroughAllCapability
        );
    }
}
