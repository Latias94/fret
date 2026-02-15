use anyhow::Context;
use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{
    Color, ColorSpace, DashPatternV1, DrawOrder, GradientStop, LinearGradient, MAX_STOPS,
    MaterialParams, Paint, RadialGradient, Scene, SceneOp, TileMode,
};
use fret_core::{
    MaterialCatalogTextureKind, MaterialDescriptor, MaterialId, MaterialKind, MaterialService,
};
use fret_render::{ClearColor, RenderSceneParams, Renderer, WgpuContext};
use std::time::Instant;

#[derive(Clone, Copy)]
struct Materials {
    dot_grid: MaterialId,
    checker: MaterialId,
    noise_sampled: Option<MaterialId>,
}

fn stops_2(a: Color, b: Color) -> ([GradientStop; MAX_STOPS], u8) {
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, a);
    stops[1] = GradientStop::new(1.0, b);
    (stops, 2)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let h = (h.rem_euclid(1.0)) * 6.0;
    let i = h.floor();
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i as i32 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

fn paint_for_kind(kind: u8, rect: Rect, seed: u32, mats: Materials, sampled: bool) -> Paint {
    let (r, g, b) = hsv_to_rgb((seed as f32 * 0.031).rem_euclid(1.0), 0.65, 0.95);
    let c0 = Color { r, g, b, a: 1.0 };
    let c1 = Color {
        r: (1.0 - r).clamp(0.0, 1.0),
        g: (1.0 - g).clamp(0.0, 1.0),
        b: (1.0 - b).clamp(0.0, 1.0),
        a: 1.0,
    };

    match kind {
        0 => Paint::Solid(c0),
        1 => {
            let (stops, stop_count) = stops_2(c0, c1);
            Paint::LinearGradient(LinearGradient {
                start: rect.origin,
                end: Point::new(rect.origin.x + rect.size.width, rect.origin.y),
                tile_mode: TileMode::Clamp,
                color_space: ColorSpace::Srgb,
                stop_count,
                stops,
            })
        }
        2 => {
            let (stops, stop_count) = stops_2(c0, c1);
            Paint::RadialGradient(RadialGradient {
                center: Point::new(
                    rect.origin.x + Px(rect.size.width.0 * 0.5),
                    rect.origin.y + Px(rect.size.height.0 * 0.5),
                ),
                radius: rect.size,
                tile_mode: TileMode::Clamp,
                color_space: ColorSpace::Srgb,
                stop_count,
                stops,
            })
        }
        _ => {
            let id = if sampled {
                mats.noise_sampled.unwrap_or(mats.dot_grid)
            } else if seed % 2 == 0 {
                mats.dot_grid
            } else {
                mats.checker
            };
            let base = [c0.r, c0.g, c0.b, 1.0];
            let fg = [c1.r, c1.g, c1.b, 1.0];
            let params = MaterialParams {
                vec4s: [
                    base,
                    fg,
                    // spacing/thickness/seed (varies per instance)
                    [
                        6.0 + ((seed % 7) as f32),
                        6.0 + (((seed / 7) % 7) as f32),
                        1.0 + ((seed % 3) as f32),
                        (seed as f32) * 0.001,
                    ],
                    // time/angle/offset (kept deterministic)
                    [0.0, ((seed % 360) as f32).to_radians(), 0.0, 0.0],
                ],
            };
            Paint::Material { id, params }
        }
    }
}

fn record_scene(scene: &mut Scene, bounds: Rect, mats: Materials, group_n: u32) {
    scene.clear();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
        background: Paint::Solid(Color {
            r: 0.06,
            g: 0.07,
            b: 0.08,
            a: 1.0,
        }),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::TRANSPARENT,
        corner_radii: Corners::all(Px(0.0)),
    });

    let cell = Size::new(Px(26.0), Px(22.0));
    let pad = Px(6.0);
    let step_x = Px(cell.width.0 + pad.0);
    let step_y = Px(cell.height.0 + pad.0);
    let cols = ((bounds.size.width.0 / step_x.0).floor() as i32).max(1) as u32;

    let mut cursor = 0u32;
    let mut next_rect = || {
        let col = cursor % cols;
        let row = cursor / cols;
        cursor = cursor.saturating_add(1);
        let x = bounds.origin.x + Px(8.0) + Px(col as f32 * step_x.0);
        let y = bounds.origin.y + Px(8.0) + Px(row as f32 * step_y.0);
        Rect::new(Point::new(x, y), cell)
    };

    // Quads: cover fill_kind x border_present x border_kind (no dash here).
    for fill_kind in 0u8..=3u8 {
        // border_present = false group (border_kind collapses to 0).
        for i in 0..group_n {
            let rect = next_rect();
            let background = paint_for_kind(fill_kind, rect, 1000 + i, mats, false);
            scene.push(SceneOp::Quad {
                order: DrawOrder(1),
                rect,
                background,
                border: Edges::all(Px(0.0)),
                border_paint: Paint::TRANSPARENT,
                corner_radii: Corners::all(Px(6.0)),
            });
        }

        // border_present = true groups (border_kind varies).
        for border_kind in 0u8..=3u8 {
            for i in 0..group_n {
                let rect = next_rect();
                let background = paint_for_kind(fill_kind, rect, 2000 + i, mats, false);
                let border_paint = paint_for_kind(border_kind, rect, 3000 + i, mats, false);
                scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect,
                    background,
                    border: Edges::all(Px(1.5)),
                    border_paint,
                    corner_radii: Corners::all(Px(6.0)),
                });
            }
        }
    }

    // StrokeRRect: cover border_kind x dash_enabled (fill_kind=transparent path in the quad shader).
    for dash_enabled in [false, true] {
        for border_kind in 0u8..=3u8 {
            for i in 0..group_n {
                let rect = next_rect();
                let stroke_paint = paint_for_kind(border_kind, rect, 4000 + i, mats, false);
                let dash = dash_enabled.then(|| DashPatternV1::new(Px(5.0), Px(3.0), Px(0.0)));
                scene.push(SceneOp::StrokeRRect {
                    order: DrawOrder(2),
                    rect,
                    stroke: Edges::all(Px(1.5)),
                    stroke_paint,
                    corner_radii: Corners::all(Px(6.0)),
                    style: fret_core::scene::StrokeStyleV1 { dash },
                });
            }
        }
    }

    // Materials: ensure both params-only and sampled paths are exercised when supported.
    // Keep these grouped to make pipeline switches sensitive to variant-key expansions.
    for sampled in [false, true] {
        for i in 0..(group_n * 2) {
            let rect = next_rect();
            let background = paint_for_kind(3, rect, 5000 + i, mats, sampled);
            scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background,
                border: Edges::all(Px(0.0)),
                border_paint: Paint::TRANSPARENT,
                corner_radii: Corners::all(Px(6.0)),
            });
        }
    }
}

fn run_headless(frames: u64, group_n: u32, wait_gpu: bool, wait_every: u64) -> anyhow::Result<()> {
    let ctx = pollster::block_on(WgpuContext::new()).context("WgpuContext::new failed")?;
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_perf_enabled(true);
    renderer.set_material_paint_budget_per_frame(4096);
    renderer.set_material_distinct_budget_per_frame(4096);

    let dot_grid = renderer
        .register_material(MaterialDescriptor::new(MaterialKind::DotGrid))
        .map_err(|err| anyhow::anyhow!("register dot_grid material failed: {err:?}"))?;
    let checker = renderer
        .register_material(MaterialDescriptor::new(MaterialKind::Checkerboard))
        .map_err(|err| anyhow::anyhow!("register checkerboard material failed: {err:?}"))?;
    let noise_sampled = renderer
        .register_material(MaterialDescriptor::sampled_with_catalog_texture(
            MaterialKind::Noise,
            MaterialCatalogTextureKind::Bayer8x8R8,
        ))
        .ok();

    let mats = Materials {
        dot_grid,
        checker,
        noise_sampled,
    };

    let viewport_size = (980u32, 720u32);
    let scale_factor = 2.0f32;
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fret headless quad-material target"),
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
    record_scene(&mut scene, bounds, mats, group_n.max(1));

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
        "headless: frames={} wall={:.2}s group_n={} sampled_supported={}",
        frames,
        elapsed.as_secs_f64(),
        group_n,
        mats.noise_sampled.is_some()
    );

    if let Some(snap) = renderer.take_perf_snapshot() {
        let pipeline_breakdown = std::env::var_os("FRET_RENDERER_PERF_PIPELINES").is_some();
        println!(
            "headless_renderer_perf: frames={} encode={:.2}ms prepare_svg={:.2}ms prepare_text={:.2}ms draws={} (quad={} viewport={} image={} text={} path={} mask={} fs={} clipmask={}) pipelines={} binds={} (ubinds={} tbinds={}) scissor={} uniform={}KB instance={}KB vertex={}KB cache_hits={} cache_misses={}",
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
            snap.scene_encoding_cache_misses
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
                    "Usage: fret-quad-material-stress --headless [--frames N] [--group-n N] [--wait-gpu] [--wait-every N]"
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
