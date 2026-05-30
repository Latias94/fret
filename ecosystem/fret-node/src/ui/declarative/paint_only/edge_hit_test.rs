use super::cache::edge_commands_for_route;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct EdgeHit {
    pub(super) edge: crate::core::EdgeId,
    pub(super) distance2_canvas: f32,
}

pub(super) fn effective_edge_hit_width_screen_px(
    interaction: &crate::io::NodeGraphInteractionState,
    style: &NodeGraphStyle,
    width_mul: f32,
) -> f32 {
    interaction
        .edge_interaction_width
        .max(style.paint.wire_interaction_width)
        .max(style.geometry.wire_width * width_mul.max(1.0))
        .max(0.0)
}

fn is_selectable_edge(
    edge: &crate::core::Edge,
    interaction: &crate::io::NodeGraphInteractionState,
) -> bool {
    interaction.elements_selectable && edge.selectable.unwrap_or(interaction.edges_selectable)
}

fn dist2_point_to_segment(p: Point, a: Point, b: Point) -> f32 {
    let apx = p.x.0 - a.x.0;
    let apy = p.y.0 - a.y.0;
    let abx = b.x.0 - a.x.0;
    let aby = b.y.0 - a.y.0;

    let ab2 = abx * abx + aby * aby;
    if !ab2.is_finite() || ab2 <= 1.0e-9 {
        return apx * apx + apy * apy;
    }

    let t = ((apx * abx + apy * aby) / ab2).clamp(0.0, 1.0);
    let cx = a.x.0 + t * abx;
    let cy = a.y.0 + t * aby;
    let dx = p.x.0 - cx;
    let dy = p.y.0 - cy;
    dx * dx + dy * dy
}

fn quad_bezier(p0: Point, p1: Point, p2: Point, t: f32) -> Point {
    let t = t.clamp(0.0, 1.0);
    let mt = 1.0 - t;
    Point::new(
        Px(mt * mt * p0.x.0 + 2.0 * mt * t * p1.x.0 + t * t * p2.x.0),
        Px(mt * mt * p0.y.0 + 2.0 * mt * t * p1.y.0 + t * t * p2.y.0),
    )
}

fn add_segment_distance(best: &mut f32, point: Point, from: Point, to: Point) {
    let d2 = dist2_point_to_segment(point, from, to);
    if d2.is_finite() {
        *best = best.min(d2);
    }
}

fn path_command_distance2(commands: &[PathCommand], point: Point, steps: usize) -> f32 {
    let steps = steps.max(1);
    let mut best = f32::INFINITY;
    let mut current = None::<Point>;
    let mut subpath_start = None::<Point>;

    for command in commands {
        match *command {
            PathCommand::MoveTo(to) => {
                current = Some(to);
                subpath_start = Some(to);
            }
            PathCommand::LineTo(to) => {
                if let Some(from) = current {
                    add_segment_distance(&mut best, point, from, to);
                }
                current = Some(to);
            }
            PathCommand::QuadTo { ctrl, to } => {
                if let Some(from) = current {
                    let mut prev = from;
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let next = quad_bezier(from, ctrl, to, t);
                        add_segment_distance(&mut best, point, prev, next);
                        prev = next;
                    }
                }
                current = Some(to);
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                if let Some(from) = current {
                    let mut prev = from;
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let next = canvas_wires::cubic_bezier(from, ctrl1, ctrl2, to, t);
                        add_segment_distance(&mut best, point, prev, next);
                        prev = next;
                    }
                }
                current = Some(to);
            }
            PathCommand::Close => {
                if let (Some(from), Some(to)) = (current, subpath_start) {
                    add_segment_distance(&mut best, point, from, to);
                    current = Some(to);
                }
            }
        }
    }

    best
}

fn edge_hit_path_commands(
    graph: &Graph,
    edge_id: crate::core::EdgeId,
    geom: &CanvasGeometry,
    zoom: f32,
    style: &NodeGraphStyle,
    edge_types: Option<&crate::ui::NodeGraphEdgeTypes>,
) -> Option<(crate::ui::presenter::EdgeRenderHint, Box<[PathCommand]>)> {
    let edge = graph.edges.get(&edge_id)?;
    let from = geom.port_center(edge.from)?;
    let to = geom.port_center(edge.to)?;
    let presenter = DefaultNodeGraphPresenter::default();
    let mut hint = presenter
        .edge_render_hint(graph, edge_id, style)
        .normalized();
    if let Some(edge_types) = edge_types {
        hint = edge_types.apply(graph, edge_id, style, hint).normalized();
    }
    let custom_path = edge_types.and_then(|edge_types| {
        edge_types.custom_path(
            graph,
            edge_id,
            style,
            &hint,
            EdgePathInput { from, to, zoom },
        )
    });
    let commands = custom_path
        .map(|path| path.commands.into_boxed_slice())
        .unwrap_or_else(|| edge_commands_for_route(hint.route, from, to, zoom));
    Some((hint, commands))
}

pub(super) fn hit_test_edge_at_canvas_point(
    graph: &Graph,
    zoom: f32,
    geom: &CanvasGeometry,
    index: &CanvasSpatialDerived,
    interaction: &crate::io::NodeGraphInteractionState,
    style: &NodeGraphStyle,
    edge_types: Option<&crate::ui::NodeGraphEdgeTypes>,
    point_canvas: Point,
    scratch: &mut Vec<crate::core::EdgeId>,
) -> Option<EdgeHit> {
    if !interaction.elements_selectable {
        return None;
    }

    let zoom = PanZoom2D::sanitize_zoom(zoom, 1.0).max(1.0e-6);
    let query_radius = effective_edge_hit_width_screen_px(interaction, style, 1.0) / zoom;
    let candidates = index.query_edges_sorted_dedup(point_canvas, query_radius, scratch);
    let steps = usize::from(interaction.bezier_hit_test_steps).max(1);
    let mut best = None::<EdgeHit>;

    for edge_id in candidates.iter().copied() {
        let Some(edge) = graph.edges.get(&edge_id) else {
            continue;
        };
        if !is_selectable_edge(edge, interaction) {
            continue;
        }
        let Some((hint, commands)) =
            edge_hit_path_commands(graph, edge_id, geom, zoom, style, edge_types)
        else {
            continue;
        };
        let hit_width = effective_edge_hit_width_screen_px(interaction, style, hint.width_mul);
        let radius = (hit_width / zoom).max(0.0);
        let distance2 = path_command_distance2(&commands, point_canvas, steps);
        if distance2 > radius * radius {
            continue;
        }
        let hit = EdgeHit {
            edge: edge_id,
            distance2_canvas: distance2,
        };
        match best {
            None => best = Some(hit),
            Some(current) if hit.distance2_canvas < current.distance2_canvas => best = Some(hit),
            _ => {}
        }
    }

    best
}
