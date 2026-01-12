use fret_core::RenderTargetId;
use fret_render::{RenderTargetColorSpace, RenderTargetDescriptor, Renderer, WgpuContext};

/// App-owned offscreen render target intended to be embedded into the UI via `SceneOp::ViewportSurface`.
///
/// This helper is meant to reduce boilerplate in Tier A integrations (engine viewports, video panels,
/// GPU-heavy widgets):
/// - allocate/resize an offscreen `wgpu::Texture`,
/// - register/update the view in `fret-render` to obtain/refresh a `RenderTargetId`,
/// - return a stable `TextureView` reference for recording engine render passes.
///
/// Notes:
/// - The first registration requires `&mut Renderer` because `RenderTargetId` allocation is renderer-owned.
/// - Subsequent resizes call `renderer.update_render_target(...)` (id remains stable).
#[derive(Debug)]
pub struct ViewportRenderTarget {
    id: RenderTargetId,
    size: (u32, u32),
    format: wgpu::TextureFormat,
    color_space: RenderTargetColorSpace,
    texture: Option<wgpu::Texture>,
    view: Option<wgpu::TextureView>,
}

impl ViewportRenderTarget {
    pub fn new(format: wgpu::TextureFormat, color_space: RenderTargetColorSpace) -> Self {
        Self {
            id: RenderTargetId::default(),
            size: (0, 0),
            format,
            color_space,
            texture: None,
            view: None,
        }
    }

    pub fn id(&self) -> RenderTargetId {
        self.id
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn texture(&self) -> Option<&wgpu::Texture> {
        self.texture.as_ref()
    }

    pub fn view(&self) -> Option<&wgpu::TextureView> {
        self.view.as_ref()
    }

    pub fn ensure_size(
        &mut self,
        context: &WgpuContext,
        renderer: &mut Renderer,
        desired_size: (u32, u32),
        label: Option<&str>,
    ) -> (RenderTargetId, &wgpu::TextureView) {
        let w = desired_size.0.max(1);
        let h = desired_size.1.max(1);
        let desired_size = (w, h);

        let needs_new = self.view.is_none() || self.texture.is_none() || self.size != desired_size;

        if needs_new {
            let texture = context.device.create_texture(&wgpu::TextureDescriptor {
                label,
                size: wgpu::Extent3d {
                    width: desired_size.0,
                    height: desired_size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let desc = RenderTargetDescriptor {
                view: view.clone(),
                size: desired_size,
                format: self.format,
                color_space: self.color_space,
            };

            if self.id == RenderTargetId::default() {
                self.id = renderer.register_render_target(desc);
            } else {
                let _ = renderer.update_render_target(self.id, desc);
            }

            self.size = desired_size;
            self.texture = Some(texture);
            self.view = Some(view);
        }

        let view_ref = self
            .view
            .as_ref()
            .expect("ViewportRenderTarget::ensure_size should always set view");
        (self.id, view_ref)
    }

    pub fn unregister(&mut self, renderer: &mut Renderer) {
        if self.id != RenderTargetId::default() {
            let _ = renderer.unregister_render_target(self.id);
            self.id = RenderTargetId::default();
        }
        self.size = (0, 0);
        self.texture = None;
        self.view = None;
    }
}
