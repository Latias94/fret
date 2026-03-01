use anyhow::Context;
use fret_core::AlphaMode;
use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, ColorSpace, DrawOrder, GradientStop, LinearGradient, MAX_STOPS, Mask, Paint,
    RadialGradient, Scene, SceneOp, TileMode, UvRect,
};
use fret_core::{FillRule, FillStyle, PathCommand, PathConstraints, PathService, PathStyle};
use fret_render::{
    ClearColor, ImageColorSpace, ImageDescriptor, RenderSceneParams, Renderer, SvgAlphaMask,
    WgpuContext, upload_alpha_mask,
};
use std::time::Instant;

fn stops_2_alpha(a: f32, b: f32) -> ([GradientStop; MAX_STOPS], u8) {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(
        0.0,
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a,
        },
    );
    stops[1] = GradientStop::new(
        1.0,
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: b,
        },
    );
    (stops, 2)
}

fn make_stripe_mask(size_px: (u32, u32)) -> SvgAlphaMask {
    let (w, h) = size_px;
    let mut alpha = vec![0u8; (w as usize) * (h as usize)];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let u = (x as u32) * 7 + (y as u32) * 3;
            let stripe = (u / 6) & 1;
            let cov = if stripe == 0 { 255 } else { 32 };
            alpha[y * (w as usize) + x] = cov;
        }
    }
    SvgAlphaMask { size_px, alpha }
}

fn prepare_clip_path(renderer: &mut Renderer) -> fret_core::PathId {
    let cmds = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(40.0), Px(8.0))),
        PathCommand::LineTo(Point::new(Px(56.0), Px(40.0))),
        PathCommand::LineTo(Point::new(Px(24.0), Px(56.0))),
        PathCommand::LineTo(Point::new(Px(8.0), Px(24.0))),
        PathCommand::Close,
    ];
    let (id, _metrics) = renderer.prepare(
        &cmds,
        PathStyle::Fill(FillStyle {
            rule: FillRule::NonZero,
        }),
        PathConstraints { scale_factor: 1.0 },
    );
    id
}

fn record_scene(
    scene: &mut Scene,
    bounds: Rect,
    mask_image: fret_core::ImageId,
    clip_path: fret_core::PathId,
    group_n: u32,
) {
    scene.clear();

    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
        background: Paint::Solid(Color {
            r: 0.05,
            g: 0.06,
            b: 0.07,
            a: 1.0,
        })
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let cell = Size::new(Px(96.0), Px(72.0));
    let pad = Px(10.0);
    let step_x = Px(cell.width.0 + pad.0);
    let step_y = Px(cell.height.0 + pad.0);
    let cols = ((bounds.size.width.0 / step_x.0).floor() as i32).max(1) as u32;

    let (lin_stops, lin_count) = stops_2_alpha(0.0, 1.0);
    let (rad_stops, rad_count) = stops_2_alpha(1.0, 0.0);

    let mut cursor = 0u32;
    let mut next_rect = || {
        let col = cursor % cols;
        let row = cursor / cols;
        cursor = cursor.saturating_add(1);
        let x = bounds.origin.x + Px(14.0) + Px(col as f32 * step_x.0);
        let y = bounds.origin.y + Px(14.0) + Px(row as f32 * step_y.0);
        Rect::new(Point::new(x, y), cell)
    };

    for i in 0..group_n {
        let rect = next_rect();
        let order = DrawOrder(1 + i);

        let clip_r = Px(12.0 + (i % 3) as f32 * 3.0);
        scene.push(SceneOp::PushClipRRect {
            rect,
            corner_radii: Corners::all(clip_r),
        });

        let lin = LinearGradient {
            start: rect.origin,
            end: Point::new(rect.origin.x + rect.size.width, rect.origin.y),
            tile_mode: TileMode::Clamp,
            color_space: ColorSpace::Srgb,
            stop_count: lin_count,
            stops: lin_stops,
        };
        scene.push(SceneOp::PushMask {
            bounds: rect,
            mask: Mask::linear_gradient(lin),
        });

        let rad = RadialGradient {
            center: Point::new(
                rect.origin.x + Px(rect.size.width.0 * 0.5),
                rect.origin.y + Px(rect.size.height.0 * 0.5),
            ),
            radius: rect.size,
            tile_mode: TileMode::Clamp,
            color_space: ColorSpace::Srgb,
            stop_count: rad_count,
            stops: rad_stops,
        };
        scene.push(SceneOp::PushMask {
            bounds: rect,
            mask: Mask::radial_gradient(rad),
        });

        scene.push(SceneOp::PushMask {
            bounds: rect,
            mask: Mask::image(mask_image, UvRect::FULL),
        });

        scene.push(SceneOp::PushClipPath {
            bounds: rect,
            origin: rect.origin,
            path: clip_path,
        });

        let hue = (i as f32 * 0.11).rem_euclid(1.0);
        let bg = Color {
            r: (0.5 + 0.5 * (hue * 6.2831853).sin()).clamp(0.0, 1.0),
            g: (0.5 + 0.5 * ((hue + 0.33) * 6.2831853).sin()).clamp(0.0, 1.0),
            b: (0.5 + 0.5 * ((hue + 0.66) * 6.2831853).sin()).clamp(0.0, 1.0),
            a: 1.0,
        };
        scene.push(SceneOp::Quad {
            order,
            rect,
            background: Paint::Solid(bg).into(),
            border: Edges::all(Px(1.25)),
            border_paint: Paint::Solid(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.35,
            })
            .into(),
            corner_radii: Corners::all(Px(10.0)),
        });

        scene.push(SceneOp::PopClip);
        scene.push(SceneOp::PopMask);
        scene.push(SceneOp::PopMask);
        scene.push(SceneOp::PopMask);
        scene.push(SceneOp::PopClip);
    }
}

fn run_headless(frames: u64, group_n: u32, wait_gpu: bool, wait_every: u64) -> anyhow::Result<()> {
    let ctx = pollster::block_on(WgpuContext::new()).context("WgpuContext::new failed")?;
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_perf_enabled(true);

    let clip_path = prepare_clip_path(&mut renderer);

    let mask = make_stripe_mask((64, 64));
    let uploaded = upload_alpha_mask(&ctx.device, &ctx.queue, &mask);
    let mask_image = renderer.register_image(ImageDescriptor {
        view: uploaded.view.clone(),
        size: uploaded.size_px,
        format: wgpu::TextureFormat::R8Unorm,
        color_space: ImageColorSpace::Linear,
        alpha_mode: AlphaMode::Premultiplied,
    });

    let viewport_size = (980u32, 720u32);
    let scale_factor = 2.0f32;
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fret headless clip-mask target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(
            Px(viewport_size.0 as f32 / scale_factor),
            Px(viewport_size.1 as f32 / scale_factor),
        ),
    );

    let mut scene = Scene::default();
    record_scene(&mut scene, bounds, mask_image, clip_path, group_n.max(1));

    let wait_every = wait_every.max(1);
    let start = Instant::now();
    for frame in 0..frames {
        let cmd = renderer.render_scene(
            &ctx.device,
            &ctx.queue,
            RenderSceneParams {
                format,
                target_view: &view,
                scene: &scene,
                clear: ClearColor::default(),
                scale_factor,
                viewport_size,
            },
        );
        ctx.queue.submit([cmd]);
        if wait_gpu && (frame % wait_every == 0) {
            let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());
        }
        if frame % 60 == 0 {
            let _ = ctx.device.poll(wgpu::PollType::Poll);
            println!("headless progress: {frame}/{frames}");
        }
    }
    let _ = ctx.device.poll(if wait_gpu {
        wgpu::PollType::wait_indefinitely()
    } else {
        wgpu::PollType::Poll
    });
    let elapsed = start.elapsed();

    println!(
        "headless: frames={} wall={:.2}s group_n={}",
        frames,
        elapsed.as_secs_f64(),
        group_n
    );

    if let Some(snap) = renderer.take_perf_snapshot() {
        let pipeline_breakdown = std::env::var_os("FRET_RENDERER_PERF_PIPELINES").is_some();
        println!(
            "headless_renderer_perf: frames={} encode={:.2}ms prepare_svg={:.2}ms prepare_text={:.2}ms draws={} (quad={} viewport={} image={} text={} path={} mask={} fs={} clipmask={}) pipelines={} binds={} (ubinds={} tbinds={}) scissor={} uniform={}KB instance={}KB vertex={}KB cache_hits={} cache_misses={} clip_path_mask_cache_entries_live={} clip_path_mask_cache_bytes_live={} clip_path_mask_cache_hits={} clip_path_mask_cache_misses={}",
            snap.frames,
            snap.encode_scene_us as f64 / 1000.0,
            snap.prepare_svg_us as f64 / 1000.0,
            snap.prepare_text_us as f64 / 1000.0,
            snap.draw_calls,
            snap.quad_draw_calls,
            snap.viewport_draw_calls,
            snap.image_draw_calls,
            snap.text_draw_calls,
            snap.path_draw_calls,
            snap.mask_draw_calls,
            snap.fullscreen_draw_calls,
            snap.clip_mask_draw_calls,
            snap.pipeline_switches,
            snap.bind_group_switches,
            snap.uniform_bind_group_switches,
            snap.texture_bind_group_switches,
            snap.scissor_sets,
            snap.uniform_bytes / 1024,
            snap.instance_bytes / 1024,
            snap.vertex_bytes / 1024,
            snap.scene_encoding_cache_hits,
            snap.scene_encoding_cache_misses,
            snap.clip_path_mask_cache_entries_live,
            snap.clip_path_mask_cache_bytes_live,
            snap.clip_path_mask_cache_hits,
            snap.clip_path_mask_cache_misses,
        );
        if pipeline_breakdown {
            println!(
                "headless_renderer_perf_pipelines: quad={} viewport={} mask={} text_mask={} text_color={} path={} path_msaa={} composite={} fullscreen={} clip_mask={}",
                snap.pipeline_switches_quad,
                snap.pipeline_switches_viewport,
                snap.pipeline_switches_mask,
                snap.pipeline_switches_text_mask,
                snap.pipeline_switches_text_color,
                snap.pipeline_switches_path,
                snap.pipeline_switches_path_msaa,
                snap.pipeline_switches_composite,
                snap.pipeline_switches_fullscreen,
                snap.pipeline_switches_clip_mask,
            );
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut frames: Option<u64> = None;
    let mut headless = false;
    let mut group_n: u32 = 32;
    let mut wait_gpu = false;
    let mut wait_every: u64 = 1;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--headless" => headless = true,
            "--frames" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("--frames requires a value");
                };
                frames = Some(value.parse()?);
            }
            "--group-n" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("--group-n requires a value");
                };
                group_n = value.parse()?;
            }
            "--wait-gpu" => wait_gpu = true,
            "--wait-every" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("--wait-every requires a value");
                };
                wait_every = value.parse()?;
            }
            "--help" | "-h" => {
                println!(
                    "Usage: fret-clip-mask-stress --headless [--frames N] [--group-n N] [--wait-gpu] [--wait-every N]"
                );
                return Ok(());
            }
            other => anyhow::bail!("unknown arg: {other}"),
        }
    }

    if !headless {
        anyhow::bail!("this tool currently supports headless runs only (pass --headless)");
    }

    let frames = frames.unwrap_or(300);
    run_headless(frames, group_n, wait_gpu, wait_every)
}
