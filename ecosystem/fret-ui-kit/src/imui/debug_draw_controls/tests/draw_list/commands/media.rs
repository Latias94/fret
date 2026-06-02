use super::*;

#[test]
fn debug_draw_list_records_image_overlay_commands() {
    let mut list = ImUiDebugDrawList::default();
    let image = ImageId::default();
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(24.0), Px(16.0)));
    let image_options = DebugDrawImageOptions {
        fit: ViewportFit::Contain,
        sampling: ImageSamplingHint::Nearest,
        opacity: 0.5,
    };
    let svg_options = DebugDrawSvgOptions {
        fit: SvgFit::Contain,
        opacity: 0.75,
    };

    list.add_image_with_options(rect, image, image_options);
    list.add_image_region(rect, image, UvRect::FULL, image_options);
    list.add_image_quad(
        image,
        [
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(24.0), Px(0.0)),
            Point::new(Px(24.0), Px(16.0)),
            Point::new(Px(0.0), Px(16.0)),
        ],
        [
            UvPoint { u: 0.0, v: 0.0 },
            UvPoint { u: 1.0, v: 0.0 },
            UvPoint { u: 1.0, v: 1.0 },
            UvPoint { u: 0.0, v: 1.0 },
        ],
    );
    list.add_image_rounded(
        rect,
        image,
        Px(4.0),
        DebugDrawRoundCorners::TOP_LEFT | DebugDrawRoundCorners::BOTTOM_RIGHT,
    );
    list.add_image_region_rounded(
        rect,
        image,
        UvRect::FULL,
        image_options,
        Px(4.0),
        DebugDrawRoundCorners::ALL,
    );
    list.add_svg_image_with_options(rect, SvgSource::Static(b"<svg/>"), svg_options);
    list.add_svg_mask_icon_with_options(
        rect,
        SvgSource::Static(b"<svg/>"),
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        svg_options,
    );

    assert_eq!(list.command_count(), 7);
    assert!(matches!(
        list.commands[0],
        DebugDrawCommand::Media(DebugDrawMediaCommand::Image { .. })
    ));
    assert!(matches!(
        list.commands[1],
        DebugDrawCommand::Media(DebugDrawMediaCommand::ImageRegion { .. })
    ));
    assert!(matches!(
        list.commands[2],
        DebugDrawCommand::Media(DebugDrawMediaCommand::ImageQuad { .. })
    ));
    assert!(matches!(
        list.commands[3],
        DebugDrawCommand::Media(DebugDrawMediaCommand::ImageRounded { .. })
    ));
    assert!(matches!(
        list.commands[4],
        DebugDrawCommand::Media(DebugDrawMediaCommand::ImageRegionRounded { .. })
    ));
    assert!(matches!(
        list.commands[5],
        DebugDrawCommand::Media(DebugDrawMediaCommand::SvgImage { .. })
    ));
    assert!(matches!(
        list.commands[6],
        DebugDrawCommand::Media(DebugDrawMediaCommand::SvgMaskIcon { .. })
    ));
}
