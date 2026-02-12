use fret_core::geometry::{Edges, Point, Px, Rect, Size};
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
        label: Some("effect_backdrop_pixelate_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("effect_backdrop_pixelate_conformance readback encoder"),
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
        label: Some("effect_backdrop_pixelate_conformance output"),
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
fn gpu_backdrop_pixelate_is_scissored_and_preserves_ordering() {
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
    let mut base = Scene::default();

    // Green background for outside-region invariance.
    base.push(SceneOp::Quad {
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

    // Vertical 1px stripes inside x=[24, 40): alternating red/blue.
    for i in 0..16u32 {
        let x = 24.0 + i as f32;
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
        base.push(SceneOp::Quad {
            order: DrawOrder(1 + i),
            rect: Rect::new(Point::new(Px(x), Px(0.0)), Size::new(Px(1.0), Px(64.0))),
            background: Paint::Solid(bg),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT),
            corner_radii: Default::default(),
        });
    }

    // Foreground marker quad.
    let foreground = SceneOp::Quad {
        order: DrawOrder(100),
        rect: Rect::new(
            Point::new(Px(26.0), Px(48.0)),
            Size::new(Px(12.0), Px(12.0)),
        ),
        background: Paint::Solid(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT),
        corner_radii: Default::default(),
    };

    let mut without_effect = base.clone();
    without_effect.push(foreground);

    let mut with_effect = base;
    with_effect.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(24.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::Pixelate { scale: 4 }]),
        quality: EffectQuality::Auto,
    });
    with_effect.push(foreground);
    with_effect.push(SceneOp::PopEffect);

    let direct = render_and_readback(&ctx, &mut renderer, &without_effect, size);
    let pixelated = render_and_readback(&ctx, &mut renderer, &with_effect, size);

    // Outside bounds: unchanged (green).
    let outside = pixel_rgba(&direct, size.0, 8, 32);
    let outside_pixelated = pixel_rgba(&pixelated, size.0, 8, 32);
    assert_eq!(
        outside, outside_pixelated,
        "pixels outside effect bounds must remain unchanged"
    );

    // Inside bounds: adjacent pixels that used to alternate should collapse to the same value.
    let a_direct = pixel_rgba(&direct, size.0, 25, 32);
    let b_direct = pixel_rgba(&direct, size.0, 26, 32);
    assert_ne!(
        a_direct, b_direct,
        "source stripes should alternate by column"
    );

    let a_pix = pixel_rgba(&pixelated, size.0, 25, 32);
    let b_pix = pixel_rgba(&pixelated, size.0, 26, 32);
    assert_eq!(
        a_pix, b_pix,
        "pixelate should make adjacent pixels within a block share the same value"
    );

    // Foreground must remain visible on top (sequence point).
    let fg = pixel_rgba(&pixelated, size.0, 32, 56);
    assert!(
        fg[0] > 200 && fg[1] > 200 && fg[2] > 200 && fg[3] > 200,
        "foreground quad should remain visible on top of the pixelated backdrop"
    );
}

#[test]
fn gpu_backdrop_pixelate_is_anchored_to_effect_bounds() {
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
    let mut base = Scene::default();

    // Green background for detecting bleed from outside the effect bounds.
    base.push(SceneOp::Quad {
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

    // Vertical 1px stripes inside x=[25, 41): repeating pattern [R, B, B].
    // This intentionally starts at a non-multiple of the pixelate scale.
    for i in 0..16u32 {
        let x = 25.0 + i as f32;
        let bg = match i % 3 {
            0 => Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            _ => Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            },
        };
        base.push(SceneOp::Quad {
            order: DrawOrder(1 + i),
            rect: Rect::new(Point::new(Px(x), Px(0.0)), Size::new(Px(1.0), Px(64.0))),
            background: Paint::Solid(bg),
            border: Edges::all(Px(0.0)),
            border_paint: Paint::Solid(Color::TRANSPARENT),
            corner_radii: Default::default(),
        });
    }

    let mut with_effect = base;
    with_effect.push(SceneOp::PushEffect {
        bounds: Rect::new(Point::new(Px(25.0), Px(0.0)), Size::new(Px(16.0), Px(64.0))),
        mode: EffectMode::Backdrop,
        chain: EffectChain::from_steps(&[EffectStep::Pixelate { scale: 4 }]),
        quality: EffectQuality::Auto,
    });
    // Foreground marker quad (also ensures the effect scope has content).
    with_effect.push(SceneOp::Quad {
        order: DrawOrder(100),
        rect: Rect::new(
            Point::new(Px(28.0), Px(48.0)),
            Size::new(Px(12.0), Px(12.0)),
        ),
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

    let pixelated = render_and_readback(&ctx, &mut renderer, &with_effect, size);

    let outside = pixel_rgba(&pixelated, size.0, 8, 32);
    let a = pixel_rgba(&pixelated, size.0, 25, 32);
    let b = pixel_rgba(&pixelated, size.0, 28, 32);
    let next_block = pixel_rgba(&pixelated, size.0, 29, 32);

    assert_ne!(
        outside, a,
        "pixelate must not sample from outside the effect bounds when bounds are not scale-aligned"
    );
    assert_eq!(
        a, b,
        "pixels within a pixelate block should share the same sampled value"
    );
    assert_ne!(
        a, next_block,
        "pixelate blocks should be anchored to the effect bounds, not the window origin"
    );
}
