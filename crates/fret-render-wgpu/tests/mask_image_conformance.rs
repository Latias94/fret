use fret_core::AlphaMode;
use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, Mask, Paint, Scene, SceneOp, UvRect};
use fret_render_wgpu::{
    ClearColor, ImageColorSpace, ImageDescriptor, RenderSceneParams, Renderer, SvgAlphaMask,
    UploadedAlphaMask, WgpuContext, upload_alpha_mask,
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
        label: Some("mask_image_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("mask_image_conformance readback encoder"),
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

fn render_and_readback(
    ctx: &WgpuContext,
    renderer: &mut Renderer,
    scene: &Scene,
    size: (u32, u32),
) -> Vec<u8> {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("mask_image_conformance output"),
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

fn half_plane_mask(size_px: (u32, u32), left_opaque: bool) -> SvgAlphaMask {
    let (w, h) = size_px;
    let mut alpha = vec![0u8; (w as usize) * (h as usize)];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let is_left = (x as u32) < (w / 2);
            let cov = if is_left == left_opaque { 255 } else { 0 };
            alpha[y * w as usize + x] = cov;
        }
    }
    SvgAlphaMask { size_px, alpha }
}

fn register_mask_image(
    ctx: &WgpuContext,
    renderer: &mut Renderer,
    mask: &SvgAlphaMask,
) -> (fret_core::ImageId, UploadedAlphaMask) {
    let uploaded = upload_alpha_mask(&ctx.device, &ctx.queue, mask);
    let id = renderer.register_image(ImageDescriptor {
        view: uploaded.view.clone(),
        size: uploaded.size_px,
        format: wgpu::TextureFormat::R8Unorm,
        color_space: ImageColorSpace::Linear,
        alpha_mode: AlphaMode::Premultiplied,
    });
    (id, uploaded)
}

#[test]
fn gpu_image_mask_basic_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));

    let mask = half_plane_mask((8, 8), false);
    let (image, _uploaded) = register_mask_image(&ctx, &mut renderer, &mask);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushMask {
        bounds: rect,
        mask: Mask::image(image, UvRect::FULL),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopMask);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
    let left = pixel_rgba(&pixels, size.0, 8, 32);
    let right = pixel_rgba(&pixels, size.0, 56, 32);

    assert!(
        left[3] <= 8,
        "expected near-transparent alpha at left: left={left:?} right={right:?}"
    );
    assert!(
        right[3] >= 247,
        "expected near-opaque alpha at right: left={left:?} right={right:?}"
    );
}

#[test]
fn gpu_image_mask_switches_sources_between_scopes() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect_top = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(32.0)));
    let rect_bot = Rect::new(Point::new(Px(0.0), Px(32.0)), Size::new(Px(64.0), Px(32.0)));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));

    let mask_a = half_plane_mask((8, 8), false);
    let mask_b = half_plane_mask((8, 8), true);
    let (image_a, _uploaded_a) = register_mask_image(&ctx, &mut renderer, &mask_a);
    let (image_b, _uploaded_b) = register_mask_image(&ctx, &mut renderer, &mask_b);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushMask {
        bounds,
        mask: Mask::image(image_a, UvRect::FULL),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: rect_top,
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopMask);

    scene.push(SceneOp::PushMask {
        bounds,
        mask: Mask::image(image_b, UvRect::FULL),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(1),
        rect: rect_bot,
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopMask);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
    let top_left = pixel_rgba(&pixels, size.0, 8, 16);
    let top_right = pixel_rgba(&pixels, size.0, 56, 16);
    let bot_left = pixel_rgba(&pixels, size.0, 8, 48);
    let bot_right = pixel_rgba(&pixels, size.0, 56, 48);

    assert!(
        top_left[3] <= 8 && top_right[3] >= 247,
        "expected mask A on top half: top_left={top_left:?} top_right={top_right:?}"
    );
    assert!(
        bot_left[3] >= 247 && bot_right[3] <= 8,
        "expected mask B on bottom half: bot_left={bot_left:?} bot_right={bot_right:?}"
    );
}

#[test]
fn gpu_nested_image_masks_degrade_deterministically() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));

    let mask_a = half_plane_mask((8, 8), false);
    let mask_b = half_plane_mask((8, 8), true);
    let (image_a, _uploaded_a) = register_mask_image(&ctx, &mut renderer, &mask_a);
    let (image_b, _uploaded_b) = register_mask_image(&ctx, &mut renderer, &mask_b);

    let mut scene = Scene::default();
    scene.push(SceneOp::PushMask {
        bounds: rect,
        mask: Mask::image(image_a, UvRect::FULL),
    });
    // Nested image mask: current wgpu implementation degrades by ignoring the inner image mask.
    scene.push(SceneOp::PushMask {
        bounds: rect,
        mask: Mask::image(image_b, UvRect::FULL),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }))
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::PopMask);
    scene.push(SceneOp::PopMask);

    let pixels = render_and_readback(&ctx, &mut renderer, &scene, size);
    let left = pixel_rgba(&pixels, size.0, 8, 32);
    let right = pixel_rgba(&pixels, size.0, 56, 32);

    assert!(
        left[3] <= 8 && right[3] >= 247,
        "expected inner image mask to be ignored (outer mask A wins): left={left:?} right={right:?}"
    );
}
