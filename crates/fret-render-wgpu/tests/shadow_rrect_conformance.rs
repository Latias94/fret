use fret_core::geometry::{Corners, Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, Scene, SceneOp};
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
        label: Some("shadow_rrect_conformance readback buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("shadow_rrect_conformance readback encoder"),
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
    scale_factor: f32,
) -> Vec<u8> {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("shadow_rrect_conformance output"),
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
            clear: ClearColor(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }),
            scale_factor,
            viewport_size: size,
        },
    );
    ctx.queue.submit([cb]);
    let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());
    read_texture_rgba8(&ctx.device, &ctx.queue, &texture, size)
}

fn u(v: f32, sf: f32) -> u32 {
    (v * sf).round() as u32
}

#[test]
fn shadow_rrect_conformance_masks_content_and_rounds_bottom_corners() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let mut scene = Scene::default();
    scene.push(SceneOp::ShadowRRect {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(32.0), Px(24.0)),
            Size::new(Px(48.0), Px(32.0)),
        ),
        corner_radii: Corners::all(Px(12.0)),
        offset: Point::new(Px(0.0), Px(8.0)),
        spread: Px(0.0),
        blur_radius: Px(8.0),
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.8,
        },
    });

    for scale_factor in [1.0_f32, 2.0_f32] {
        let size = (u(128.0, scale_factor), u(96.0, scale_factor));
        let pixels = render_and_readback(&ctx, &mut renderer, &scene, size, scale_factor);

        let inside_content = pixel_rgba(
            &pixels,
            size.0,
            u(56.0, scale_factor),
            u(40.0, scale_factor),
        );
        let below_center = pixel_rgba(
            &pixels,
            size.0,
            u(56.0, scale_factor),
            u(68.0, scale_factor),
        );
        let below_corner = pixel_rgba(
            &pixels,
            size.0,
            u(34.0, scale_factor),
            u(68.0, scale_factor),
        );

        assert!(
            inside_content[3] < 8,
            "shadow-only draw must keep the content box transparent; got rgba={inside_content:?} sf={scale_factor}"
        );
        assert!(
            below_center[3] > 40,
            "expected a visible bottom shadow footprint; got rgba={below_center:?} sf={scale_factor}"
        );
        assert!(
            below_corner[3] + 20 < below_center[3],
            "rounded bottom corners should stay softer than the bottom center; center={below_center:?} corner={below_corner:?} sf={scale_factor}"
        );
    }
}

#[test]
fn shadow_rrect_conformance_positive_spread_expands_hard_footprint() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let mut no_spread = Scene::default();
    no_spread.push(SceneOp::ShadowRRect {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(32.0), Px(24.0)),
            Size::new(Px(48.0), Px(32.0)),
        ),
        corner_radii: Corners::all(Px(12.0)),
        offset: Point::new(Px(0.0), Px(0.0)),
        spread: Px(0.0),
        blur_radius: Px(0.0),
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    });

    let mut positive_spread = Scene::default();
    positive_spread.push(SceneOp::ShadowRRect {
        order: DrawOrder(0),
        rect: Rect::new(
            Point::new(Px(32.0), Px(24.0)),
            Size::new(Px(48.0), Px(32.0)),
        ),
        corner_radii: Corners::all(Px(12.0)),
        offset: Point::new(Px(0.0), Px(0.0)),
        spread: Px(6.0),
        blur_radius: Px(0.0),
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        },
    });

    for scale_factor in [1.0_f32, 2.0_f32] {
        let size = (u(128.0, scale_factor), u(96.0, scale_factor));
        let no_spread_pixels =
            render_and_readback(&ctx, &mut renderer, &no_spread, size, scale_factor);
        let positive_spread_pixels =
            render_and_readback(&ctx, &mut renderer, &positive_spread, size, scale_factor);

        let spread_band_probe = (u(28.0, scale_factor), u(40.0, scale_factor));
        let content_probe = (u(56.0, scale_factor), u(40.0, scale_factor));

        let no_spread_band = pixel_rgba(
            &no_spread_pixels,
            size.0,
            spread_band_probe.0,
            spread_band_probe.1,
        );
        let positive_spread_band = pixel_rgba(
            &positive_spread_pixels,
            size.0,
            spread_band_probe.0,
            spread_band_probe.1,
        );
        let positive_spread_content = pixel_rgba(
            &positive_spread_pixels,
            size.0,
            content_probe.0,
            content_probe.1,
        );

        assert!(
            no_spread_band[3] < 8,
            "expected no-spread shadow to stay outside the probe band; got rgba={no_spread_band:?} sf={scale_factor}"
        );
        assert!(
            positive_spread_band[3] > 220,
            "expected positive spread to extend the hard footprint into the probe band; got rgba={positive_spread_band:?} sf={scale_factor}"
        );
        assert!(
            positive_spread_content[3] < 8,
            "spread must not fill the content hole; got rgba={positive_spread_content:?} sf={scale_factor}"
        );
    }
}

#[test]
fn shadow_rrect_conformance_obeys_clip_stack() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => {
            return;
        }
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let make_shadow_scene = |clipped: bool| {
        let mut scene = Scene::default();
        if clipped {
            scene.push(SceneOp::PushClipRect {
                rect: Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(128.0), Px(60.0))),
            });
        }
        scene.push(SceneOp::ShadowRRect {
            order: DrawOrder(0),
            rect: Rect::new(
                Point::new(Px(32.0), Px(24.0)),
                Size::new(Px(48.0), Px(32.0)),
            ),
            corner_radii: Corners::all(Px(12.0)),
            offset: Point::new(Px(0.0), Px(8.0)),
            spread: Px(0.0),
            blur_radius: Px(8.0),
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.8,
            },
        });
        if clipped {
            scene.push(SceneOp::PopClip);
        }
        scene
    };

    let unclipped = make_shadow_scene(false);
    let clipped = make_shadow_scene(true);

    for scale_factor in [1.0_f32, 2.0_f32] {
        let size = (u(128.0, scale_factor), u(96.0, scale_factor));
        let unclipped_pixels =
            render_and_readback(&ctx, &mut renderer, &unclipped, size, scale_factor);
        let clipped_pixels = render_and_readback(&ctx, &mut renderer, &clipped, size, scale_factor);

        let inside_clip_probe = (u(56.0, scale_factor), u(56.0, scale_factor));
        let outside_clip_probe = (u(56.0, scale_factor), u(68.0, scale_factor));

        let unclipped_inside = pixel_rgba(
            &unclipped_pixels,
            size.0,
            inside_clip_probe.0,
            inside_clip_probe.1,
        );
        let clipped_inside = pixel_rgba(
            &clipped_pixels,
            size.0,
            inside_clip_probe.0,
            inside_clip_probe.1,
        );
        let unclipped_outside = pixel_rgba(
            &unclipped_pixels,
            size.0,
            outside_clip_probe.0,
            outside_clip_probe.1,
        );
        let clipped_outside = pixel_rgba(
            &clipped_pixels,
            size.0,
            outside_clip_probe.0,
            outside_clip_probe.1,
        );

        assert!(
            unclipped_inside[3] > 20 && clipped_inside[3] > 20,
            "shadow should remain visible inside the clip; unclipped={unclipped_inside:?} clipped={clipped_inside:?} sf={scale_factor}"
        );
        assert!(
            unclipped_outside[3] > 20,
            "control scene should show shadow outside the clip boundary; got rgba={unclipped_outside:?} sf={scale_factor}"
        );
        assert!(
            clipped_outside[3] < 8,
            "clip stack must suppress shadow coverage outside the clip; got rgba={clipped_outside:?} sf={scale_factor}"
        );
    }
}
