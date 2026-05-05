#![cfg(feature = "imui")]

use fret_core::scene::ImageSamplingHint;
use fret_core::{
    Color, ImageId, Point, Px, Rect, Size, StrokeCapV1, StrokeJoinV1, SvgFit, UvRect, ViewportFit,
};
use fret_ui::{SvgSource, UiHost};
use fret_ui_kit::imui::{
    DebugDrawImageOptions, DebugDrawOptions, DebugDrawRoundCorners, DebugDrawStrokeStyle,
    DebugDrawSvgOptions, UiWriterImUiFacadeExt,
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
        draw.add_convex_poly_filled(
            [
                Point::new(Px(52.0), Px(48.0)),
                Point::new(Px(64.0), Px(40.0)),
                Point::new(Px(76.0), Px(48.0)),
                Point::new(Px(70.0), Px(60.0)),
                Point::new(Px(58.0), Px(60.0)),
            ],
            Color::from_srgb_hex_rgb(0x10_b9_81),
        );
        draw.add_concave_poly_filled(
            [
                Point::new(Px(82.0), Px(42.0)),
                Point::new(Px(104.0), Px(42.0)),
                Point::new(Px(94.0), Px(52.0)),
                Point::new(Px(104.0), Px(62.0)),
                Point::new(Px(82.0), Px(62.0)),
            ],
            Color::from_srgb_hex_rgb(0x34_d3_99),
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
        draw.add_quad_with_style(
            Point::new(Px(36.0), Px(10.0)),
            Point::new(Px(56.0), Px(8.0)),
            Point::new(Px(60.0), Px(26.0)),
            Point::new(Px(38.0), Px(30.0)),
            Color::from_srgb_hex_rgb(0xfb_71_85),
            DebugDrawStrokeStyle::new(Px(1.0)).with_join(StrokeJoinV1::Round),
        );
        draw.add_quad_filled(
            Point::new(Px(60.0), Px(12.0)),
            Point::new(Px(76.0), Px(10.0)),
            Point::new(Px(78.0), Px(28.0)),
            Point::new(Px(62.0), Px(30.0)),
            Color::from_srgb_hex_rgb(0x2d_d4_bf),
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
        draw.add_ngon_with_style(
            Point::new(Px(132.0), Px(72.0)),
            Px(9.0),
            5,
            Color::from_srgb_hex_rgb(0x65_a3_ff),
            DebugDrawStrokeStyle::new(Px(1.0)).with_join(StrokeJoinV1::Round),
        );
        draw.add_ngon_filled(
            Point::new(Px(156.0), Px(72.0)),
            Px(8.0),
            6,
            Color::from_srgb_hex_rgb(0xc0_84_fc),
        );
        draw.add_ellipse_with_style(
            Point::new(Px(104.0), Px(88.0)),
            Size::new(Px(18.0), Px(8.0)),
            0.35,
            24,
            Color::from_srgb_hex_rgb(0x38_bd_f8),
            DebugDrawStrokeStyle::new(Px(1.0)).with_cap(StrokeCapV1::Round),
        );
        draw.add_ellipse_filled(
            Point::new(Px(148.0), Px(88.0)),
            Size::new(Px(16.0), Px(7.0)),
            0.5,
            0,
            Color::from_srgb_hex_rgb(0xf0_ab_fc),
        );
        draw.add_bezier_quadratic_with_style(
            Point::new(Px(88.0), Px(62.0)),
            Point::new(Px(104.0), Px(48.0)),
            Point::new(Px(120.0), Px(62.0)),
            Color::from_srgb_hex_rgb(0x84_cc_16),
            DebugDrawStrokeStyle::new(Px(1.0)).with_cap(StrokeCapV1::Round),
        );
        draw.add_bezier_cubic(
            Point::new(Px(124.0), Px(62.0)),
            Point::new(Px(136.0), Px(48.0)),
            Point::new(Px(148.0), Px(76.0)),
            Point::new(Px(160.0), Px(62.0)),
            Color::from_srgb_hex_rgb(0xf9_73_16),
            Px(1.0),
        );
        draw.path(|path| {
            path.line_to(Point::new(Px(164.0), Px(68.0)))
                .line_to_merge_duplicate(Point::new(Px(164.0), Px(68.0)))
                .line_to(Point::new(Px(178.0), Px(58.0)))
                .line_to(Point::new(Px(192.0), Px(68.0)))
                .rect_with_rounding(
                    Rect::new(
                        Point::new(Px(194.0), Px(54.0)),
                        Size::new(Px(18.0), Px(14.0)),
                    ),
                    Px(4.0),
                    DebugDrawRoundCorners::TOP | DebugDrawRoundCorners::BOTTOM_RIGHT,
                )
                .arc_to(
                    Point::new(Px(202.0), Px(68.0)),
                    Px(8.0),
                    std::f32::consts::PI,
                    std::f32::consts::TAU,
                    6,
                )
                .arc_to_fast(Point::new(Px(220.0), Px(68.0)), Px(8.0), 0, 3)
                .elliptical_arc_to(
                    Point::new(Px(232.0), Px(72.0)),
                    Size::new(Px(10.0), Px(5.0)),
                    0.25,
                    0.0,
                    std::f32::consts::PI,
                    6,
                )
                .bezier_quadratic_curve_to(
                    Point::new(Px(202.0), Px(54.0)),
                    Point::new(Px(212.0), Px(68.0)),
                    4,
                )
                .bezier_cubic_curve_to(
                    Point::new(Px(220.0), Px(58.0)),
                    Point::new(Px(228.0), Px(78.0)),
                    Point::new(Px(236.0), Px(68.0)),
                    4,
                );
            path.stroke_with_style(
                Color::from_srgb_hex_rgb(0xfa_e8_ff),
                DebugDrawStrokeStyle::new(Px(1.0)).with_cap(StrokeCapV1::Round),
                false,
            );
            path.line_to(Point::new(Px(166.0), Px(76.0)))
                .line_to(Point::new(Px(190.0), Px(76.0)))
                .line_to(Point::new(Px(178.0), Px(92.0)));
            path.fill_convex(Color::from_srgb_hex_rgb(0xa7_f3_d0));
            path.line_to(Point::new(Px(198.0), Px(76.0)))
                .line_to(Point::new(Px(222.0), Px(76.0)))
                .line_to(Point::new(Px(210.0), Px(84.0)))
                .line_to(Point::new(Px(222.0), Px(92.0)))
                .line_to(Point::new(Px(198.0), Px(92.0)));
            path.fill_concave(Color::from_srgb_hex_rgb(0x86_ef_ac));
        });
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
