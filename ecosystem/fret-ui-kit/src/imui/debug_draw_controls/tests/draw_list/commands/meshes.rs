use super::*;

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
