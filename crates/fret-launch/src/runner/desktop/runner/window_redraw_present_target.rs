use fret_render::{RenderError, SurfaceAcquireError, SurfaceState, WgpuContext};

pub(super) struct WindowRedrawPresentTargetInput<'a, 'window> {
    pub(super) context: &'a WgpuContext,
    pub(super) surface: &'a SurfaceState<'window>,
    pub(super) frame_view: Option<(wgpu::SurfaceTexture, wgpu::TextureView)>,
}

pub(super) struct WindowRedrawPresentTarget {
    frame_view: Option<(wgpu::SurfaceTexture, wgpu::TextureView)>,
    fallback_target: Option<(wgpu::Texture, wgpu::TextureView)>,
}

impl WindowRedrawPresentTarget {
    pub(super) fn target_view(&self) -> &wgpu::TextureView {
        self.frame_view
            .as_ref()
            .map(|(_, view)| view)
            .or_else(|| self.fallback_target.as_ref().map(|(_, view)| view))
            .expect("renderer perf fallback should provide a target view")
    }

    pub(super) fn frame_view(&self) -> Option<&(wgpu::SurfaceTexture, wgpu::TextureView)> {
        self.frame_view.as_ref()
    }

    pub(super) fn into_frame_view(self) -> Option<(wgpu::SurfaceTexture, wgpu::TextureView)> {
        self.frame_view
    }
}

pub(super) fn acquire_window_redraw_present_frame(
    surface: &SurfaceState<'_>,
) -> Result<Option<(wgpu::SurfaceTexture, wgpu::TextureView)>, RenderError> {
    match surface.get_current_frame_view() {
        Ok(frame_view) => Ok(Some(frame_view)),
        Err(source) => {
            if !renderer_perf_fallback_enabled() || source != SurfaceAcquireError::Other {
                return Err(RenderError::SurfaceAcquireFailed { source });
            }
            Ok(None)
        }
    }
}

pub(super) fn prepare_window_redraw_present_target(
    input: WindowRedrawPresentTargetInput<'_, '_>,
) -> WindowRedrawPresentTarget {
    let fallback_target = if input.frame_view.is_none() {
        let target = input
            .context
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("fret diag renderer perf fallback target"),
                size: wgpu::Extent3d {
                    width: input.surface.size().0.max(1),
                    height: input.surface.size().1.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: input.surface.format(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        Some((target, view))
    } else {
        None
    };

    WindowRedrawPresentTarget {
        frame_view: input.frame_view,
        fallback_target,
    }
}

fn renderer_perf_fallback_enabled() -> bool {
    std::env::var_os("FRET_DIAG_RENDERER_PERF").is_some_and(|value| !value.is_empty())
}
