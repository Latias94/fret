use fret_core::geometry::{Point, Px, Rect, Size};
use fret_core::scene::{DrawOrder, Scene, SceneOp};
use fret_render_wgpu::{
    ClearColor, RenderSceneParams, RenderTargetAlphaMode, RenderTargetColorSpace,
    RenderTargetDescriptor, RenderTargetMetadata, RenderTargetRotation, Renderer, WgpuContext,
};
use std::sync::mpsc;

fn read_texture_rgba8(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: (u32, u32),
) -> Vec<u8> {
    let (width, height) = size;
    let bytes_per_pixel: u32 = 4;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(256) * 256;
    let buffer_size = padded_bytes_per_row as u64 * height as u64;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("viewport_surface_metadata_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("viewport_surface_metadata_conformance readback encoder"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    let slice = buffer.slice(..);
    let (tx, rx) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv().expect("map_async channel closed").unwrap();

    let mapped = slice.get_mapped_range();
    let mut pixels = vec![0u8; (unpadded_bytes_per_row * height) as usize];
    for row in 0..height as usize {
        let src = row * padded_bytes_per_row as usize;
        let dst = row * unpadded_bytes_per_row as usize;
        pixels[dst..dst + unpadded_bytes_per_row as usize]
            .copy_from_slice(&mapped[src..src + unpadded_bytes_per_row as usize]);
    }
    drop(mapped);
    buffer.unmap();
    pixels
}

fn pixel_rgba(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let idx = ((y * width + x) * 4) as usize;
    [
        pixels[idx],
        pixels[idx + 1],
        pixels[idx + 2],
        pixels[idx + 3],
    ]
}

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

fn render_and_readback(
    ctx: &WgpuContext,
    renderer: &mut Renderer,
    scene: &Scene,
    size: (u32, u32),
) -> Vec<u8> {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("viewport_surface_metadata_conformance output"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let cb = renderer.render_scene(
        &ctx.device,
        &ctx.queue,
        RenderSceneParams {
            format,
            target_view: &view,
            scene,
            clear: ClearColor(wgpu::Color::TRANSPARENT),
            scale_factor: 1.0,
            viewport_size: size,
        },
    );
    ctx.queue.submit([cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());
    read_texture_rgba8(&ctx.device, &ctx.queue, &texture, size)
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

    let straight = render_and_readback(&ctx, &mut renderer, &scene, size);
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

    let premul = render_and_readback(&ctx, &mut renderer, &scene, size);
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

    let r0 = render_and_readback(&ctx, &mut renderer, &scene, size);
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
    let r180 = render_and_readback(&ctx, &mut renderer, &scene, size);
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
    let mx = render_and_readback(&ctx, &mut renderer, &scene, size);
    let tl3 = pixel_rgba(&mx, size.0, 8, 8);
    assert_eq!(
        tl3,
        [0, 255, 0, 255],
        "expected TL green with mirror_x, got {tl3:?}"
    );
}
