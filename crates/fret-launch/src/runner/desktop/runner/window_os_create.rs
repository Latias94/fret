use std::sync::Arc;

use fret_runner_winit::accessibility;
#[cfg(windows)]
use fret_runtime::TaskbarVisibility;
use fret_runtime::{ActivationPolicy, PlatformCapabilities, WindowStyleRequest, WindowZLevel};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowLevel};

use super::{WindowCreateSpec, WindowPosition, WinitAppDriver, WinitRunner, macos_window_log};
use crate::RunnerError;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn create_os_window(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        mut spec: WindowCreateSpec,
        style: WindowStyleRequest,
        _parent_window: Option<winit::raw_window_handle::RawWindowHandle>,
        caps: &PlatformCapabilities,
    ) -> Result<(Arc<dyn Window>, Option<accessibility::WinitAccessibility>), RunnerError> {
        spec.normalize_size_constraints();

        let accessibility_enabled = self.config.accessibility_enabled
            && std::env::var_os("FRET_A11Y_DISABLE").is_none_or(|v| v.is_empty());

        let mut attrs = winit::window::WindowAttributes::default()
            .with_title(spec.title)
            .with_surface_size(winit::dpi::LogicalSize::new(
                spec.size.width,
                spec.size.height,
            ))
            .with_visible(if accessibility_enabled {
                false
            } else {
                spec.visible
            });
        if let Some(min_size) = spec.min_size {
            attrs = attrs.with_min_surface_size(winit::dpi::LogicalSize::new(
                min_size.width,
                min_size.height,
            ));
        }
        if let Some(max_size) = spec.max_size {
            attrs = attrs.with_max_surface_size(winit::dpi::LogicalSize::new(
                max_size.width,
                max_size.height,
            ));
        }
        if let Some(resize_increments) = spec.resize_increments {
            attrs = attrs.with_surface_resize_increments(winit::dpi::LogicalSize::new(
                resize_increments.width,
                resize_increments.height,
            ));
        }
        if let Some(resizable) = style.resizable
            && caps.ui.window_resizable
        {
            attrs = attrs.with_resizable(resizable);
        }
        if let Some(decorations) = style.decorations
            && caps.ui.window_decorations
            && matches!(decorations, fret_runtime::WindowDecorationsRequest::None)
        {
            attrs = attrs.with_decorations(false);
        }
        let effective_background_material = style.background_material.map(|m| {
            fret_runtime::runner_window_style_diagnostics::clamp_background_material_request(
                m, caps,
            )
        });

        let effective_surface_composited_alpha = if caps.ui.window_transparent {
            if let Some(transparent) = style.transparent {
                transparent
            } else {
                effective_background_material
                    .is_some_and(|m| m != fret_runtime::WindowBackgroundMaterialRequest::None)
            }
        } else {
            false
        };

        if caps.ui.window_transparent {
            // NOTE: `transparent` is a create-time property in winit; we may keep the window
            // composited for its lifetime even if the material is later set to None at runtime.
            attrs = attrs.with_transparent(effective_surface_composited_alpha);
        }
        if let Some(policy) = style.activation
            && (policy == ActivationPolicy::Activates || caps.ui.window_non_activating)
        {
            let active = matches!(policy, ActivationPolicy::Activates);
            attrs = attrs.with_active(active);
        }
        if let Some(position) = spec.position {
            let position = match position {
                WindowPosition::Logical(pos) => winit::dpi::Position::Logical(
                    winit::dpi::LogicalPosition::new(pos.x as f64, pos.y as f64),
                ),
                WindowPosition::Physical(pos) => {
                    winit::dpi::Position::Physical(winit::dpi::PhysicalPosition::new(pos.x, pos.y))
                }
            };
            attrs = attrs.with_position(position);
        }
        #[cfg(windows)]
        {
            if let Some(taskbar) = style.taskbar
                && (taskbar == TaskbarVisibility::Show || caps.ui.window_skip_taskbar)
            {
                use winit::platform::windows::WindowAttributesWindows;

                let win = WindowAttributesWindows::default()
                    .with_skip_taskbar(matches!(taskbar, TaskbarVisibility::Hide));
                attrs = attrs.with_platform_attributes(Box::new(win));
            }
        }
        #[cfg(target_os = "macos")]
        if _parent_window.is_some() {
            // macOS tool/aux windows: best-effort parent/child relationship so DockFloating windows
            // follow the parent window's Space/fullscreen lifecycle.
            //
            // winit maps this to `NSWindow.addChildWindow_ordered(...)`.
            attrs = unsafe { attrs.with_parent_window(_parent_window) };
        }
        let window = Arc::<dyn Window>::from(
            event_loop
                .create_window(attrs)
                .map_err(|source| RunnerError::CreateWindowFailed { source })?,
        );

        macos_window_log(format_args!("[create] winit={:?}", window.id()));

        let accessibility = accessibility_enabled
            .then(|| accessibility::WinitAccessibility::new(event_loop, window.as_ref()));

        if accessibility_enabled && spec.visible {
            window.set_visible(true);
        }

        if let Some(level) = style.z_level
            && (level == WindowZLevel::Normal
                || caps.ui.window_z_level != fret_runtime::WindowZLevelQuality::None)
        {
            window.set_window_level(match level {
                WindowZLevel::Normal => WindowLevel::Normal,
                WindowZLevel::AlwaysOnTop => WindowLevel::AlwaysOnTop,
            });
        }

        if effective_surface_composited_alpha
            && let Some(material) = effective_background_material
            && material != fret_runtime::WindowBackgroundMaterialRequest::None
        {
            let _ =
                super::window_platform::set_window_background_material(window.as_ref(), material);
        }

        if let Some(hit_test) = style.hit_test.clone() {
            let (effective, _reason) =
                fret_runtime::RunnerWindowStyleDiagnosticsStore::clamp_hit_test_request(
                    hit_test, caps,
                );
            let _ = super::window_platform::set_window_hit_test(window.as_ref(), &effective);
        }
        if let Some(opacity) = style.opacity
            && caps.ui.window_opacity
        {
            let _ = super::window_platform::set_window_opacity(window.as_ref(), opacity.as_f32());
        }

        Ok((window, accessibility))
    }
}
