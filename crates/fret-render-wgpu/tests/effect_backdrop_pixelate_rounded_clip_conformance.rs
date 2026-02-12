use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, DrawOrder, EffectChain, EffectMode, EffectQuality, EffectStep, Paint, Scene, SceneOp,
};
use fret_render_wgpu::{ClearColor, RenderSceneParams, Renderer, WgpuContext};
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
        label: Some("effect_backdrop_pixelate_rounded_clip_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("effect_backdrop_pixelate_rounded_clip_conformance readback encoder"),
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
        label: Some("effect_backdrop_pixelate_rounded_clip_conformance output"),
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

fn push_bounds_stripes(scene: &mut Scene, bounds: Rect, order_base: u32) {
    for i in 0..bounds.size.width.0 as u32 {
        let x = bounds.origin.x.0 + i as f32;
        let is_red = (i % 2) == 0;
        let bg = if is_red {
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        } else {
            Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            }
        };
        scene.push(SceneOp::Quad {
            order: DrawOrder(order_base + i),
            rect: Rect::new(
                Point::new(Px(x), bounds.origin.y),
                Size::new(Px(1.0), bounds.size.height),
            ),
            background: Paint::Solid(bg),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT),
            corner_radii: Default::default(),
        });
    }
}

#[test]
fn gpu_backdrop_pixelate_respects_rounded_clip_stack_on_writeback() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            // No adapter/device available (common in some headless environments).
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_intermediate_budget_bytes(u64::MAX);

    let size = (64u32, 64u32);
    let bounds = Rect::new(
        Point::new(Px(16.0), Px(16.0)),
        Size::new(Px(32.0), Px(32.0)),
    );

    let mut without_effect = Scene::default();
    without_effect.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: Paint::Solid(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });
    push_bounds_stripes(&mut without_effect, bounds, 1);
    without_effect.push(SceneOp::PushClipRRect {
        rect: bounds,
        corner_radii: Corners::all(Px(14.0)),
    });
    // Foreground marker: should remain visible regardless of effect order.
    without_effect.push(SceneOp::Quad {
        order: DrawOrder(100),
        rect: Rect::new(Point::new(Px(28.0), Px(36.0)), Size::new(Px(8.0), Px(8.0))),
        background: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });
    without_effect.push(SceneOp::PopClip);

    let mut with_effect = Scene::default();
    with_effect.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0))),
        background: Paint::Solid(Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });
    push_bounds_stripes(&mut with_effect, bounds, 1);
    with_effect.push(SceneOp::PushClipRRect {
        rect: bounds,
        corner_radii: Corners::all(Px(14.0)),
    });
    with_effect.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::Pixelate { scale: 4 }]),
        quality: EffectQuality::Auto,
    });
    with_effect.push(SceneOp::Quad {
        order: DrawOrder(100),
        rect: Rect::new(Point::new(Px(28.0), Px(36.0)), Size::new(Px(8.0), Px(8.0))),
        background: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    });
    with_effect.push(SceneOp::PopEffect);
    with_effect.push(SceneOp::PopClip);

    let direct = render_and_readback(&ctx, &mut renderer, &without_effect, size);
    let pixelated = render_and_readback(&ctx, &mut renderer, &with_effect, size);

    // Outside effect bounds: unchanged.
    let outside = pixel_rgba(&direct, size.0, 8, 32);
    let outside_pixelated = pixel_rgba(&pixelated, size.0, 8, 32);
    assert_eq!(
        outside, outside_pixelated,
        "pixels outside effect bounds must remain unchanged"
    );

    // Inside bounds but outside the rounded clip: unchanged (no leakage into corners).
    let corner_outside_clip = pixel_rgba(&direct, size.0, 17, 17);
    let corner_outside_clip_pixelated = pixel_rgba(&pixelated, size.0, 17, 17);
    assert_eq!(
        corner_outside_clip, corner_outside_clip_pixelated,
        "pixels outside the rounded clip (but inside effect bounds) must remain unchanged"
    );

    // Inside bounds near stripes: pixelation should affect at least some pixels.
    let mut any_changed = false;
    for x in 20u32..44u32 {
        let inside = pixel_rgba(&direct, size.0, x, 32);
        let inside_pixelated = pixel_rgba(&pixelated, size.0, x, 32);
        if inside != inside_pixelated {
            any_changed = true;
            break;
        }
    }
    assert!(
        any_changed,
        "expected pixelate to affect at least one pixel inside the rounded clip"
    );

    // Foreground marker must remain on top (PushEffect is a sequence point).
    let fg = pixel_rgba(&pixelated, size.0, 32, 40);
    assert!(
        fg[0] > 200 && fg[1] > 200 && fg[2] > 200 && fg[3] > 200,
        "foreground quad should remain visible on top of the backdrop pixelate"
    );
}
