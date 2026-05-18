mod support;

use fret_core::geometry::{Point, Px};
use fret_core::scene::{DrawOrder, Paint, Scene, SceneOp};
use fret_core::{
    FillRule, FillStyle, MaterialBindingShape, MaterialDescriptor, MaterialKind, MaterialService,
    PathCommand, PathConstraints, PathService, PathStyle,
};
use fret_render_wgpu::{Renderer, WgpuContext};
use support::{pixel_rgba, render_scene_rgba8};

fn prepare_fill_path(renderer: &mut Renderer, commands: &[PathCommand]) -> fret_core::PathId {
    let (id, _metrics) = renderer.prepare(
        commands,
        PathStyle::Fill(FillStyle {
            rule: FillRule::NonZero,
        }),
        PathConstraints { scale_factor: 1.0 },
    );
    id
}

#[test]
fn path_material_paint_renders_and_is_not_degraded() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };

    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
    renderer.set_perf_enabled(true);

    let material_id = renderer
        .register_material(MaterialDescriptor {
            kind: MaterialKind::Checkerboard,
            binding: MaterialBindingShape::ParamsOnly,
        })
        .expect("register material");

    let size = (64u32, 64u32);
    let rect = [
        PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(64.0), Px(0.0))),
        PathCommand::LineTo(Point::new(Px(64.0), Px(64.0))),
        PathCommand::LineTo(Point::new(Px(0.0), Px(64.0))),
        PathCommand::Close,
    ];
    let path = prepare_fill_path(&mut renderer, &rect);

    let mut scene = Scene::default();
    scene.push(SceneOp::Path {
        order: DrawOrder(0),
        origin: Point::new(Px(0.0), Px(0.0)),
        path,
        paint: (Paint::Material {
            id: material_id,
            params: fret_core::scene::MaterialParams {
                vec4s: [
                    [1.0, 0.0, 0.0, 1.0],
                    [0.0, 1.0, 0.0, 1.0],
                    [8.0, 8.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                ],
            },
        })
        .into(),
    });

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);

    let a = pixel_rgba(&pixels, size.0, 4, 4);
    let b = pixel_rgba(&pixels, size.0, 12, 4);
    assert_ne!(a, [0, 0, 0, 0], "path should be visible");
    assert_ne!(b, [0, 0, 0, 0], "path should be visible");
    assert_ne!(a, b, "checkerboard should alternate between base/fg");

    let snap = renderer
        .take_last_frame_perf_snapshot()
        .expect("perf snapshot");
    assert_eq!(
        snap.path_material_paints_degraded_to_solid_base, 0,
        "material path paint should not be degraded under default budgets"
    );
    assert_eq!(snap.material_distinct, 1);
}
