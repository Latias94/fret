#![cfg(feature = "imui")]

use fret_core::scene::ImageSamplingHint;
use fret_core::{
    Color, ImageId, Point, Px, Rect, Size, StrokeCapV1, StrokeJoinV1, SvgFit, UvRect, ViewportFit,
};
use fret_ui::{SvgSource, UiHost};
use fret_ui_kit::imui::{
    DebugDrawImageOptions, DebugDrawOptions, DebugDrawStrokeStyle, DebugDrawSvgOptions,
    UiWriterImUiFacadeExt,
};

#[allow(dead_code)]
fn debug_draw_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    ui.debug_draw("debug-hud", |draw| {
        draw.add_line(
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(80.0), Px(24.0)),
            Color::from_srgb_hex_rgb(0xef_44_44),
            Px(1.0),
        );
        draw.add_polyline_with_style(
            [
                Point::new(Px(4.0), Px(44.0)),
                Point::new(Px(24.0), Px(28.0)),
                Point::new(Px(48.0), Px(36.0)),
            ],
            Color::from_srgb_hex_rgb(0xf5_9e_0b),
            DebugDrawStrokeStyle::new(Px(1.0))
                .with_join(StrokeJoinV1::Round)
                .with_cap(StrokeCapV1::Round)
                .with_dash(Px(4.0), Px(2.0), Px(0.0)),
            false,
        );
        draw.add_rect(
            Rect::new(Point::new(Px(8.0), Px(8.0)), Size::new(Px(64.0), Px(32.0))),
            Color::from_srgb_hex_rgb(0x22_c5_5e),
            Px(2.0),
        );
        draw.add_rect_filled(
            Rect::new(
                Point::new(Px(12.0), Px(12.0)),
                Size::new(Px(16.0), Px(16.0)),
            ),
            Color::from_srgb_hex_rgb(0x3b_82_f6),
        );
        draw.push_clip_rect(Rect::new(
            Point::new(Px(80.0), Px(0.0)),
            Size::new(Px(80.0), Px(64.0)),
        ));
        draw.add_triangle_with_style(
            Point::new(Px(88.0), Px(8.0)),
            Point::new(Px(112.0), Px(8.0)),
            Point::new(Px(100.0), Px(30.0)),
            Color::from_srgb_hex_rgb(0xa8_55_f7),
            DebugDrawStrokeStyle::new(Px(1.0)).with_join(StrokeJoinV1::Bevel),
        );
        draw.add_triangle_filled(
            Point::new(Px(88.0), Px(34.0)),
            Point::new(Px(112.0), Px(34.0)),
            Point::new(Px(100.0), Px(54.0)),
            Color::from_srgb_hex_rgb(0xec_48_99),
        );
        draw.add_circle_with_style(
            Point::new(Px(132.0), Px(20.0)),
            Px(10.0),
            Color::from_srgb_hex_rgb(0x14_b8_a6),
            DebugDrawStrokeStyle::new(Px(1.0)).with_cap(StrokeCapV1::Round),
        );
        draw.add_circle_filled(
            Point::new(Px(132.0), Px(46.0)),
            Px(8.0),
            Color::from_srgb_hex_rgb(0x06_b6_d4),
        );
        draw.pop_clip_rect();
        draw.add_image_with_options(
            Rect::new(
                Point::new(Px(148.0), Px(8.0)),
                Size::new(Px(24.0), Px(24.0)),
            ),
            ImageId::default(),
            DebugDrawImageOptions {
                fit: ViewportFit::Contain,
                sampling: ImageSamplingHint::Nearest,
                opacity: 0.8,
            },
        );
        draw.add_image_region(
            Rect::new(
                Point::new(Px(176.0), Px(8.0)),
                Size::new(Px(24.0), Px(24.0)),
            ),
            ImageId::default(),
            UvRect::FULL,
            DebugDrawImageOptions::default(),
        );
        draw.add_svg_image_with_options(
            Rect::new(
                Point::new(Px(148.0), Px(36.0)),
                Size::new(Px(24.0), Px(24.0)),
            ),
            SvgSource::Static(b"<svg/>"),
            DebugDrawSvgOptions {
                fit: SvgFit::Contain,
                opacity: 0.9,
            },
        );
        draw.add_svg_mask_icon(
            Rect::new(
                Point::new(Px(176.0), Px(36.0)),
                Size::new(Px(24.0), Px(24.0)),
            ),
            SvgSource::Static(b"<svg/>"),
            Color::from_srgb_hex_rgb(0xff_ff_ff),
        );
        draw.add_text(
            Point::new(Px(10.0), Px(54.0)),
            "debug",
            Color::from_srgb_hex_rgb(0xff_ff_ff),
            Px(12.0),
        );
    });

    ui.debug_draw_with_options(
        "debug-configured",
        DebugDrawOptions {
            test_id: Some("imui.debug_draw".into()),
            ..Default::default()
        },
        |draw| {
            draw.add_text(
                Point::new(Px(0.0), Px(0.0)),
                "configured",
                Color::from_srgb_hex_rgb(0xff_ff_ff),
                Px(10.0),
            );
        },
    );
}

#[test]
fn debug_draw_options_default_to_clipped_canvas() {
    let options = DebugDrawOptions::default();
    assert!(options.clip_to_bounds);
    assert!(options.test_id.is_none());
}
