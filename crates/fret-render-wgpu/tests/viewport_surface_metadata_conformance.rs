use fret_core::geometry::{Point, Px, Rect, Size};
use fret_core::scene::{DrawOrder, Scene, SceneOp};
use fret_render_wgpu::{
    RenderTargetAlphaMode, RenderTargetColorSpace, RenderTargetDescriptor, RenderTargetMetadata,
    RenderTargetRotation, Renderer, WgpuContext,
};

mod support;

use support::{pixel_rgba, render_scene_rgba8};

fn write_rgba8_texture_solid(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: (u32, u32),
    c: [u8; 4],
) {
    let (w, h) = size;
    let mut data = vec![0u8; (w * h * 4) as usize];
    for px in data.chunks_exact_mut(4) {
        px.copy_from_slice(&c);
    }
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}

fn write_rgba8_texture_quadrants(queue: &wgpu::Queue, texture: &wgpu::Texture, size: (u32, u32)) {
    let (w, h) = size;
    let mut data = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let top = y < h / 2;
            let left = x < w / 2;
            let c = match (top, left) {
                (true, true) => [255, 0, 0, 255],       // TL: red
                (true, false) => [0, 255, 0, 255],      // TR: green
                (false, true) => [0, 0, 255, 255],      // BL: blue
                (false, false) => [255, 255, 255, 255], // BR: white
            };
            let idx = ((y * w + x) * 4) as usize;
            data[idx..idx + 4].copy_from_slice(&c);
        }
    }
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}

#[test]
fn gpu_viewport_surface_respects_alpha_mode_metadata() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let src = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("viewport_surface_metadata_conformance src solid"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    write_rgba8_texture_solid(&ctx.queue, &src, size, [255, 0, 0, 128]); // straight alpha (not premul)
    let src_view = src.create_view(&wgpu::TextureViewDescriptor::default());

    let metadata = RenderTargetMetadata {
        alpha_mode: RenderTargetAlphaMode::Straight,
        ..Default::default()
    };

    let target = renderer.register_render_target(RenderTargetDescriptor {
        view: src_view,
        size,
        format: wgpu::TextureFormat::Rgba8Unorm,
        color_space: RenderTargetColorSpace::Linear,
        metadata,
    });

    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let mut scene = Scene::default();
    scene.push(SceneOp::ViewportSurface {
        order: DrawOrder(0),
        rect,
        target,
        opacity: 1.0,
    });

    let straight = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let s = pixel_rgba(&straight, size.0, 32, 32);

    // With straight alpha metadata, the viewport shader premultiplies the source.
    assert!(s[3] >= 96 && s[3] <= 160, "expected ~0.5 alpha, got {s:?}");
    assert!(
        s[0] >= 96 && s[0] <= 160,
        "expected ~0.5 premul red for straight source, got {s:?}"
    );

    // Now treat the same straight source as premultiplied and verify it becomes visibly brighter.
    let mut premul_meta = metadata;
    premul_meta.alpha_mode = RenderTargetAlphaMode::Premultiplied;
    let _ = renderer.update_render_target(
        target,
        RenderTargetDescriptor {
            view: src.create_view(&wgpu::TextureViewDescriptor::default()),
            size,
            format: wgpu::TextureFormat::Rgba8Unorm,
            color_space: RenderTargetColorSpace::Linear,
            metadata: premul_meta,
        },
    );

    let premul = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let p = pixel_rgba(&premul, size.0, 32, 32);
    assert!(
        p[0] >= 160 && p[0] >= s[0].saturating_add(32),
        "expected noticeably brighter red when treating straight as premul, got {p:?} vs straight {s:?}"
    );
}

#[test]
fn gpu_viewport_surface_respects_orientation_metadata() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let src = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("viewport_surface_metadata_conformance src quadrants"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    write_rgba8_texture_quadrants(&ctx.queue, &src, size);

    let metadata = RenderTargetMetadata {
        alpha_mode: RenderTargetAlphaMode::Premultiplied,
        ..Default::default()
    };

    let target = renderer.register_render_target(RenderTargetDescriptor {
        view: src.create_view(&wgpu::TextureViewDescriptor::default()),
        size,
        format: wgpu::TextureFormat::Rgba8Unorm,
        color_space: RenderTargetColorSpace::Linear,
        metadata,
    });

    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));
    let mut scene = Scene::default();
    scene.push(SceneOp::ViewportSurface {
        order: DrawOrder(0),
        rect,
        target,
        opacity: 1.0,
    });

    let r0 = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let tl = pixel_rgba(&r0, size.0, 8, 8);
    assert_eq!(tl, [255, 0, 0, 255], "expected TL red at R0, got {tl:?}");

    // R180: TL should sample BR (white).
    let mut rot = metadata;
    rot.orientation.rotation = RenderTargetRotation::R180;
    let _ = renderer.update_render_target(
        target,
        RenderTargetDescriptor {
            view: src.create_view(&wgpu::TextureViewDescriptor::default()),
            size,
            format: wgpu::TextureFormat::Rgba8Unorm,
            color_space: RenderTargetColorSpace::Linear,
            metadata: rot,
        },
    );
    let r180 = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let tl2 = pixel_rgba(&r180, size.0, 8, 8);
    assert_eq!(
        tl2,
        [255, 255, 255, 255],
        "expected TL white at R180, got {tl2:?}"
    );

    // Mirror X at R0: TL should sample TR (green).
    let mut mir = metadata;
    mir.orientation.mirror_x = true;
    let _ = renderer.update_render_target(
        target,
        RenderTargetDescriptor {
            view: src.create_view(&wgpu::TextureViewDescriptor::default()),
            size,
            format: wgpu::TextureFormat::Rgba8Unorm,
            color_space: RenderTargetColorSpace::Linear,
            metadata: mir,
        },
    );
    let mx = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let tl3 = pixel_rgba(&mx, size.0, 8, 8);
    assert_eq!(
        tl3,
        [0, 255, 0, 255],
        "expected TL green with mirror_x, got {tl3:?}"
    );
}
