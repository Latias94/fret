mod support;

use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{DrawOrder, MaterialParams, Paint, Scene, SceneOp};
use fret_core::{MaterialDescriptor, MaterialKind, MaterialService};
use fret_render_wgpu::{Renderer, WgpuContext};
use slotmap::Key;
use support::{pixel_rgba, render_scene_rgba8};

#[test]
fn gpu_dot_grid_material_smoke_conformance() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let id = renderer
        .register_material(MaterialDescriptor::new(MaterialKind::DotGrid))
        .expect("dot_grid material must register");

    let params = MaterialParams {
        vec4s: [
            // base (black)
            [0.0, 0.0, 0.0, 1.0],
            // fg (white)
            [1.0, 1.0, 1.0, 1.0],
            // spacing/thickness/seed
            [8.0, 8.0, 2.0, 1.0],
            // time/angle/offset
            [0.0, 0.0, 0.0, 0.0],
        ],
    };

    let size = (64u32, 64u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(64.0)));

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::Material { id, params }).into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let dot = pixel_rgba(&pixels, size.0, 4, 4);
    let bg = pixel_rgba(&pixels, size.0, 1, 1);

    assert!(
        dot[3] > 240 && bg[3] > 240,
        "expected opaque alpha: dot={dot:?} bg={bg:?}"
    );
    assert!(
        dot[0] > 200 && dot[1] > 200 && dot[2] > 200,
        "expected bright dot: dot={dot:?}"
    );
    assert!(
        bg[0] < 30 && bg[1] < 30 && bg[2] < 30,
        "expected dark background: bg={bg:?}"
    );
}

#[test]
fn unknown_material_id_degrades_to_transparent() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let params = MaterialParams {
        vec4s: [
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [8.0, 8.0, 2.0, 1.0],
            [0.0, 0.0, 0.0, 0.0],
        ],
    };

    let size = (32u32, 32u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(32.0)));

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::Material {
            id: fret_core::MaterialId::null(),
            params,
        })
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let mid = pixel_rgba(&pixels, size.0, 16, 16);
    assert!(mid[3] < 5, "expected transparent fallback: mid={mid:?}");
}

#[test]
fn material_paint_budget_pressure_degrades_to_base_color() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    renderer.set_material_paint_budget_per_frame(0);

    let id = renderer
        .register_material(MaterialDescriptor::new(MaterialKind::DotGrid))
        .expect("dot_grid material must register");

    let params = MaterialParams {
        vec4s: [
            // base (red)
            [1.0, 0.0, 0.0, 1.0],
            // fg (green)
            [0.0, 1.0, 0.0, 1.0],
            // spacing/thickness/seed
            [8.0, 8.0, 2.0, 1.0],
            // time/angle/offset
            [0.0, 0.0, 0.0, 0.0],
        ],
    };

    let size = (32u32, 32u32);
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(32.0)));

    let mut scene = Scene::default();
    scene.push(SceneOp::Quad {
        order: DrawOrder(0),
        rect,
        background: (Paint::Material { id, params }).into(),
        border: Edges::all(Px(0.0)),
        border_paint: (Paint::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let mid = pixel_rgba(&pixels, size.0, 16, 16);

    assert!(mid[3] > 240, "expected opaque alpha: mid={mid:?}");
    assert!(
        mid[0] > 230 && mid[1] < 20 && mid[2] < 20,
        "expected red base-color fallback: mid={mid:?}"
    );
}
