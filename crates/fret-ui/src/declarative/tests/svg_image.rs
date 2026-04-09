use super::*;

use crate::SvgSource;
use crate::element::{Length, SvgImageProps};
use fret_core::SvgFit;

#[test]
fn svg_image_props_paint_to_svg_image_scene_op() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();
    let svg_bytes: &'static [u8] =
        br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" fill="#00ff88"/></svg>"##;

    let node = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "svg-image-scene-op",
        |cx| {
            let mut image = SvgImageProps::new(SvgSource::Static(svg_bytes));
            image.layout.size.width = Length::Px(Px(24.0));
            image.layout.size.height = Length::Px(Px(18.0));
            image.fit = SvgFit::Width;
            image.opacity = 0.75;
            vec![cx.svg_image_props(image)]
        },
    );
    ui.set_root(node);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let svg_image = scene.ops().iter().find_map(|op| match op {
        SceneOp::SvgImage {
            rect, fit, opacity, ..
        } => Some((*rect, *fit, *opacity)),
        _ => None,
    });
    assert_eq!(
        svg_image,
        Some((
            Rect::new(
                fret_core::Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(24.0), Px(18.0))
            ),
            SvgFit::Width,
            0.75,
        )),
        "expected SvgImageProps to lower to SceneOp::SvgImage with the requested fit and opacity"
    );
    assert_eq!(
        services.svg_register_calls, 1,
        "expected a static SvgImage source to register once during paint"
    );
}
