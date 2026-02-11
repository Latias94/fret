use fret_core::{NodeId as UiNodeId, PathCommand, PathConstraints, PathId, PathMetrics, PathStyle};
use fret_core::{Point, Px, Rect, Scene, Size, TextBlobId, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::ui::NodeGraphCanvas;
use crate::ui::edge_types::{EdgeCustomPath, EdgePathInput, NodeGraphEdgeTypes};
use crate::ui::presenter::EdgeMarker;

use super::{TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_ports};

#[derive(Default)]
struct CaptureServices {
    path_prepares: Vec<(Vec<PathCommand>, PathStyle, PathConstraints)>,
}

impl fret_core::TextService for CaptureServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: fret_core::TextConstraints,
    ) -> (TextBlobId, fret_core::TextMetrics) {
        (
            TextBlobId::default(),
            fret_core::TextMetrics {
                size: Size::new(Px(0.0), Px(0.0)),
                baseline: Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl fret_core::PathService for CaptureServices {
    fn prepare(
        &mut self,
        commands: &[PathCommand],
        style: PathStyle,
        constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        self.path_prepares
            .push((commands.to_vec(), style, constraints));
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl fret_core::SvgService for CaptureServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        true
    }
}

impl fret_core::MaterialService for CaptureServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}

fn paint_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    tree: &mut UiTree<TestUiHostImpl>,
    services: &mut CaptureServices,
    bounds: Rect,
) {
    let mut scene = Scene::default();
    let mut observe_model = |_id, _inv: Invalidation| {};
    let mut observe_global = |_id, _inv: Invalidation| {};

    let mut cx = fret_ui::retained_bridge::PaintCx {
        app: host,
        tree,
        node: UiNodeId::default(),
        window: None,
        focus: None,
        children: &[],
        bounds,
        scale_factor: 1.0,
        accumulated_transform: Transform2D::IDENTITY,
        children_render_transform: None,
        services,
        observe_model: &mut observe_model,
        observe_global: &mut observe_global,
        scene: &mut scene,
    };

    canvas.paint(&mut cx);
}

fn arrow_tip_and_base(cmds: &[PathCommand]) -> Option<(Point, Point)> {
    if cmds.len() != 4 {
        return None;
    }
    let PathCommand::MoveTo(tip) = cmds[0] else {
        return None;
    };
    let PathCommand::LineTo(p1) = cmds[1] else {
        return None;
    };
    let PathCommand::LineTo(p2) = cmds[2] else {
        return None;
    };
    if cmds[3] != PathCommand::Close {
        return None;
    }

    let base = Point::new(Px(0.5 * (p1.x.0 + p2.x.0)), Px(0.5 * (p1.y.0 + p2.y.0)));
    Some((tip, base))
}

fn capture_edge_wire_and_markers_for_bounds(bounds: Rect) -> (Vec<Vec<PathCommand>>, usize) {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, b, b_in) = make_test_graph_two_nodes_with_ports();
    if let Some(node) = graph_value.nodes.get_mut(&b) {
        node.pos.y = 120.0;
    }

    let edge_id = EdgeId::new();
    graph_value.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
        s.interaction.only_render_visible_elements = true;
        s.interaction.frame_view_duration_ms = 0;
    });

    let edge_types = NodeGraphEdgeTypes::new()
        .with_fallback(|_g, _e, _style, mut h| {
            h.start_marker = Some(EdgeMarker::arrow(12.0));
            h.end_marker = Some(EdgeMarker::arrow(12.0));
            h
        })
        .with_fallback_path(|_g, _e, _style, _hint, input: EdgePathInput| {
            Some(EdgeCustomPath {
                cache_key: 1,
                commands: vec![
                    PathCommand::MoveTo(input.from),
                    PathCommand::LineTo(input.to),
                ],
            })
        });

    let mut canvas = NodeGraphCanvas::new(graph, view).with_edge_types(edge_types);
    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CaptureServices::default();

    for _ in 0..8 {
        paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
        if canvas.edges_build_states.is_empty() {
            break;
        }
    }
    assert!(
        canvas.edges_build_states.is_empty(),
        "expected edge cache warmup to complete within a few frames"
    );

    let wire_width_bits = Px(canvas.style.wire_width).0.to_bits();

    let mut out = Vec::new();
    for (cmds, style, _constraints) in &services.path_prepares {
        if let Some((_tip, _base)) = arrow_tip_and_base(cmds) {
            out.push(cmds.clone());
            continue;
        }

        let PathStyle::Stroke(stroke) = style else {
            continue;
        };
        if stroke.width.0.to_bits() != wire_width_bits {
            continue;
        }
        if cmds.len() == 2
            && matches!(cmds[0], PathCommand::MoveTo(_))
            && matches!(cmds[1], PathCommand::LineTo(_))
        {
            out.push(cmds.clone());
        }
    }

    out.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
    (out, canvas.edges_scene_cache.entries_len())
}

#[test]
fn cached_edge_paths_match_between_tiled_and_single_tile_cache_modes() {
    let small_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let large_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(3000.0), Px(3000.0)),
    );

    let (small_paths, small_cache_entries) = capture_edge_wire_and_markers_for_bounds(small_bounds);
    let (large_paths, large_cache_entries) = capture_edge_wire_and_markers_for_bounds(large_bounds);

    assert_eq!(
        small_paths, large_paths,
        "expected edge wire+marker path commands to be identical regardless of cache tiling mode"
    );
    assert!(
        small_cache_entries <= 2,
        "expected non-tiled edge cache to store a single entry (got {small_cache_entries})"
    );
    assert!(
        large_cache_entries > 2,
        "expected tiled edge cache to store multiple entries (got {large_cache_entries})"
    );
}
