use fret_core::RenderTargetId;
use fret_render::{
    RenderTargetColorSpace, RenderTargetDescriptor, RenderTargetMetadata, Renderer, WgpuContext,
};

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
    usage: wgpu::TextureUsages,
    view_formats: Vec<wgpu::TextureFormat>,
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: Vec::new(),
            texture: None,
            view: None,
        }
    }

    pub fn with_usage(mut self, usage: wgpu::TextureUsages) -> Self {
        self.usage = usage;
        self
    }

    pub fn with_view_formats(mut self, view_formats: &[wgpu::TextureFormat]) -> Self {
        self.view_formats = view_formats.to_vec();
        self
    }

    pub fn id(&self) -> RenderTargetId {
        self.id
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn color_space(&self) -> RenderTargetColorSpace {
        self.color_space
    }

    pub fn usage(&self) -> wgpu::TextureUsages {
        self.usage
    }

    pub fn view_formats(&self) -> &[wgpu::TextureFormat] {
        &self.view_formats
    }

    pub fn texture(&self) -> Option<&wgpu::Texture> {
        self.texture.as_ref()
    }

    pub fn view(&self) -> Option<&wgpu::TextureView> {
        self.view.as_ref()
    }

    pub fn ensure_size_owned_view(
        &mut self,
        context: &WgpuContext,
        renderer: &mut Renderer,
        desired_size: (u32, u32),
        label: Option<&str>,
    ) -> (RenderTargetId, wgpu::TextureView) {
        let (id, view) = {
            let (id, view_ref) = self.ensure_size(context, renderer, desired_size, label);
            (id, view_ref.clone())
        };
        (id, view)
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
                usage: self.usage,
                view_formats: &self.view_formats,
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let desc = RenderTargetDescriptor {
                view: view.clone(),
                size: desired_size,
                format: self.format,
                color_space: self.color_space,
                metadata: RenderTargetMetadata::default(),
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

/// Like [`ViewportRenderTarget`], but also manages a depth attachment texture for 3D/engine-style viewports.
#[derive(Debug)]
pub struct ViewportRenderTargetWithDepth {
    color: ViewportRenderTarget,
    depth_format: wgpu::TextureFormat,
    depth_usage: wgpu::TextureUsages,
    depth_view_formats: Vec<wgpu::TextureFormat>,
    depth_texture: Option<wgpu::Texture>,
    depth_view: Option<wgpu::TextureView>,
}

impl ViewportRenderTargetWithDepth {
    pub fn new(
        color_format: wgpu::TextureFormat,
        color_space: RenderTargetColorSpace,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            color: ViewportRenderTarget::new(color_format, color_space),
            depth_format,
            depth_usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            depth_view_formats: Vec::new(),
            depth_texture: None,
            depth_view: None,
        }
    }

    pub fn with_color_usage(mut self, usage: wgpu::TextureUsages) -> Self {
        self.color = self.color.with_usage(usage);
        self
    }

    pub fn with_color_view_formats(mut self, view_formats: &[wgpu::TextureFormat]) -> Self {
        self.color = self.color.with_view_formats(view_formats);
        self
    }

    pub fn with_depth_usage(mut self, usage: wgpu::TextureUsages) -> Self {
        self.depth_usage = usage;
        self
    }

    pub fn with_depth_view_formats(mut self, view_formats: &[wgpu::TextureFormat]) -> Self {
        self.depth_view_formats = view_formats.to_vec();
        self
    }

    pub fn id(&self) -> RenderTargetId {
        self.color.id()
    }

    pub fn color_format(&self) -> wgpu::TextureFormat {
        self.color.format()
    }

    pub fn depth_format(&self) -> wgpu::TextureFormat {
        self.depth_format
    }

    pub fn size(&self) -> (u32, u32) {
        self.color.size()
    }

    pub fn ensure_size_owned_views(
        &mut self,
        context: &WgpuContext,
        renderer: &mut Renderer,
        desired_size: (u32, u32),
        color_label: Option<&str>,
        depth_label: Option<&str>,
    ) -> (RenderTargetId, wgpu::TextureView, wgpu::TextureView) {
        let (id, color, depth) = {
            let (id, color_ref, depth_ref) =
                self.ensure_size(context, renderer, desired_size, color_label, depth_label);
            (id, color_ref.clone(), depth_ref.clone())
        };
        (id, color, depth)
    }

    pub fn ensure_size(
        &mut self,
        context: &WgpuContext,
        renderer: &mut Renderer,
        desired_size: (u32, u32),
        color_label: Option<&str>,
        depth_label: Option<&str>,
    ) -> (RenderTargetId, &wgpu::TextureView, &wgpu::TextureView) {
        let desired_size = (desired_size.0.max(1), desired_size.1.max(1));
        let prev_size = self.color.size();
        let (id, color_view) = self
            .color
            .ensure_size(context, renderer, desired_size, color_label);
        let size = desired_size;

        if self.depth_view.is_none() || self.depth_texture.is_none() || prev_size != size {
            let depth = context.device.create_texture(&wgpu::TextureDescriptor {
                label: depth_label,
                size: wgpu::Extent3d {
                    width: size.0.max(1),
                    height: size.1.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.depth_format,
                usage: self.depth_usage,
                view_formats: &self.depth_view_formats,
            });
            let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
            self.depth_texture = Some(depth);
            self.depth_view = Some(depth_view);
        }

        let depth_view_ref = self
            .depth_view
            .as_ref()
            .expect("ViewportRenderTargetWithDepth::ensure_size should always set depth_view");
        (id, color_view, depth_view_ref)
    }

    pub fn unregister(&mut self, renderer: &mut Renderer) {
        self.color.unregister(renderer);
        self.depth_texture = None;
        self.depth_view = None;
    }
}
