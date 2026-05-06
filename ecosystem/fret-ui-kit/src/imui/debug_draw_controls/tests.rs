use super::*;

use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Corners, Size};
use fret_ui::element::ElementKind;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(240.0)),
    )
}

fn empty_commands() -> Arc<[DebugDrawCommand]> {
    Arc::from(Vec::<DebugDrawCommand>::new().into_boxed_slice())
}

fn assert_point_near(actual: Point, expected: Point) {
    assert!(
        (actual.x.0 - expected.x.0).abs() <= 0.000_1,
        "x mismatch: actual {:?}, expected {:?}",
        actual,
        expected
    );
    assert!(
        (actual.y.0 - expected.y.0).abs() <= 0.000_1,
        "y mismatch: actual {:?}, expected {:?}",
        actual,
        expected
    );
}

#[test]
fn debug_draw_default_element_stays_noninteractive_canvas() {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui::elements::with_element_cx(&mut app, window, bounds(), "debug-draw.canvas", |cx| {
        let mut response = ResponseExt::default();
        let element = debug_draw_element(
            cx,
            empty_commands(),
            DebugDrawOptions {
                test_id: Some(Arc::from("imui.debug_draw")),
                ..Default::default()
            },
            &mut response,
        );

        assert!(matches!(element.kind, ElementKind::Canvas(_)));
        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|decoration| decoration.test_id.as_deref()),
            Some("imui.debug_draw")
        );
        assert!(!response.enabled);
    });
}

#[test]
fn debug_draw_interaction_wraps_canvas_in_pressable_response_surface() {
    let window = AppWindowId::default();
    let mut app = App::new();

    fret_ui::elements::with_element_cx(&mut app, window, bounds(), "debug-draw.pressable", |cx| {
        let mut response = ResponseExt::default();
        let element = debug_draw_element(
            cx,
            empty_commands(),
            DebugDrawOptions {
                test_id: Some(Arc::from("imui.debug_draw.interactive")),
                interaction: DebugDrawInteractionOptions::enabled()
                    .focusable(true)
                    .with_a11y_label("Debug draw canvas"),
                ..Default::default()
            },
            &mut response,
        );

        let ElementKind::Pressable(props) = &element.kind else {
            panic!("interactive debug draw should wrap the canvas in a pressable");
        };
        assert!(props.enabled);
        assert!(props.focusable);
        assert_eq!(props.a11y.label.as_deref(), Some("Debug draw canvas"));
        assert_eq!(props.a11y.test_id.as_deref(), None);
        assert_eq!(element.children.len(), 1);
        assert!(matches!(element.children[0].kind, ElementKind::Canvas(_)));
        assert_eq!(
            element.children[0]
                .semantics_decoration
                .as_ref()
                .and_then(|decoration| decoration.test_id.as_deref()),
            Some("imui.debug_draw.interactive")
        );
        assert!(response.enabled);
    });
}

#[test]
fn debug_draw_list_records_commands_in_order() {
    let mut list = ImUiDebugDrawList::default();
    assert!(list.is_empty());

    list.add_line(
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(10.0), Px(10.0)),
        Color::from_srgb_hex_rgb(0xff_00_00),
        Px(1.0),
    );
    list.add_polyline(
        [
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(4.0), Px(8.0)),
            Point::new(Px(8.0), Px(2.0)),
        ],
        Color::from_srgb_hex_rgb(0xff_ff_00),
        Px(1.0),
        false,
    );
    list.add_convex_poly_filled(
        [
            Point::new(Px(12.0), Px(62.0)),
            Point::new(Px(24.0), Px(54.0)),
            Point::new(Px(36.0), Px(62.0)),
            Point::new(Px(30.0), Px(76.0)),
            Point::new(Px(18.0), Px(76.0)),
        ],
        Color::from_srgb_hex_rgb(0x10_b9_81),
    );
    list.add_rect(
        Rect::new(Point::new(Px(2.0), Px(3.0)), Size::new(Px(4.0), Px(5.0))),
        Color::from_srgb_hex_rgb(0x00_ff_00),
        Px(2.0),
    );
    list.add_rect_filled(
        Rect::new(Point::new(Px(1.0), Px(1.0)), Size::new(Px(2.0), Px(2.0))),
        Color::from_srgb_hex_rgb(0x00_00_ff),
    );
    list.add_rect_filled_multi_color(
        Rect::new(Point::new(Px(4.0), Px(1.0)), Size::new(Px(6.0), Px(5.0))),
        Color::from_srgb_hex_rgb(0xff_00_00),
        Color::from_srgb_hex_rgb(0x00_ff_00),
        Color::from_srgb_hex_rgb(0x00_00_ff),
        Color::from_srgb_hex_rgb(0xff_ff_00),
    );
    list.add_quad(
        Point::new(Px(8.0), Px(8.0)),
        Point::new(Px(18.0), Px(6.0)),
        Point::new(Px(22.0), Px(18.0)),
        Point::new(Px(10.0), Px(20.0)),
        Color::from_srgb_hex_rgb(0xfb_71_85),
        Px(1.0),
    );
    list.add_quad_filled(
        Point::new(Px(24.0), Px(8.0)),
        Point::new(Px(34.0), Px(6.0)),
        Point::new(Px(38.0), Px(18.0)),
        Point::new(Px(26.0), Px(20.0)),
        Color::from_srgb_hex_rgb(0x2d_d4_bf),
    );
    list.add_triangle(
        Point::new(Px(1.0), Px(1.0)),
        Point::new(Px(5.0), Px(1.0)),
        Point::new(Px(3.0), Px(4.0)),
        Color::from_srgb_hex_rgb(0xff_00_ff),
        Px(1.0),
    );
    list.add_triangle_filled(
        Point::new(Px(2.0), Px(2.0)),
        Point::new(Px(6.0), Px(2.0)),
        Point::new(Px(4.0), Px(5.0)),
        Color::from_srgb_hex_rgb(0x00_ff_ff),
    );
    list.add_circle(
        Point::new(Px(20.0), Px(20.0)),
        Px(8.0),
        Color::from_srgb_hex_rgb(0xff_aa_00),
        Px(2.0),
    );
    list.add_circle_filled(
        Point::new(Px(40.0), Px(20.0)),
        Px(6.0),
        Color::from_srgb_hex_rgb(0xaa_00_ff),
    );
    list.add_ngon(
        Point::new(Px(56.0), Px(20.0)),
        Px(8.0),
        5,
        Color::from_srgb_hex_rgb(0x65_a3_ff),
        Px(1.0),
    );
    list.add_ngon_filled(
        Point::new(Px(76.0), Px(20.0)),
        Px(6.0),
        6,
        Color::from_srgb_hex_rgb(0xc0_84_fc),
    );
    list.add_ellipse(
        Point::new(Px(96.0), Px(20.0)),
        Size::new(Px(12.0), Px(6.0)),
        0.25,
        16,
        Color::from_srgb_hex_rgb(0x38_bd_f8),
        Px(1.0),
    );
    list.add_ellipse_filled(
        Point::new(Px(122.0), Px(20.0)),
        Size::new(Px(10.0), Px(5.0)),
        0.5,
        0,
        Color::from_srgb_hex_rgb(0xf0_ab_fc),
    );
    list.add_bezier_quadratic(
        Point::new(Px(2.0), Px(60.0)),
        Point::new(Px(20.0), Px(42.0)),
        Point::new(Px(38.0), Px(60.0)),
        Color::from_srgb_hex_rgb(0x22_d3_ee),
        Px(1.0),
    );
    list.add_bezier_cubic(
        Point::new(Px(42.0), Px(60.0)),
        Point::new(Px(54.0), Px(42.0)),
        Point::new(Px(70.0), Px(78.0)),
        Point::new(Px(82.0), Px(60.0)),
        Color::from_srgb_hex_rgb(0xf4_72_b6),
        Px(1.0),
    );
    list.add_text(
        Point::new(Px(4.0), Px(5.0)),
        "debug",
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(12.0),
    );

    assert_eq!(list.command_count(), 19);
    assert!(matches!(list.commands[0], DebugDrawCommand::Line { .. }));
    assert!(matches!(
        list.commands[1],
        DebugDrawCommand::Polyline { .. }
    ));
    assert!(matches!(
        list.commands[2],
        DebugDrawCommand::ConvexPolyFilled { .. }
    ));
    assert!(matches!(list.commands[3], DebugDrawCommand::Rect { .. }));
    assert!(matches!(
        list.commands[4],
        DebugDrawCommand::RectFilled { .. }
    ));
    assert!(matches!(
        list.commands[5],
        DebugDrawCommand::RectFilledMultiColor { .. }
    ));
    assert!(matches!(list.commands[6], DebugDrawCommand::Quad { .. }));
    assert!(matches!(
        list.commands[7],
        DebugDrawCommand::QuadFilled { .. }
    ));
    assert!(matches!(
        list.commands[8],
        DebugDrawCommand::Triangle { .. }
    ));
    assert!(matches!(
        list.commands[9],
        DebugDrawCommand::TriangleFilled { .. }
    ));
    assert!(matches!(list.commands[10], DebugDrawCommand::Circle { .. }));
    assert!(matches!(
        list.commands[11],
        DebugDrawCommand::CircleFilled { .. }
    ));
    assert!(matches!(list.commands[12], DebugDrawCommand::Ngon { .. }));
    assert!(matches!(
        list.commands[13],
        DebugDrawCommand::NgonFilled { .. }
    ));
    assert!(matches!(
        list.commands[14],
        DebugDrawCommand::Ellipse { .. }
    ));
    assert!(matches!(
        list.commands[15],
        DebugDrawCommand::EllipseFilled { .. }
    ));
    assert!(matches!(
        list.commands[16],
        DebugDrawCommand::BezierQuadratic { .. }
    ));
    assert!(matches!(
        list.commands[17],
        DebugDrawCommand::BezierCubic { .. }
    ));
    assert!(matches!(list.commands[18], DebugDrawCommand::Text { .. }));
}

#[test]
fn debug_draw_list_records_triangle_mesh_commands() {
    let mut list = ImUiDebugDrawList::default();
    let vertices = [
        DebugDrawVertex::colored(
            Point::new(Px(0.0), Px(0.0)),
            Color::from_srgb_hex_rgb(0xff_00_00),
        ),
        DebugDrawVertex::colored(
            Point::new(Px(8.0), Px(0.0)),
            Color::from_srgb_hex_rgb(0x00_ff_00),
        ),
        DebugDrawVertex::colored(
            Point::new(Px(4.0), Px(8.0)),
            Color::from_srgb_hex_rgb(0x00_00_ff),
        ),
    ];
    list.add_triangle_mesh(vertices, [0, 1, 2]);
    list.add_image_triangle_mesh_with_options(
        ImageId::default(),
        vertices.map(|vertex| {
            DebugDrawVertex::new(vertex.position, UvPoint::new(0.5, 0.25), vertex.color)
        }),
        [0, 1, 2],
        DebugDrawImageMeshOptions {
            sampling: ImageSamplingHint::Nearest,
            opacity: 0.5,
        },
    );

    assert_eq!(list.command_count(), 2);
    assert!(matches!(
        list.commands[0],
        DebugDrawCommand::TriangleMesh { .. }
    ));
    let DebugDrawCommand::ImageTriangleMesh { options, .. } = &list.commands[1] else {
        panic!("expected image triangle mesh command");
    };
    assert_eq!(options.sampling, ImageSamplingHint::Nearest);
    assert_eq!(options.opacity, 0.5);
}

#[test]
fn debug_draw_list_reports_command_summaries_in_merge_order() {
    let mut list = ImUiDebugDrawList::default();
    let vertices = [
        DebugDrawVertex::colored(
            Point::new(Px(0.0), Px(0.0)),
            Color::from_srgb_hex_rgb(0xff_00_00),
        ),
        DebugDrawVertex::colored(
            Point::new(Px(8.0), Px(0.0)),
            Color::from_srgb_hex_rgb(0x00_ff_00),
        ),
        DebugDrawVertex::colored(
            Point::new(Px(4.0), Px(8.0)),
            Color::from_srgb_hex_rgb(0x00_00_ff),
        ),
    ];

    list.channels_split(3);
    list.add_rect_filled(
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(4.0), Px(4.0))),
        Color::from_srgb_hex_rgb(0xff_ff_ff),
    );
    list.channels_set_current(2);
    list.add_line(
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(8.0), Px(8.0)),
        Color::from_srgb_hex_rgb(0xff_00_00),
        Px(1.0),
    );
    list.channels_set_current(1);
    list.add_image_triangle_mesh(ImageId::default(), vertices, [0, 1, 2]);

    let summaries = list.command_summaries();
    assert_eq!(summaries.len(), 3);
    assert_eq!(
        summaries
            .iter()
            .map(|summary| (summary.channel, summary.kind))
            .collect::<Vec<_>>(),
        vec![
            (Some(0), DebugDrawCommandKind::RectFilled),
            (Some(1), DebugDrawCommandKind::ImageTriangleMesh),
            (Some(2), DebugDrawCommandKind::Line),
        ]
    );
    assert_eq!(summaries[1].image, Some(ImageId::default()));
    assert_eq!(summaries[1].vertex_count, 3);
    assert_eq!(summaries[1].index_count, 3);
    assert_eq!(summaries[1].triangle_count, 1);
}

#[test]
fn debug_draw_list_summary_counts_visible_command_classes() {
    let mut list = ImUiDebugDrawList::default();
    list.push_clip_rect(Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(16.0), Px(16.0)),
    ));
    list.add_image(
        Rect::new(Point::new(Px(1.0), Px(1.0)), Size::new(Px(8.0), Px(8.0))),
        ImageId::default(),
    );
    list.add_svg_image(
        Rect::new(Point::new(Px(2.0), Px(2.0)), Size::new(Px(8.0), Px(8.0))),
        SvgSource::Static(b"<svg/>"),
    );
    list.add_rect_filled_multi_color(
        Rect::new(Point::new(Px(3.0), Px(3.0)), Size::new(Px(10.0), Px(10.0))),
        Color::from_srgb_hex_rgb(0xff_00_00),
        Color::from_srgb_hex_rgb(0x00_ff_00),
        Color::from_srgb_hex_rgb(0x00_00_ff),
        Color::from_srgb_hex_rgb(0xff_ff_ff),
    );
    list.add_text(
        Point::new(Px(4.0), Px(4.0)),
        "debug",
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(12.0),
    );
    list.pop_clip_rect();

    let summary = list.list_summary();
    assert_eq!(summary.command_count, 6);
    assert_eq!(summary.clip_push_count, 1);
    assert_eq!(summary.clip_pop_count, 1);
    assert_eq!(summary.image_command_count, 1);
    assert_eq!(summary.svg_command_count, 1);
    assert_eq!(summary.text_command_count, 1);
    assert_eq!(summary.vertex_count, 4);
    assert_eq!(summary.index_count, 6);
    assert_eq!(summary.triangle_count, 2);
}

#[test]
fn debug_draw_command_summaries_track_effective_clip_stack() {
    let outer = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(32.0)));
    let inner = Rect::new(Point::new(Px(4.0), Px(4.0)), Size::new(Px(12.0), Px(12.0)));

    let mut list = ImUiDebugDrawList::default();
    list.add_line(
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(8.0), Px(8.0)),
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(1.0),
    );
    list.push_clip_rect(outer);
    list.add_rect_filled(
        Rect::new(Point::new(Px(2.0), Px(2.0)), Size::new(Px(6.0), Px(6.0))),
        Color::from_srgb_hex_rgb(0xff_00_00),
    );
    list.push_clip_rect(inner);
    list.add_text(
        Point::new(Px(6.0), Px(6.0)),
        "clipped",
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(12.0),
    );
    list.pop_clip_rect();
    list.add_image(
        Rect::new(Point::new(Px(8.0), Px(8.0)), Size::new(Px(6.0), Px(6.0))),
        ImageId::default(),
    );
    list.pop_clip_rect();

    let summaries = list.command_summaries();
    assert_eq!(summaries[0].clip_rect, None);
    assert_eq!(summaries[0].clip_depth, 0);
    assert_eq!(summaries[1].clip_rect, Some(outer));
    assert_eq!(summaries[1].clip_depth, 1);
    assert_eq!(summaries[2].clip_rect, Some(outer));
    assert_eq!(summaries[2].clip_depth, 1);
    assert_eq!(summaries[3].clip_rect, Some(inner));
    assert_eq!(summaries[3].clip_depth, 2);
    assert_eq!(summaries[4].clip_rect, Some(inner));
    assert_eq!(summaries[4].clip_depth, 2);
    assert_eq!(summaries[5].clip_rect, Some(outer));
    assert_eq!(summaries[5].clip_depth, 1);
    assert_eq!(summaries[6].clip_rect, Some(outer));
    assert_eq!(summaries[6].clip_depth, 1);
    assert_eq!(summaries[7].clip_rect, None);
    assert_eq!(summaries[7].clip_depth, 0);

    let summary = list.list_summary();
    assert_eq!(summary.max_clip_depth, 2);
    assert_eq!(summary.final_clip_depth, 0);
}

#[test]
fn debug_draw_list_records_clip_stack_commands() {
    let mut list = ImUiDebugDrawList::default();
    list.push_clip_rect(Rect::new(
        Point::new(Px(2.0), Px(3.0)),
        Size::new(Px(40.0), Px(50.0)),
    ));
    list.pop_clip_rect();

    assert_eq!(list.command_count(), 2);
    assert!(matches!(
        list.commands[0],
        DebugDrawCommand::PushClipRect { .. }
    ));
    assert!(matches!(list.commands[1], DebugDrawCommand::PopClipRect));
}

#[test]
fn debug_draw_channels_merge_in_channel_order() {
    let mut list = ImUiDebugDrawList::default();
    list.add_line(
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(1.0), Px(1.0)),
        Color::from_srgb_hex_rgb(0xff_00_00),
        Px(1.0),
    );

    list.channels_split(3);
    list.channels_set_current(2);
    list.add_text(
        Point::new(Px(8.0), Px(8.0)),
        "foreground",
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(12.0),
    );
    list.channels_set_current(1);
    list.add_rect_filled(
        Rect::new(Point::new(Px(2.0), Px(2.0)), Size::new(Px(4.0), Px(4.0))),
        Color::from_srgb_hex_rgb(0x00_ff_00),
    );
    list.channels_set_current(0);
    list.add_circle_filled(
        Point::new(Px(6.0), Px(6.0)),
        Px(2.0),
        Color::from_srgb_hex_rgb(0x00_00_ff),
    );

    assert_eq!(list.command_count(), 4);
    list.channels_merge();

    assert_eq!(list.command_count(), 4);
    assert!(matches!(list.commands[0], DebugDrawCommand::Line { .. }));
    assert!(matches!(
        list.commands[1],
        DebugDrawCommand::CircleFilled { .. }
    ));
    assert!(matches!(
        list.commands[2],
        DebugDrawCommand::RectFilled { .. }
    ));
    assert!(matches!(list.commands[3], DebugDrawCommand::Text { .. }));
}

#[test]
fn debug_draw_channels_ignore_invalid_channel_switches() {
    let mut list = ImUiDebugDrawList::default();
    list.channels_split(2);
    list.channels_set_current(4);
    list.add_text(
        Point::new(Px(0.0), Px(0.0)),
        "still-channel-zero",
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        Px(12.0),
    );
    list.channels_merge();

    assert_eq!(list.command_count(), 1);
    assert!(matches!(list.commands[0], DebugDrawCommand::Text { .. }));
}

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
    assert!(matches!(list.commands[0], DebugDrawCommand::Image { .. }));
    assert!(matches!(
        list.commands[1],
        DebugDrawCommand::ImageRegion { .. }
    ));
    assert!(matches!(
        list.commands[2],
        DebugDrawCommand::ImageQuad { .. }
    ));
    assert!(matches!(
        list.commands[3],
        DebugDrawCommand::ImageRounded { .. }
    ));
    assert!(matches!(
        list.commands[4],
        DebugDrawCommand::ImageRegionRounded { .. }
    ));
    assert!(matches!(
        list.commands[5],
        DebugDrawCommand::SvgImage { .. }
    ));
    assert!(matches!(
        list.commands[6],
        DebugDrawCommand::SvgMaskIcon { .. }
    ));
}

#[test]
fn debug_draw_list_records_concave_poly_fill_command() {
    let mut list = ImUiDebugDrawList::default();
    let points = [
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(18.0), Px(0.0)),
        Point::new(Px(10.0), Px(8.0)),
        Point::new(Px(18.0), Px(16.0)),
        Point::new(Px(0.0), Px(16.0)),
    ];

    list.add_concave_poly_filled(points, Color::from_srgb_hex_rgb(0xff_ff_ff));

    let DebugDrawCommand::ConcavePolyFilled {
        points: recorded, ..
    } = &list.commands[0]
    else {
        panic!("concave polygon fill should record a dedicated command");
    };
    assert_eq!(&**recorded, &points);
}

#[test]
fn debug_draw_path_builder_records_stroke_and_fill_commands() {
    let mut list = ImUiDebugDrawList::default();
    let p0 = Point::new(Px(0.0), Px(0.0));
    let p1 = Point::new(Px(12.0), Px(0.0));
    let p2 = Point::new(Px(12.0), Px(10.0));
    let p3 = Point::new(Px(0.0), Px(10.0));

    list.path(|path| {
        assert!(path.is_empty());
        path.line_to(p0)
            .line_to_merge_duplicate(p0)
            .line_to_merge_duplicate(p1)
            .line_to(p2);
        assert_eq!(path.point_count(), 3);

        path.stroke_with_style(
            Color::from_srgb_hex_rgb(0xff_aa_00),
            DebugDrawStrokeStyle::new(Px(2.0)).with_join(StrokeJoinV1::Round),
            true,
        );
        assert!(path.is_empty());

        path.line_to(p0).line_to(p1).line_to(p2).line_to(p3);
        path.fill_convex(Color::from_srgb_hex_rgb(0x22_c5_5e));
    });

    assert_eq!(list.command_count(), 2);
    let DebugDrawCommand::Polyline {
        points,
        style,
        closed,
        ..
    } = &list.commands[0]
    else {
        panic!("path stroke should record a polyline command");
    };
    assert_eq!(&**points, &[p0, p1, p2]);
    assert_eq!(style.width, Px(2.0));
    assert_eq!(style.join, StrokeJoinV1::Round);
    assert!(*closed);

    let DebugDrawCommand::ConvexPolyFilled { points, .. } = &list.commands[1] else {
        panic!("path fill should record a convex fill command");
    };
    assert_eq!(&**points, &[p0, p1, p2, p3]);
}

#[test]
fn debug_draw_path_builder_records_concave_fill_command() {
    let mut list = ImUiDebugDrawList::default();
    let points = [
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(18.0), Px(0.0)),
        Point::new(Px(10.0), Px(8.0)),
        Point::new(Px(18.0), Px(16.0)),
        Point::new(Px(0.0), Px(16.0)),
    ];

    list.path(|path| {
        path.line_to(points[0])
            .line_to(points[1])
            .line_to(points[2])
            .line_to(points[3])
            .line_to(points[4]);
        path.fill_concave(Color::from_srgb_hex_rgb(0xff_ff_ff));
        assert!(path.is_empty());

        path.line_to(points[0]).line_to(points[1]);
        path.fill_concave(Color::from_srgb_hex_rgb(0xff_ff_ff));
        assert!(path.is_empty());
    });

    assert_eq!(list.command_count(), 1);
    let DebugDrawCommand::ConcavePolyFilled {
        points: recorded, ..
    } = &list.commands[0]
    else {
        panic!("path concave fill should record a dedicated command");
    };
    assert_eq!(&**recorded, &points);
}

#[test]
fn debug_draw_path_builder_appends_rect_points() {
    let mut list = ImUiDebugDrawList::default();
    let rect = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(20.0), Px(10.0)),
    );

    list.path(|path| {
        path.rect(rect);
        assert_eq!(path.point_count(), 4);
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), true);
    });

    let DebugDrawCommand::Polyline { points, closed, .. } = &list.commands[0] else {
        panic!("path rect helper should record a closed polyline command");
    };
    assert!(*closed);
    assert_eq!(points.len(), 4);
    assert_eq!(
        &**points,
        &[
            Point::new(Px(10.0), Px(20.0)),
            Point::new(Px(30.0), Px(20.0)),
            Point::new(Px(30.0), Px(30.0)),
            Point::new(Px(10.0), Px(30.0)),
        ]
    );
}

#[test]
fn debug_draw_path_builder_appends_rounded_rect_corner_samples() {
    let mut list = ImUiDebugDrawList::default();
    let rect = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(20.0), Px(10.0)),
    );

    list.path(|path| {
        path.rect_with_rounding(
            rect,
            Px(4.0),
            DebugDrawRoundCorners::TOP_LEFT | DebugDrawRoundCorners::BOTTOM_RIGHT,
        );
        assert_eq!(path.point_count(), 10);
        path.fill_convex(Color::from_srgb_hex_rgb(0xff_ff_ff));
    });

    let DebugDrawCommand::ConvexPolyFilled { points, .. } = &list.commands[0] else {
        panic!("rounded path rect helper should record sampled convex fill points");
    };
    assert_eq!(points.len(), 10);
    assert_point_near(points[0], Point::new(Px(10.0), Px(24.0)));
    assert_point_near(points[3], Point::new(Px(14.0), Px(20.0)));
    assert_point_near(points[4], Point::new(Px(30.0), Px(20.0)));
    assert_point_near(points[5], Point::new(Px(30.0), Px(26.0)));
    assert_point_near(points[8], Point::new(Px(26.0), Px(30.0)));
    assert_point_near(points[9], Point::new(Px(10.0), Px(30.0)));
}

#[test]
fn debug_draw_path_builder_rect_rounding_clamps_and_handles_invalid_inputs() {
    let mut list = ImUiDebugDrawList::default();
    let rect = Rect::new(Point::new(Px(10.0), Px(20.0)), Size::new(Px(12.0), Px(8.0)));

    list.path(|path| {
        path.rect_with_rounding(rect, Px(50.0), DebugDrawRoundCorners::ALL);
        assert_eq!(path.point_count(), 16);
        assert_point_near(path.points[0], Point::new(Px(10.0), Px(23.0)));
        path.clear();

        path.rect_with_rounding(rect, Px(4.0), DebugDrawRoundCorners::NONE);
        assert_eq!(path.point_count(), 4);
        assert_eq!(path.points[0], Point::new(Px(10.0), Px(20.0)));
        assert_eq!(path.points[2], Point::new(Px(22.0), Px(28.0)));
        path.clear();

        path.rect_with_rounding(
            Rect::new(Point::new(Px(10.0), Px(20.0)), Size::new(Px(0.0), Px(8.0))),
            Px(4.0),
            DebugDrawRoundCorners::ALL,
        );
        path.rect_with_rounding(rect, Px(f32::NAN), DebugDrawRoundCorners::ALL);
        assert!(path.is_empty());
    });

    assert_eq!(list.command_count(), 0);
}

#[test]
fn debug_draw_path_builder_appends_bezier_curve_samples() {
    let mut list = ImUiDebugDrawList::default();
    let start = Point::new(Px(0.0), Px(0.0));
    let quad_mid = Point::new(Px(10.0), Px(5.0));
    let quad_end = Point::new(Px(20.0), Px(0.0));
    let cubic_mid = Point::new(Px(30.0), Px(5.0));
    let cubic_end = Point::new(Px(40.0), Px(10.0));

    list.path(|path| {
        path.line_to(start)
            .bezier_quadratic_curve_to(Point::new(Px(10.0), Px(10.0)), quad_end, 2)
            .bezier_cubic_curve_to(
                Point::new(Px(30.0), Px(0.0)),
                Point::new(Px(30.0), Px(10.0)),
                cubic_end,
                2,
            );
        assert_eq!(path.point_count(), 5);
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
    });

    let DebugDrawCommand::Polyline { points, .. } = &list.commands[0] else {
        panic!("path Bezier helpers should record a sampled polyline command");
    };
    assert_eq!(
        &**points,
        &[start, quad_mid, quad_end, cubic_mid, cubic_end]
    );
}

#[test]
fn debug_draw_path_builder_bezier_helpers_require_a_start_point_and_default_segments() {
    let mut list = ImUiDebugDrawList::default();
    let start = Point::new(Px(0.0), Px(0.0));
    let ctrl = Point::new(Px(10.0), Px(10.0));
    let end = Point::new(Px(20.0), Px(0.0));

    list.path(|path| {
        path.bezier_quadratic_curve_to(ctrl, end, 2);
        assert!(path.is_empty());

        path.line_to(start).bezier_quadratic_curve_to(ctrl, end, 0);
        assert_eq!(path.point_count(), DEFAULT_PATH_BEZIER_SEGMENTS + 1);
        path.clear();

        path.bezier_cubic_curve_to(ctrl, ctrl, end, 2);
        assert!(path.is_empty());
    });

    assert_eq!(list.command_count(), 0);
}

#[test]
fn debug_draw_path_builder_appends_arc_samples() {
    let mut list = ImUiDebugDrawList::default();
    let center = Point::new(Px(10.0), Px(20.0));

    list.path(|path| {
        path.arc_to(center, Px(8.0), 0.0, std::f32::consts::PI, 2);
        assert_eq!(path.point_count(), 3);
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
    });

    let DebugDrawCommand::Polyline { points, .. } = &list.commands[0] else {
        panic!("path arc helper should record a sampled polyline command");
    };
    assert_eq!(points.len(), 3);
    assert_point_near(points[0], Point::new(Px(18.0), Px(20.0)));
    assert_point_near(points[1], Point::new(Px(10.0), Px(28.0)));
    assert_point_near(points[2], Point::new(Px(2.0), Px(20.0)));
}

#[test]
fn debug_draw_path_builder_arc_helpers_handle_fast_default_and_degenerate_inputs() {
    let mut list = ImUiDebugDrawList::default();
    let center = Point::new(Px(10.0), Px(20.0));

    list.path(|path| {
        path.arc_to(center, Px(0.25), 0.0, std::f32::consts::PI, 4);
        assert_eq!(path.point_count(), 1);
        assert_eq!(path.clear().point_count(), 0);

        path.arc_to(center, Px(8.0), f32::NAN, std::f32::consts::PI, 4);
        path.arc_to(center, Px(0.0), 0.0, std::f32::consts::PI, 4);
        assert!(path.is_empty());

        path.arc_to(center, Px(8.0), 0.0, std::f32::consts::FRAC_PI_2, 0);
        assert_eq!(path.point_count(), DEFAULT_PATH_ARC_SEGMENTS + 1);
        path.clear();

        path.arc_to_fast(center, Px(8.0), 0, 3);
        assert_eq!(path.point_count(), 4);
        path.clear();

        path.arc_to_fast(center, Px(8.0), 3, 0);
        assert_eq!(path.point_count(), 4);
        path.clear();
    });

    assert_eq!(list.command_count(), 0);
}

#[test]
fn debug_draw_path_builder_appends_elliptical_arc_samples() {
    let mut list = ImUiDebugDrawList::default();
    let center = Point::new(Px(10.0), Px(20.0));

    list.path(|path| {
        path.elliptical_arc_to(
            center,
            Size::new(Px(8.0), Px(4.0)),
            0.0,
            0.0,
            std::f32::consts::PI,
            2,
        );
        assert_eq!(path.point_count(), 3);
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
    });

    let DebugDrawCommand::Polyline { points, .. } = &list.commands[0] else {
        panic!("path elliptical arc helper should record a sampled polyline command");
    };
    assert_eq!(points.len(), 3);
    assert_point_near(points[0], Point::new(Px(18.0), Px(20.0)));
    assert_point_near(points[1], Point::new(Px(10.0), Px(24.0)));
    assert_point_near(points[2], Point::new(Px(2.0), Px(20.0)));
}

#[test]
fn debug_draw_path_builder_elliptical_arc_handles_rotation_default_and_invalid_inputs() {
    let mut list = ImUiDebugDrawList::default();
    let center = Point::new(Px(10.0), Px(20.0));

    list.path(|path| {
        path.elliptical_arc_to(
            center,
            Size::new(Px(8.0), Px(4.0)),
            std::f32::consts::FRAC_PI_2,
            0.0,
            std::f32::consts::PI,
            2,
        );
        assert_eq!(path.point_count(), 3);
        assert_point_near(path.points[0], Point::new(Px(10.0), Px(28.0)));
        assert_point_near(path.points[1], Point::new(Px(6.0), Px(20.0)));
        assert_point_near(path.points[2], Point::new(Px(10.0), Px(12.0)));
        path.clear();

        path.elliptical_arc_to(
            center,
            Size::new(Px(8.0), Px(4.0)),
            0.0,
            0.0,
            std::f32::consts::FRAC_PI_2,
            0,
        );
        assert_eq!(path.point_count(), DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS + 1);
        path.clear();

        path.elliptical_arc_to(
            center,
            Size::new(Px(0.0), Px(4.0)),
            0.0,
            0.0,
            std::f32::consts::PI,
            2,
        );
        path.elliptical_arc_to(
            center,
            Size::new(Px(8.0), Px(4.0)),
            f32::NAN,
            0.0,
            std::f32::consts::PI,
            2,
        );
        path.elliptical_arc_to(
            center,
            Size::new(Px(8.0), Px(4.0)),
            0.0,
            f32::NAN,
            std::f32::consts::PI,
            2,
        );
        assert!(path.is_empty());
    });

    assert_eq!(list.command_count(), 0);
}

#[test]
fn debug_draw_path_builder_clears_invalid_finished_paths_without_recording() {
    let mut list = ImUiDebugDrawList::default();
    let p0 = Point::new(Px(0.0), Px(0.0));
    let p1 = Point::new(Px(8.0), Px(0.0));

    list.path(|path| {
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
        assert!(path.is_empty());

        path.line_to(p0);
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
        assert!(path.is_empty());

        path.line_to(p0).line_to(p1);
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), true);
        assert!(path.is_empty());

        path.line_to(p0).line_to(p1);
        path.fill_convex(Color::from_srgb_hex_rgb(0xff_ff_ff));
        assert!(path.is_empty());

        path.line_to(p0).line_to(p1);
        assert_eq!(path.point_count(), 2);
        path.clear();
        assert!(path.is_empty());
    });

    assert_eq!(list.command_count(), 0);
}

#[test]
fn image_overlay_helpers_sanitize_opacity_and_uv_rects() {
    assert_eq!(normalized_opacity(-1.0), 0.0);
    assert_eq!(normalized_opacity(2.0), 1.0);
    assert_eq!(normalized_opacity(f32::NAN), 1.0);

    assert!(uv_rect_is_valid(UvRect::FULL));
    assert!(!uv_rect_is_valid(UvRect {
        u0: 0.5,
        v0: 0.0,
        u1: 0.25,
        v1: 1.0,
    }));
}

#[test]
fn rounded_image_helpers_follow_imgui_path_rect_corner_rules() {
    let rect = Rect::new(Point::new(Px(10.0), Px(20.0)), Size::new(Px(12.0), Px(8.0)));

    let all = rounded_rect_corner_radii(rect, Px(50.0), DebugDrawRoundCorners::ALL);
    assert_eq!(all, Corners::all(Px(3.0)));
    assert!(corner_radii_are_visible(all));

    let diagonal = rounded_rect_corner_radii(
        rect,
        Px(50.0),
        DebugDrawRoundCorners::TOP_LEFT | DebugDrawRoundCorners::BOTTOM_RIGHT,
    );
    assert_eq!(diagonal.top_left, Px(7.0));
    assert_eq!(diagonal.top_right, Px(0.0));
    assert_eq!(diagonal.bottom_right, Px(7.0));
    assert_eq!(diagonal.bottom_left, Px(0.0));

    assert_eq!(
        rounded_rect_corner_radii(rect, Px(4.0), DebugDrawRoundCorners::NONE),
        Corners::all(Px(0.0))
    );
    assert_eq!(
        rounded_rect_corner_radii(rect, Px(f32::NAN), DebugDrawRoundCorners::ALL),
        Corners::all(Px(0.0))
    );
}

#[test]
fn rect_path_closes_clockwise_edges() {
    let path = rect_path(Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(30.0), Px(40.0)),
    ));

    assert_eq!(
        path,
        [
            PathCommand::MoveTo(Point::new(Px(10.0), Px(20.0))),
            PathCommand::LineTo(Point::new(Px(40.0), Px(20.0))),
            PathCommand::LineTo(Point::new(Px(40.0), Px(60.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(60.0))),
            PathCommand::Close,
        ]
    );
}

#[test]
fn debug_draw_stroke_style_uses_v1_for_default_and_v2_for_explicit_policy() {
    let default_style = DebugDrawStrokeStyle::new(Px(2.0));
    assert_eq!(
        default_style.path_style(),
        PathStyle::Stroke(StrokeStyle { width: Px(2.0) })
    );

    let styled = DebugDrawStrokeStyle::new(Px(3.0))
        .with_join(StrokeJoinV1::Round)
        .with_cap(StrokeCapV1::Round)
        .with_miter_limit(8.0)
        .with_dash(Px(6.0), Px(4.0), Px(1.0));

    let PathStyle::StrokeV2(stroke) = styled.path_style() else {
        panic!("explicit debug-draw stroke policy should use StrokeV2");
    };
    assert_eq!(stroke.width, Px(3.0));
    assert_eq!(stroke.join, StrokeJoinV1::Round);
    assert_eq!(stroke.cap, StrokeCapV1::Round);
    assert_eq!(stroke.miter_limit, 8.0);
    assert_eq!(
        stroke.dash,
        Some(DashPatternV1::new(Px(6.0), Px(4.0), Px(1.0)))
    );
}

#[test]
fn debug_draw_stroke_style_ignores_invalid_dash_and_miter_inputs() {
    let style = DebugDrawStrokeStyle::new(Px(2.0))
        .with_miter_limit(f32::NAN)
        .with_dash(Px(0.0), Px(4.0), Px(0.0))
        .with_dash_pattern(DashPatternV1::new(Px(4.0), Px(-1.0), Px(0.0)));

    assert_eq!(style.miter_limit, 4.0);
    assert_eq!(style.dash, None);
    assert_eq!(
        style.path_style(),
        PathStyle::Stroke(StrokeStyle { width: Px(2.0) })
    );
}

#[test]
fn polyline_path_requires_enough_points_and_closes_when_requested() {
    assert!(polyline_path(&[Point::new(Px(0.0), Px(0.0))], false).is_none());
    assert!(
        polyline_path(
            &[Point::new(Px(0.0), Px(0.0)), Point::new(Px(1.0), Px(1.0))],
            true,
        )
        .is_none()
    );

    let path = polyline_path(
        &[
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(10.0), Px(0.0)),
            Point::new(Px(10.0), Px(10.0)),
        ],
        true,
    )
    .unwrap();

    assert_eq!(
        path,
        vec![
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(10.0))),
            PathCommand::Close,
        ]
    );
}

#[test]
fn convex_poly_fill_path_requires_three_points_and_closes() {
    assert!(convex_poly_fill_path(&[Point::new(Px(0.0), Px(0.0))]).is_none());
    assert!(
        convex_poly_fill_path(&[Point::new(Px(0.0), Px(0.0)), Point::new(Px(10.0), Px(0.0)),])
            .is_none()
    );

    let path = convex_poly_fill_path(&[
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(10.0), Px(0.0)),
        Point::new(Px(12.0), Px(8.0)),
        Point::new(Px(2.0), Px(10.0)),
    ])
    .unwrap();

    assert_eq!(
        path,
        vec![
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(12.0), Px(8.0))),
            PathCommand::LineTo(Point::new(Px(2.0), Px(10.0))),
            PathCommand::Close,
        ]
    );
}

#[test]
fn concave_poly_fill_path_requires_three_points_and_closes() {
    assert!(concave_poly_fill_path(&[Point::new(Px(0.0), Px(0.0))]).is_none());
    assert!(
        concave_poly_fill_path(&[Point::new(Px(0.0), Px(0.0)), Point::new(Px(10.0), Px(0.0)),])
            .is_none()
    );

    let path = concave_poly_fill_path(&[
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(18.0), Px(0.0)),
        Point::new(Px(10.0), Px(8.0)),
        Point::new(Px(18.0), Px(16.0)),
        Point::new(Px(0.0), Px(16.0)),
    ])
    .unwrap();

    assert_eq!(
        path,
        vec![
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(18.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(8.0))),
            PathCommand::LineTo(Point::new(Px(18.0), Px(16.0))),
            PathCommand::LineTo(Point::new(Px(0.0), Px(16.0))),
            PathCommand::Close,
        ]
    );
}

#[test]
fn triangle_path_closes_and_degenerate_triangles_are_detected() {
    let p1 = Point::new(Px(0.0), Px(0.0));
    let p2 = Point::new(Px(10.0), Px(0.0));
    let p3 = Point::new(Px(5.0), Px(8.0));

    assert_eq!(
        triangle_path(p1, p2, p3),
        [
            PathCommand::MoveTo(p1),
            PathCommand::LineTo(p2),
            PathCommand::LineTo(p3),
            PathCommand::Close,
        ]
    );
    assert!(!triangle_is_degenerate(p1, p2, p3));
    assert!(triangle_is_degenerate(p1, Point::new(Px(5.0), Px(0.0)), p2));
}

#[test]
fn quad_path_closes_four_ordered_points() {
    let p1 = Point::new(Px(0.0), Px(0.0));
    let p2 = Point::new(Px(10.0), Px(2.0));
    let p3 = Point::new(Px(12.0), Px(12.0));
    let p4 = Point::new(Px(2.0), Px(10.0));

    assert_eq!(
        quad_path(p1, p2, p3, p4),
        [
            PathCommand::MoveTo(p1),
            PathCommand::LineTo(p2),
            PathCommand::LineTo(p3),
            PathCommand::LineTo(p4),
            PathCommand::Close,
        ]
    );
}

#[test]
fn circle_path_uses_four_cubic_arcs_and_closes() {
    let path = circle_path(Point::new(Px(10.0), Px(20.0)), Px(8.0));

    assert_eq!(path.len(), 6);
    assert_eq!(path[0], PathCommand::MoveTo(Point::new(Px(18.0), Px(20.0))));
    assert!(matches!(path[1], PathCommand::CubicTo { .. }));
    assert!(matches!(path[2], PathCommand::CubicTo { .. }));
    assert!(matches!(path[3], PathCommand::CubicTo { .. }));
    assert!(matches!(path[4], PathCommand::CubicTo { .. }));
    assert_eq!(path[5], PathCommand::Close);
}

#[test]
fn ngon_path_requires_three_segments_and_positive_radius() {
    assert!(ngon_path(Point::new(Px(0.0), Px(0.0)), Px(8.0), 2).is_none());
    assert!(ngon_path(Point::new(Px(0.0), Px(0.0)), Px(0.0), 3).is_none());

    let path = ngon_path(Point::new(Px(10.0), Px(20.0)), Px(8.0), 4).unwrap();

    assert_eq!(path.len(), 5);
    assert_eq!(path[0], PathCommand::MoveTo(Point::new(Px(18.0), Px(20.0))));
    assert!(matches!(path[1], PathCommand::LineTo(_)));
    assert!(matches!(path[2], PathCommand::LineTo(_)));
    assert!(matches!(path[3], PathCommand::LineTo(_)));
    assert_eq!(path[4], PathCommand::Close);
}

#[test]
fn ellipse_path_defaults_segments_and_supports_rotation() {
    assert!(
        ellipse_path(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(8.0), Px(4.0)),
            0.0,
            2
        )
        .is_none()
    );
    assert!(
        ellipse_path(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(0.0), Px(4.0)),
            0.0,
            4
        )
        .is_none()
    );
    assert!(
        ellipse_path(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(8.0), Px(4.0)),
            f32::NAN,
            4,
        )
        .is_none()
    );

    let default_path = ellipse_path(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(8.0), Px(4.0)),
        0.0,
        0,
    )
    .unwrap();
    assert_eq!(default_path.len(), DEFAULT_ELLIPSE_SEGMENTS + 1);
    assert_eq!(
        default_path[0],
        PathCommand::MoveTo(Point::new(Px(18.0), Px(20.0)))
    );
    assert_eq!(default_path[DEFAULT_ELLIPSE_SEGMENTS], PathCommand::Close);

    let rotated_path = ellipse_path(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(8.0), Px(4.0)),
        std::f32::consts::FRAC_PI_2,
        4,
    )
    .unwrap();
    let PathCommand::MoveTo(point) = &rotated_path[0] else {
        panic!("rotated ellipse should start with a MoveTo");
    };
    assert!((point.x.0 - 10.0).abs() <= 0.000_1);
    assert!((point.y.0 - 28.0).abs() <= 0.000_1);
    assert_eq!(rotated_path[4], PathCommand::Close);
}

#[test]
fn bezier_paths_use_native_quad_and_cubic_commands() {
    let from = Point::new(Px(0.0), Px(0.0));
    let ctrl = Point::new(Px(10.0), Px(20.0));
    let ctrl1 = Point::new(Px(8.0), Px(16.0));
    let ctrl2 = Point::new(Px(18.0), Px(-6.0));
    let to = Point::new(Px(24.0), Px(0.0));

    assert_eq!(
        bezier_quadratic_path(from, ctrl, to),
        [PathCommand::MoveTo(from), PathCommand::QuadTo { ctrl, to }]
    );
    assert_eq!(
        bezier_cubic_path(from, ctrl1, ctrl2, to),
        [
            PathCommand::MoveTo(from),
            PathCommand::CubicTo { ctrl1, ctrl2, to },
        ]
    );
}
