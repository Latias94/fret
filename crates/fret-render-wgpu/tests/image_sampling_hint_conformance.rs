use fret_core::AlphaMode;
use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, ImageSamplingHint, Paint, Scene, SceneOp, UvRect};
use fret_render_wgpu::{
    ClearColor, ImageColorSpace, ImageDescriptor, RenderSceneParams, Renderer, WgpuContext,
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
        label: Some("image_sampling_hint_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("image_sampling_hint_conformance readback encoder"),
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

fn register_rgba_checkerboard(
    ctx: &WgpuContext,
    renderer: &mut Renderer,
) -> (fret_core::ImageId, wgpu::Texture) {
    let size = (2u32, 2u32);
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image_sampling_hint_conformance checkerboard"),
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

    // 2x2 RGBA8 checkerboard:
    // (0,0)=black, (1,0)=white
    // (0,1)=white, (1,1)=black
    let bytes: [u8; 16] = [
        0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
    ];
    ctx.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * size.0),
            rows_per_image: Some(size.1),
        },
        wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let id = renderer.register_image(ImageDescriptor {
        view: view.clone(),
        size,
        format: wgpu::TextureFormat::Rgba8Unorm,
        color_space: ImageColorSpace::Linear,
        alpha_mode: AlphaMode::Opaque,
    });
    (id, texture)
}

#[test]
fn image_sampling_hint_nearest_vs_linear_differs() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let (image, _texture) = register_rgba_checkerboard(&ctx, &mut renderer);

    let tile_px = 6u32;
    let size = (tile_px * 2, tile_px);
    let rect_left = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(tile_px as f32), Px(tile_px as f32)),
    );
    let rect_right = Rect::new(
        Point::new(Px(tile_px as f32), Px(0.0)),
        Size::new(Px(tile_px as f32), Px(tile_px as f32)),
    );

    let mut scene = Scene::default();
    scene.push(SceneOp::ImageRegion {
        order: DrawOrder(0),
        rect: rect_left,
        image,
        uv: UvRect::FULL,
        sampling: ImageSamplingHint::Nearest,
        opacity: 1.0,
    });
    scene.push(SceneOp::ImageRegion {
        order: DrawOrder(0),
        rect: rect_right,
        image,
        uv: UvRect::FULL,
        sampling: ImageSamplingHint::Linear,
        opacity: 1.0,
    });

    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image_sampling_hint_conformance output"),
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
            scene: &scene,
            clear: ClearColor(wgpu::Color::TRANSPARENT),
            scale_factor: 1.0,
            viewport_size: size,
        },
    );
    ctx.queue.submit([cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());

    let pixels = read_texture_rgba8(&ctx.device, &ctx.queue, &texture, size);
    let mut nearest_has_mid = false;
    let mut linear_has_mid = false;
    for y in 1..tile_px.saturating_sub(1) {
        for x in 1..tile_px.saturating_sub(1) {
            let nearest = pixel_rgba(&pixels, size.0, x, y);
            let linear = pixel_rgba(&pixels, size.0, tile_px + x, y);
            assert_eq!(nearest[3], 255, "nearest alpha must be opaque");
            assert_eq!(linear[3], 255, "linear alpha must be opaque");

            let nearest_r = nearest[0];
            let linear_r = linear[0];
            let nearest_is_mid = (16..=239).contains(&nearest_r);
            let linear_is_mid = (16..=239).contains(&linear_r);
            nearest_has_mid |= nearest_is_mid;
            linear_has_mid |= linear_is_mid;
        }
    }

    assert!(
        linear_has_mid,
        "expected at least one blended texel for linear sampling"
    );
    assert!(
        !nearest_has_mid,
        "expected no blended texels for nearest sampling"
    );
}

#[test]
fn image_sampling_hint_preserves_order_across_mixed_primitives() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    let (image, _texture) = register_rgba_checkerboard(&ctx, &mut renderer);

    let tile_px = 6u32;
    let size = (tile_px * 2, tile_px);
    let full_rect = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(size.0 as f32), Px(size.1 as f32)),
    );
    let rect_left = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(tile_px as f32), Px(tile_px as f32)),
    );
    let rect_right = Rect::new(
        Point::new(Px(tile_px as f32), Px(0.0)),
        Size::new(Px(tile_px as f32), Px(tile_px as f32)),
    );

    let mut scene = Scene::default();
    scene.push(SceneOp::ImageRegion {
        order: DrawOrder(0),
        rect: rect_left,
        image,
        uv: UvRect::FULL,
        sampling: ImageSamplingHint::Nearest,
        opacity: 1.0,
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: full_rect,
        background: Paint::Solid(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 0.25,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::ImageRegion {
        order: DrawOrder(0),
        rect: rect_right,
        image,
        uv: UvRect::FULL,
        sampling: ImageSamplingHint::Linear,
        opacity: 1.0,
    });

    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image_sampling_hint_conformance output (order)"),
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
            scene: &scene,
            clear: ClearColor(wgpu::Color::TRANSPARENT),
            scale_factor: 1.0,
            viewport_size: size,
        },
    );
    ctx.queue.submit([cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());

    let pixels = read_texture_rgba8(&ctx.device, &ctx.queue, &texture, size);
    let left = pixel_rgba(&pixels, size.0, 1, 1);
    let right = pixel_rgba(&pixels, size.0, tile_px + 1, 1);

    assert_eq!(left[3], 255, "left alpha must be opaque");
    assert_eq!(right[3], 255, "right alpha must be opaque");

    assert!(
        left[0] >= right[0].saturating_add(20),
        "expected left to be red-tinted by the quad: left={left:?} right={right:?}"
    );
}
