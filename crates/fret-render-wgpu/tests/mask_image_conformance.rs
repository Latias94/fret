mod support;

use fret_core::AlphaMode;
use fret_core::geometry::{Corners, Edges, Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, Mask, Paint, Scene, SceneOp, UvRect};
use fret_render_wgpu::{
    ImageColorSpace, ImageDescriptor, Renderer, SvgAlphaMask, UploadedAlphaMask, WgpuContext,
    upload_alpha_mask,
};
use support::{pixel_rgba, render_scene_rgba8};

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

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
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

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
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

    let pixels = render_scene_rgba8(&ctx, &mut renderer, &scene, size, 1.0);
    let left = pixel_rgba(&pixels, size.0, 8, 32);
    let right = pixel_rgba(&pixels, size.0, 56, 32);

    assert!(
        left[3] <= 8 && right[3] >= 247,
        "expected inner image mask to be ignored (outer mask A wins): left={left:?} right={right:?}"
    );
}
