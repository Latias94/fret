use std::collections::HashMap;

use fret_core::{PanelKey, RenderTargetId, geometry::Rect};
use fret_render::{RenderTargetColorSpace, RenderTargetDescriptor, Renderer};
use fret_runner_winit_wgpu::RenderTargetUpdate;

pub struct ViewportTarget {
    pub target: RenderTargetId,
    pub target_px_size: (u32, u32),
    pub view: wgpu::TextureView,
    texture: wgpu::Texture,
    label: &'static str,
}

impl ViewportTarget {
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
    pub const MAX_TARGET_PX: u32 = 8192;
    pub const RESIZE_BUCKET_PX: u32 = 64;

    pub fn new(
        device: &wgpu::Device,
        renderer: &mut Renderer,
        label: &'static str,
        target_px_size: (u32, u32),
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: target_px_size.0,
                height: target_px_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let ui_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let target = renderer.register_render_target(RenderTargetDescriptor {
            view: ui_view,
            size: target_px_size,
            format: Self::FORMAT,
            color_space: RenderTargetColorSpace::Srgb,
        });

        Self {
            target,
            target_px_size,
            texture,
            view,
            label,
        }
    }

    fn round_up(v: u32, bucket: u32) -> u32 {
        if bucket <= 1 {
            return v;
        }
        ((v + bucket - 1) / bucket) * bucket
    }

    pub fn desired_px_from_content(content: Rect, scale_factor: f32) -> (u32, u32) {
        let w = (content.size.width.0 * scale_factor).round().max(1.0) as u32;
        let h = (content.size.height.0 * scale_factor).round().max(1.0) as u32;

        let w = Self::round_up(w.max(1).min(Self::MAX_TARGET_PX), Self::RESIZE_BUCKET_PX);
        let h = Self::round_up(h.max(1).min(Self::MAX_TARGET_PX), Self::RESIZE_BUCKET_PX);
        (w.max(1), h.max(1))
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        desired_px: (u32, u32),
    ) -> Option<RenderTargetUpdate> {
        if self.target_px_size == desired_px {
            return None;
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(self.label),
            size: wgpu::Extent3d {
                width: desired_px.0,
                height: desired_px.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let engine_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let ui_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let desc = RenderTargetDescriptor {
            view: ui_view,
            size: desired_px,
            format: Self::FORMAT,
            color_space: RenderTargetColorSpace::Srgb,
        };

        self.texture = texture;
        self.view = engine_view;
        self.target_px_size = desired_px;
        Some(RenderTargetUpdate::Update {
            id: self.target,
            desc,
        })
    }
}

#[derive(Default)]
pub struct ViewportTargets {
    targets: HashMap<PanelKey, ViewportTarget>,
}

impl ViewportTargets {
    pub fn insert(&mut self, panel: PanelKey, target: ViewportTarget) {
        self.targets.insert(panel, target);
    }

    pub fn panel_keys(&self) -> impl Iterator<Item = &PanelKey> {
        self.targets.keys()
    }

    pub fn get(&self, panel: &PanelKey) -> Option<&ViewportTarget> {
        self.targets.get(panel)
    }

    pub fn get_mut(&mut self, panel: &PanelKey) -> Option<&mut ViewportTarget> {
        self.targets.get_mut(panel)
    }
}
