use std::collections::HashMap;
use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, RenderTargetId};
use fret_render::viewport_overlay::{
    Overlay3dBatch, Overlay3dCpuBuilder, Overlay3dPipelineConfig, Overlay3dPipelines,
    Overlay3dUniforms, ViewportOverlay3dContext,
};

use super::{ViewportOverlay3dHooks, ViewportOverlay3dHooksService};

#[derive(Debug, Clone)]
struct ViewportOverlay3dImmediateEntry {
    color_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    overlay: Overlay3dPipelines,
    batch: Overlay3dBatch,
}

/// A small helper cache for recording immediate-mode 3D overlays (gizmos, debug draw) in the engine pass.
///
/// This is intentionally runner-facing (wgpu-facing) and keeps the "pass topology" boundary explicit:
/// the host still owns command encoder + render pass creation, and calls `record_viewport_overlay_3d(...)`
/// to execute overlay hooks.
#[derive(Debug, Default)]
pub struct ViewportOverlay3dImmediateService {
    installed: bool,
    config: Overlay3dPipelineConfig,
    entries: HashMap<(AppWindowId, RenderTargetId), ViewportOverlay3dImmediateEntry>,
}

impl ViewportOverlay3dImmediateService {
    pub fn with_pipeline_config(mut self, config: Overlay3dPipelineConfig) -> Self {
        self.config = config;
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: AppWindowId,
        target: RenderTargetId,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        uniforms: Overlay3dUniforms,
        cpu: &Overlay3dCpuBuilder,
    ) -> Overlay3dPipelines {
        let entry = self.entries.entry((window, target)).or_insert_with(|| {
            ViewportOverlay3dImmediateEntry {
                color_format,
                depth_format,
                overlay: Overlay3dPipelines::new_with_config(
                    device,
                    color_format,
                    depth_format,
                    self.config,
                ),
                batch: Overlay3dBatch::default(),
            }
        });

        if entry.color_format != color_format || entry.depth_format != depth_format {
            *entry = ViewportOverlay3dImmediateEntry {
                color_format,
                depth_format,
                overlay: Overlay3dPipelines::new_with_config(
                    device,
                    color_format,
                    depth_format,
                    self.config,
                ),
                batch: Overlay3dBatch::default(),
            };
        }

        queue.write_buffer(&entry.overlay.uniform, 0, bytemuck::bytes_of(&uniforms));
        entry.batch.upload(
            device,
            queue,
            cpu.solid_test(),
            cpu.solid_ghost(),
            cpu.solid_always(),
            cpu.line_test(),
            cpu.line_ghost(),
            cpu.line_always(),
        );
        entry.overlay.clone()
    }

    pub fn record(
        &self,
        window: AppWindowId,
        target: RenderTargetId,
        pass: &mut wgpu::RenderPass<'_>,
    ) {
        let Some(entry) = self.entries.get(&(window, target)) else {
            return;
        };
        entry.batch.record(&entry.overlay, pass);
    }

    pub fn clear(&mut self, window: AppWindowId, target: RenderTargetId) {
        let _ = self.entries.remove(&(window, target));
    }

    pub fn clear_window(&mut self, window: AppWindowId) {
        self.entries.retain(|(w, _), _| *w != window);
    }
}

struct ViewportOverlay3dImmediateHooks;

impl ViewportOverlay3dHooks for ViewportOverlay3dImmediateHooks {
    fn record(
        &self,
        app: &mut App,
        window: AppWindowId,
        target: RenderTargetId,
        pass: &mut wgpu::RenderPass<'_>,
        _ctx: &ViewportOverlay3dContext,
    ) {
        let Some(svc) = app.global::<ViewportOverlay3dImmediateService>() else {
            return;
        };
        svc.record(window, target, pass);
    }
}

/// Installs the immediate 3D overlay helper into the app's `ViewportOverlay3dHooksService`.
///
/// This is idempotent: calling it multiple times does not install duplicate hooks.
pub fn install_viewport_overlay_3d_immediate(app: &mut App) {
    let needs_install =
        app.with_global_mut(ViewportOverlay3dImmediateService::default, |svc, _app| {
            if svc.installed {
                false
            } else {
                svc.installed = true;
                true
            }
        });

    if needs_install {
        app.with_global_mut(ViewportOverlay3dHooksService::default, |svc, _app| {
            svc.push(Arc::new(ViewportOverlay3dImmediateHooks));
        });
    }
}

/// Uploads the immediate overlay batch for the given window/target and returns the overlay pipelines.
///
/// This also ensures the immediate overlay hook is installed, so `record_viewport_overlay_3d(...)`
/// will replay the uploaded batches inside the engine pass.
#[allow(clippy::too_many_arguments)]
pub fn upload_viewport_overlay_3d_immediate(
    app: &mut App,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    window: AppWindowId,
    target: RenderTargetId,
    color_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    uniforms: Overlay3dUniforms,
    cpu: &Overlay3dCpuBuilder,
) -> Overlay3dPipelines {
    install_viewport_overlay_3d_immediate(app);
    app.with_global_mut(ViewportOverlay3dImmediateService::default, |svc, _app| {
        svc.upload(
            device,
            queue,
            window,
            target,
            color_format,
            depth_format,
            uniforms,
            cpu,
        )
    })
}
