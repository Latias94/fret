use fret_core::{NodeId as UiNodeId, PathCommand, PathConstraints, PathId, PathMetrics, PathStyle};
use fret_core::{Point, Px, Rect, Scene, SceneOp, Size, TextBlobId, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::ui::NodeGraphCanvas;
use crate::ui::edge_types::{EdgeCustomPath, EdgePathInput, NodeGraphEdgeTypes};
use crate::ui::presenter::EdgeRenderHint;
use crate::ui::presenter::{EdgeMarker, EdgeRouteKind, NodeGraphPresenter};
use crate::ui::style::NodeGraphStyle;

use super::{TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_ports};

use std::sync::Arc;

#[derive(Default)]
struct CaptureServices {
    path_prepares: Vec<(Vec<PathCommand>, PathStyle, PathConstraints)>,
    text_prepares: Vec<(String, fret_core::TextConstraints)>,
}

impl fret_core::TextService for CaptureServices {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        constraints: fret_core::TextConstraints,
    ) -> (TextBlobId, fret_core::TextMetrics) {
        self.text_prepares
            .push((input.text().to_string(), constraints));
        let text = input.text();
        (
            TextBlobId::default(),
            fret_core::TextMetrics {
                size: Size::new(Px(text.len() as f32 * 7.0), Px(14.0)),
                baseline: Px(11.0),
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

fn paint_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    tree: &mut UiTree<TestUiHostImpl>,
    services: &mut CaptureServices,
    bounds: Rect,
) -> Scene {
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
    scene
}

fn extract_edge_label_ops(scene: &Scene, style: &NodeGraphStyle) -> Vec<(Rect, Point)> {
    let mut out = Vec::new();
    let ops = scene.ops();
    for ix in 0..ops.len().saturating_sub(1) {
        let SceneOp::Quad {
            order,
            rect,
            background,
            border_color,
            ..
        } = ops[ix]
        else {
            continue;
        };
        if order != fret_core::DrawOrder(2) {
            continue;
        }
        if background != style.edge_label_background || border_color != style.edge_label_border {
            continue;
        }
        let SceneOp::Text { origin, .. } = ops[ix + 1] else {
            continue;
        };
        out.push((rect, origin));
    }
    out
}

#[derive(Default)]
struct LabelOnlyPresenter {
    label: Arc<str>,
}

impl NodeGraphPresenter for LabelOnlyPresenter {
    fn node_title(&self, _graph: &crate::core::Graph, _node: crate::core::NodeId) -> Arc<str> {
        Arc::<str>::from("")
    }

    fn port_label(&self, _graph: &crate::core::Graph, _port: crate::core::PortId) -> Arc<str> {
        Arc::<str>::from("")
    }

    fn edge_render_hint(
        &self,
        _graph: &crate::core::Graph,
        _edge: EdgeId,
        _style: &NodeGraphStyle,
    ) -> EdgeRenderHint {
        EdgeRenderHint {
            label: Some(Arc::clone(&self.label)),
            route: EdgeRouteKind::Straight,
            width_mul: 1.0,
            ..EdgeRenderHint::default()
        }
    }
}

fn capture_edge_label_for_bounds(bounds: Rect) -> (Rect, Point, usize) {
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
        s.interaction.bezier_hit_test_steps = 8;
    });

    let label: Arc<str> = Arc::<str>::from("EdgeLabel");
    let presenter = LabelOnlyPresenter {
        label: Arc::clone(&label),
    };

    let dy = Px(80.0);
    let edge_types = NodeGraphEdgeTypes::new()
        .with_fallback(|_g, _e, _style, mut h| {
            h.start_marker = Some(EdgeMarker::arrow(12.0));
            h.end_marker = Some(EdgeMarker::arrow(12.0));
            h
        })
        .with_fallback_path(move |_g, _e, _style, _hint, input: EdgePathInput| {
            let from = Point::new(input.from.x, Px(input.from.y.0 + dy.0));
            let to = Point::new(input.to.x, Px(input.to.y.0 + dy.0));
            Some(EdgeCustomPath {
                cache_key: 1,
                commands: vec![PathCommand::MoveTo(from), PathCommand::LineTo(to)],
            })
        });

    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_presenter(presenter)
        .with_edge_types(edge_types);

    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);
    let from = geom.port_center(a_out).expect("from port center");
    let to = geom.port_center(b_in).expect("to port center");

    let base_mid = Point::new(Px(0.5 * (from.x.0 + to.x.0)), Px(0.5 * (from.y.0 + to.y.0)));
    let d = Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0));
    let normal = crate::ui::canvas::route_math::normal_from_tangent(d);
    let offset = canvas.style.edge_label_offset;
    let base_anchor = Point::new(
        Px(base_mid.x.0 + normal.x.0 * offset),
        Px(base_mid.y.0 + normal.y.0 * offset),
    );
    let expected_anchor = Point::new(base_anchor.x, Px(base_anchor.y.0 + dy.0));
    let expected_text_origin = Point::new(
        Px(expected_anchor.x.0 - 0.5 * (label.len() as f32 * 7.0)),
        Px(expected_anchor.y.0 - 0.5 * 14.0 + 11.0),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CaptureServices::default();
    for _ in 0..8 {
        let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

        let tile_build_done = canvas.edge_labels_build_states.is_empty();
        let single_build_done = canvas.edge_labels_build_state.is_none();
        if tile_build_done && single_build_done && canvas.edge_labels_scene_cache.entries_len() > 0
        {
            break;
        }
    }
    assert!(
        canvas.edge_labels_build_states.is_empty() && canvas.edge_labels_build_state.is_none(),
        "expected edge label cache warmup to complete within a few frames"
    );
    assert!(
        canvas.edge_labels_scene_cache.entries_len() > 0,
        "expected label cache to store at least one entry"
    );

    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let labels = extract_edge_label_ops(&scene, &canvas.style);
    assert!(
        !labels.is_empty(),
        "expected at least one label-like quad+text pair in the scene"
    );

    let mut best: Option<(Rect, Point, f32)> = None;
    for (rect, origin) in labels {
        let dx = (origin.x.0 - expected_text_origin.x.0).abs();
        let dy = (origin.y.0 - expected_text_origin.y.0).abs();
        let err = dx + dy;
        match best {
            None => best = Some((rect, origin, err)),
            Some((_, _, best_err)) if err < best_err => best = Some((rect, origin, err)),
            Some(_) => {}
        }
    }

    let (rect, origin, err) = best.expect("best match must exist");
    assert!(
        err <= 1.0e-3,
        "expected label to use custom-path offset anchor; err={err}"
    );

    (rect, origin, canvas.edge_labels_scene_cache.entries_len())
}

#[test]
fn cached_edge_labels_match_between_tiled_and_single_tile_cache_modes() {
    let small_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let large_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(3000.0), Px(3000.0)),
    );

    let (small_rect, small_origin, small_cache_entries) =
        capture_edge_label_for_bounds(small_bounds);
    let (large_rect, large_origin, large_cache_entries) =
        capture_edge_label_for_bounds(large_bounds);

    let rect_dx = (small_rect.origin.x.0 - large_rect.origin.x.0).abs()
        + (small_rect.origin.y.0 - large_rect.origin.y.0).abs()
        + (small_rect.size.width.0 - large_rect.size.width.0).abs()
        + (small_rect.size.height.0 - large_rect.size.height.0).abs();
    let origin_dx =
        (small_origin.x.0 - large_origin.x.0).abs() + (small_origin.y.0 - large_origin.y.0).abs();
    assert!(
        rect_dx <= 1.0e-3 && origin_dx <= 1.0e-3,
        "expected label rect/text origin to be identical regardless of cache tiling mode; rect_dx={rect_dx} origin_dx={origin_dx}"
    );

    assert!(
        small_cache_entries <= 2,
        "expected non-tiled label cache to store a single entry (got {small_cache_entries})"
    );
    assert!(
        large_cache_entries > 2,
        "expected tiled label cache to store multiple entries (got {large_cache_entries})"
    );
}
