use super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(in crate::runner::desktop::runner) fn apply_window_style_request(
        &mut self,
        window: fret_core::AppWindowId,
        style: WindowStyleRequest,
    ) {
        let Some(state) = self.windows.get(window) else {
            return;
        };

        let window_handle = state.window.clone();
        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let requested_background_material = style.background_material;

        if let Some(level) = style.z_level
            && (level == WindowZLevel::Normal
                || caps.ui.window_z_level != fret_runtime::WindowZLevelQuality::None)
        {
            window_handle.set_window_level(match level {
                WindowZLevel::Normal => WindowLevel::Normal,
                WindowZLevel::AlwaysOnTop => WindowLevel::AlwaysOnTop,
            });
        }

        if let Some(hit_test) = style.hit_test.clone() {
            let dock_drag_pointer_id = self.dock_drag_pointer_id();
            let (effective, _reason) =
                fret_runtime::RunnerWindowStyleDiagnosticsStore::clamp_hit_test_request(
                    hit_test, &caps,
                );
            let applied = window::set_window_hit_test(window_handle.as_ref(), &effective);
            let passthrough_all = matches!(
                effective,
                fret_runtime::WindowHitTestRequestV1::PassthroughAll
            );
            if let Some(follow) = self.dock_tearoff_follow.as_mut()
                && follow.window == window
            {
                follow.hit_test_passthrough_all_applied = passthrough_all && applied;
                if let Some(pointer_id) = dock_drag_pointer_id
                    && let Some(drag) = self.app.drag_mut(pointer_id)
                    && drag.source_window == follow.source_window
                {
                    drag.transparent_payload_hit_test_passthrough_applied =
                        passthrough_all && applied;
                }
            }
        }

        if let Some(opacity) = style.opacity
            && caps.ui.window_opacity
        {
            let _ = window::set_window_opacity(window_handle.as_ref(), opacity.as_f32());
        }

        self.app.with_global_mut(
            fret_runtime::RunnerWindowStyleDiagnosticsStore::default,
            |svc, _app| {
                svc.apply_style_patch(window, style, &caps);
            },
        );

        let effective_style = self
            .app
            .global::<fret_runtime::RunnerWindowStyleDiagnosticsStore>()
            .and_then(|s| s.effective_snapshot(window));

        if requested_background_material.is_some() {
            let material = effective_style
                .as_ref()
                .map(|s| s.background_material)
                .unwrap_or_else(|| {
                    fret_runtime::runner_window_style_diagnostics::clamp_background_material_request(
                        requested_background_material
                            .unwrap_or(fret_runtime::WindowBackgroundMaterialRequest::None),
                        &caps,
                    )
                });
            let _ = window::set_window_background_material(window_handle.as_ref(), material);
        }

        if let Some(context) = self.context.as_ref() {
            let want_surface_composited_alpha = effective_style
                .as_ref()
                .is_some_and(|s| s.surface_composited_alpha);
            if let Some(state) = self.windows.get_mut(window)
                && let Some(surface) = state.surface.as_mut()
            {
                window_lifecycle::configure_surface_alpha_mode_for_composited_window(
                    &context.adapter,
                    &context.device,
                    surface,
                    want_surface_composited_alpha,
                );
                let surface_record =
                    render::capture_surface_config_diagnostics_record(&surface.config);
                let _ = surface;
                let _ = state;
                self.record_surface_config_snapshot(window, surface_record);
            }
        }

        window_handle.request_redraw();
    }
}
