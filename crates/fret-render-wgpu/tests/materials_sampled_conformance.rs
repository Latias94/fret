mod support;

use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{DrawOrder, MaterialParams, Paint, Scene, SceneOp};
use fret_core::{MaterialCatalogTextureKind, MaterialDescriptor, MaterialKind, MaterialService};
use fret_render_wgpu::{Renderer, WgpuContext};
use support::{pixel_rgba, render_scene_rgba8};

#[test]
fn sampled_noise_material_uses_catalog_texture_layer() {
    let ctx = match pollster::block_on(WgpuContext::new()) {
        Ok(ctx) => ctx,
        Err(_err) => return,
    };
    let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

    let id = renderer
        .register_material(MaterialDescriptor::sampled_with_catalog_texture(
            MaterialKind::Noise,
            MaterialCatalogTextureKind::Bayer8x8R8,
        ))
        .expect("sampled noise material must register on capable backends");

    let params = MaterialParams {
        vec4s: [
            // base (black)
            [0.0, 0.0, 0.0, 1.0],
            // fg (white)
            [1.0, 1.0, 1.0, 1.0],
            // spacing + intensity (Noise uses x as scale, y as intensity)
            [1.0, 1.0, 0.0, 0.0],
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

    // Bayer 8x8 reference (see renderer catalog upload).
    fn bayer8x8(x: u32, y: u32) -> u8 {
        const M: [[u8; 8]; 8] = [
            [0, 48, 12, 60, 3, 51, 15, 63],
            [32, 16, 44, 28, 35, 19, 47, 31],
            [8, 56, 4, 52, 11, 59, 7, 55],
            [40, 24, 36, 20, 43, 27, 39, 23],
            [2, 50, 14, 62, 1, 49, 13, 61],
            [34, 18, 46, 30, 33, 17, 45, 29],
            [10, 58, 6, 54, 9, 57, 5, 53],
            [42, 26, 38, 22, 41, 25, 37, 21],
        ];
        M[(y & 7) as usize][(x & 7) as usize]
    }

    let x = 3u32;
    let y = 1u32;
    let expected = bayer8x8(x, y).saturating_mul(4);
    let px = pixel_rgba(&pixels, size.0, x, y);

    assert!(px[3] > 240, "expected opaque alpha: px={px:?}");
    for c in [px[0], px[1], px[2]] {
        let d = c.abs_diff(expected);
        assert!(
            d <= 4,
            "expected sampled Bayer value ~{expected}, got {px:?} (abs_diff={d})"
        );
    }
}
