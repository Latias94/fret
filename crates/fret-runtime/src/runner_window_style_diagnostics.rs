use std::collections::HashMap;

use fret_core::AppWindowId;

use crate::PlatformCapabilities;
use crate::window_style::{
    ActivationPolicy, MousePolicy, TaskbarVisibility, WindowBackgroundMaterialRequest,
    WindowDecorationsRequest, WindowStyleRequest, WindowZLevel,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunnerWindowStyleEffectiveSnapshotV1 {
    pub decorations: WindowDecorationsRequest,
    pub resizable: bool,
    pub transparent: bool,
    pub background_material: WindowBackgroundMaterialRequest,
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
            transparent: false,
            background_material: WindowBackgroundMaterialRequest::None,
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
}

impl RunnerWindowStyleDiagnosticsStore {
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

        if caps.ui.window_transparent {
            if let Some(transparent) = requested.transparent {
                next.transparent = transparent;
            } else if next.background_material != WindowBackgroundMaterialRequest::None {
                // Background materials may require a composited alpha surface. If the caller did
                // not explicitly request `transparent`, runners may implicitly treat it as true
                // once a non-None material is effectively applied. See ADR 0310.
                next.transparent = true;
            }
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
            current.background_material = clamp_background_material_request(material, caps);
            if caps.ui.window_transparent {
                let explicit = self.transparent_explicit.get(&window).copied().flatten();
                current.transparent = match explicit {
                    Some(v) => v,
                    None => current.background_material != WindowBackgroundMaterialRequest::None,
                };
            }
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
