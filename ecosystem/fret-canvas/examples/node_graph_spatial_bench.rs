use fret_core::time::{Duration, Instant};
use std::env;

use fret_canvas::spatial::DefaultIndexWithBackrefs;
use fret_canvas::wires::{
    DEFAULT_BEZIER_FLATTEN_TOLERANCE_SCREEN_PX, DEFAULT_BEZIER_HIT_TEST_STEPS,
    bezier_wire_distance2, bezier_wire_distance2_polyline_adaptive, wire_aabb,
};

#[cfg(feature = "kurbo")]
use fret_canvas::wires::bezier_wire_distance2_kurbo;
#[cfg(feature = "kurbo")]
use fret_canvas::wires::bezier_wire_distance2_polyline;
use fret_core::{Point, Px, Rect, Size};

#[derive(Clone, Copy, Debug)]
enum Scenario {
    Uniform,
    Cluster,
}

impl Scenario {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "uniform" => Some(Self::Uniform),
            "cluster" => Some(Self::Cluster),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Config {
    seed: u64,
    scenario: Scenario,
    world: f32,
    zoom: f32,

    nodes: usize,
    edges: usize,
    ports_per_side: usize,
    edge_near_k: usize,

    node_w_min: f32,
    node_w_max: f32,
    node_h_min: f32,
    node_h_max: f32,

    port_size: f32,
    edge_pad: f32,

    frames: usize,
    moving_nodes: usize,
    move_delta: f32,

    // Query workload (per frame)
    port_queries: usize,
    edge_queries: usize,
    rect_queries: usize,

    port_radius: f32,
    edge_radius: f32,
    edge_hit_width: f32,

    viewport_w: f32,
    viewport_h: f32,

    cell: f32,

    compare_kurbo: bool,
    adaptive_polyline: bool,
    polyline_tol_screen_px: f32,
    compare_band_screen_px: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            seed: 1,
            scenario: Scenario::Uniform,
            world: 20_000.0,
            zoom: 1.0,

            nodes: 5_000,
            edges: 12_000,
            ports_per_side: 2,
            edge_near_k: 8,

            node_w_min: 160.0,
            node_w_max: 320.0,
            node_h_min: 80.0,
            node_h_max: 180.0,

            port_size: 12.0,
            edge_pad: 8.0,

            frames: 240,
            moving_nodes: 128,
            move_delta: 14.0,

            port_queries: 400,
            edge_queries: 400,
            rect_queries: 40,

            port_radius: 18.0,
            edge_radius: 24.0,
            edge_hit_width: 6.0,

            viewport_w: 1200.0,
            viewport_h: 800.0,

            cell: 128.0,

            compare_kurbo: false,
            adaptive_polyline: false,
            polyline_tol_screen_px: DEFAULT_BEZIER_FLATTEN_TOLERANCE_SCREEN_PX,
            compare_band_screen_px: 2.0,
        }
    }
}

#[derive(Clone, Debug)]
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        let v = self.next_u32();
        (v as f32) / (u32::MAX as f32)
    }

    fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        if !min.is_finite() || !max.is_finite() || min >= max {
            return min;
        }
        min + (max - min) * self.next_f32()
    }

    fn choose_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.next_u32() as usize) % max
    }
}

fn parse_args() -> Config {
    let mut cfg = Config::default();
    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        let Some(value) = it.next() else { break };
        match arg.as_str() {
            "--seed" => cfg.seed = value.parse().unwrap_or(cfg.seed),
            "--scenario" => cfg.scenario = Scenario::parse(&value).unwrap_or(cfg.scenario),
            "--world" => cfg.world = value.parse().unwrap_or(cfg.world),
            "--zoom" => cfg.zoom = value.parse().unwrap_or(cfg.zoom),
            "--nodes" => cfg.nodes = value.parse().unwrap_or(cfg.nodes),
            "--edges" => cfg.edges = value.parse().unwrap_or(cfg.edges),
            "--ports-per-side" => cfg.ports_per_side = value.parse().unwrap_or(cfg.ports_per_side),
            "--edge-near-k" => cfg.edge_near_k = value.parse().unwrap_or(cfg.edge_near_k),
            "--frames" => cfg.frames = value.parse().unwrap_or(cfg.frames),
            "--moving-nodes" => cfg.moving_nodes = value.parse().unwrap_or(cfg.moving_nodes),
            "--move-delta" => cfg.move_delta = value.parse().unwrap_or(cfg.move_delta),
            "--port-queries" => cfg.port_queries = value.parse().unwrap_or(cfg.port_queries),
            "--edge-queries" => cfg.edge_queries = value.parse().unwrap_or(cfg.edge_queries),
            "--rect-queries" => cfg.rect_queries = value.parse().unwrap_or(cfg.rect_queries),
            "--edge-hit-width" => cfg.edge_hit_width = value.parse().unwrap_or(cfg.edge_hit_width),
            "--cell" => cfg.cell = value.parse().unwrap_or(cfg.cell),
            "--compare-kurbo" => {
                cfg.compare_kurbo = match value.as_str() {
                    "1" | "true" | "True" | "TRUE" | "yes" | "on" => true,
                    "0" | "false" | "False" | "FALSE" | "no" | "off" => false,
                    _ => cfg.compare_kurbo,
                }
            }
            "--compare-band-screen" => {
                cfg.compare_band_screen_px = value
                    .parse()
                    .ok()
                    .filter(|v: &f32| v.is_finite() && *v > 0.0)
                    .unwrap_or(cfg.compare_band_screen_px);
            }
            "--adaptive-polyline" => {
                cfg.adaptive_polyline = match value.as_str() {
                    "1" | "true" | "True" | "TRUE" | "yes" | "on" => true,
                    "0" | "false" | "False" | "FALSE" | "no" | "off" => false,
                    _ => cfg.adaptive_polyline,
                }
            }
            "--polyline-tol-screen" => {
                cfg.polyline_tol_screen_px = value
                    .parse()
                    .ok()
                    .filter(|v: &f32| v.is_finite() && *v > 0.0)
                    .unwrap_or(cfg.polyline_tol_screen_px);
            }
            _ => {}
        }
    }
    cfg
}

fn fmt_dur(d: Duration) -> String {
    let us = d.as_secs_f64() * 1_000_000.0;
    if us < 1_000.0 {
        format!("{us:.1}us")
    } else if us < 1_000_000.0 {
        format!("{:.2}ms", us / 1_000.0)
    } else {
        format!("{:.2}s", us / 1_000_000.0)
    }
}

#[derive(Clone, Copy, Debug)]
struct Node {
    rect: Rect,
}

#[derive(Clone, Copy, Debug)]
struct Port {
    rect: Rect,
    center: Point,
}

#[derive(Clone, Copy, Debug)]
struct Edge {
    from: u32,
    to: u32,
}

#[derive(Clone, Copy, Debug)]
struct PortRef {
    node: u32,
    side: u8,
    idx: u16,
}

fn port_id(node: u32, side: u8, idx: u16) -> u32 {
    (node << 8) | ((side as u32) << 7) | (idx as u32 & 0x7f)
}

fn decode_port_id(id: u32) -> PortRef {
    PortRef {
        node: id >> 8,
        side: ((id >> 7) & 1) as u8,
        idx: (id & 0x7f) as u16,
    }
}

fn gen_nodes(cfg: Config, rng: &mut Rng) -> Vec<Node> {
    let clusters: Vec<(f32, f32)> = match cfg.scenario {
        Scenario::Uniform => Vec::new(),
        Scenario::Cluster => (0..8)
            .map(|_| (rng.range_f32(0.0, cfg.world), rng.range_f32(0.0, cfg.world)))
            .collect(),
    };

    let mut nodes = Vec::with_capacity(cfg.nodes);
    for _ in 0..cfg.nodes {
        let (cx, cy) = match cfg.scenario {
            Scenario::Uniform => (rng.range_f32(0.0, cfg.world), rng.range_f32(0.0, cfg.world)),
            Scenario::Cluster => {
                let (kx, ky) = clusters[rng.choose_usize(clusters.len())];
                let jitter = cfg.world * 0.03;
                (
                    (kx + rng.range_f32(-jitter, jitter)).clamp(0.0, cfg.world),
                    (ky + rng.range_f32(-jitter, jitter)).clamp(0.0, cfg.world),
                )
            }
        };
        let w = rng.range_f32(cfg.node_w_min, cfg.node_w_max);
        let h = rng.range_f32(cfg.node_h_min, cfg.node_h_max);
        let rect = Rect::new(
            Point::new(Px(cx - 0.5 * w), Px(cy - 0.5 * h)),
            Size::new(Px(w), Px(h)),
        );
        nodes.push(Node { rect });
    }
    nodes
}

fn node_centers(nodes: &[Node]) -> Vec<(f32, f32)> {
    nodes
        .iter()
        .map(|n| {
            (
                n.rect.origin.x.0 + 0.5 * n.rect.size.width.0,
                n.rect.origin.y.0 + 0.5 * n.rect.size.height.0,
            )
        })
        .collect()
}

fn compute_ports(cfg: Config, nodes: &[Node]) -> Vec<Port> {
    let ports_per_node = cfg.ports_per_side * 2;
    let mut ports = Vec::with_capacity(nodes.len() * ports_per_node);

    for node in nodes {
        let x0 = node.rect.origin.x.0;
        let y0 = node.rect.origin.y.0;
        let w = node.rect.size.width.0;
        let h = node.rect.size.height.0;

        let pad = 12.0;
        let span = (h - 2.0 * pad).max(1.0);
        for i in 0..cfg.ports_per_side {
            let t = (i as f32 + 1.0) / (cfg.ports_per_side as f32 + 1.0);
            let cy = y0 + pad + span * t;
            for side in 0..=1u8 {
                let cx = if side == 0 { x0 } else { x0 + w };
                let pr = Rect::new(
                    Point::new(Px(cx - 0.5 * cfg.port_size), Px(cy - 0.5 * cfg.port_size)),
                    Size::new(Px(cfg.port_size), Px(cfg.port_size)),
                );
                ports.push(Port {
                    rect: pr,
                    center: Point::new(Px(cx), Px(cy)),
                });
            }
        }
    }

    ports
}

fn port_vec_index(cfg: Config, port: PortRef) -> usize {
    // Layout from `compute_ports`: for each node: for each idx: side in [0,1] push.
    let per_node = cfg.ports_per_side * 2;
    let idx = port.idx as usize;
    (port.node as usize) * per_node + idx * 2 + (port.side as usize)
}

fn gen_edges(cfg: Config, rng: &mut Rng, centers: &[(f32, f32)]) -> (Vec<Edge>, Vec<Vec<u32>>) {
    let ports_per_node = cfg.ports_per_side * 2;
    let total_ports = cfg.nodes * ports_per_node;

    let mut edges = Vec::with_capacity(cfg.edges);
    let mut edges_by_port: Vec<Vec<u32>> = vec![Vec::new(); total_ports];

    for edge_id in 0..cfg.edges {
        let from_node = rng.choose_usize(cfg.nodes) as u32;
        let from_center = centers[from_node as usize];

        // Prefer a nearby-ish target by sampling K candidates and picking the closest by center.
        let mut best = rng.choose_usize(cfg.nodes) as u32;
        let mut best_d2 = f32::INFINITY;
        let k = cfg.edge_near_k.max(1);
        for _ in 0..k {
            let cand = rng.choose_usize(cfg.nodes) as u32;
            let c = centers[cand as usize];
            let dx = c.0 - from_center.0;
            let dy = c.1 - from_center.1;
            let d2 = dx * dx + dy * dy;
            if d2 < best_d2 {
                best_d2 = d2;
                best = cand;
            }
        }
        let to_node = best;

        let from_idx = rng.choose_usize(cfg.ports_per_side) as u16;
        let to_idx = rng.choose_usize(cfg.ports_per_side) as u16;
        let from = port_id(from_node, 1, from_idx);
        let to = port_id(to_node, 0, to_idx);
        edges.push(Edge { from, to });

        let e = edge_id as u32;
        let from_vi = port_vec_index(cfg, decode_port_id(from));
        let to_vi = port_vec_index(cfg, decode_port_id(to));
        edges_by_port[from_vi].push(e);
        edges_by_port[to_vi].push(e);
    }

    (edges, edges_by_port)
}

fn recompute_port(cfg: Config, node: Rect, idx: usize, side: u8) -> Port {
    let x0 = node.origin.x.0;
    let y0 = node.origin.y.0;
    let w = node.size.width.0;
    let h = node.size.height.0;

    let pad = 12.0;
    let span = (h - 2.0 * pad).max(1.0);
    let t = (idx as f32 + 1.0) / (cfg.ports_per_side as f32 + 1.0);
    let cy = y0 + pad + span * t;
    let cx = if side == 0 { x0 } else { x0 + w };

    let pr = Rect::new(
        Point::new(Px(cx - 0.5 * cfg.port_size), Px(cy - 0.5 * cfg.port_size)),
        Size::new(Px(cfg.port_size), Px(cfg.port_size)),
    );

    Port {
        rect: pr,
        center: Point::new(Px(cx), Px(cy)),
    }
}

fn main() {
    let cfg = parse_args();
    let mut rng = Rng::new(cfg.seed);
    let zoom = if cfg.zoom.is_finite() && cfg.zoom > 0.0 {
        cfg.zoom
    } else {
        1.0
    };

    let mut nodes = gen_nodes(cfg, &mut rng);
    let centers = node_centers(&nodes);
    let mut ports = compute_ports(cfg, &nodes);
    let (edges, edges_by_port) = gen_edges(cfg, &mut rng, &centers);

    let mut node_index = DefaultIndexWithBackrefs::<u32>::new(cfg.cell);
    let mut port_index = DefaultIndexWithBackrefs::<u32>::new(cfg.cell);
    let mut edge_index = DefaultIndexWithBackrefs::<u32>::new(cfg.cell);
    let backend = node_index.backend_name();

    let t0 = Instant::now();
    for (i, node) in nodes.iter().enumerate() {
        node_index.insert_rect(i as u32, node.rect);
    }
    for (i, port) in ports.iter().enumerate() {
        port_index.insert_rect(i as u32, port.rect);
    }
    for (i, edge) in edges.iter().enumerate() {
        let from = ports[port_vec_index(cfg, decode_port_id(edge.from))].center;
        let to = ports[port_vec_index(cfg, decode_port_id(edge.to))].center;
        edge_index.insert_rect(i as u32, wire_aabb(from, to, zoom, cfg.edge_pad));
    }
    let build = t0.elapsed();

    let mut moving: Vec<u32> = (0..cfg.moving_nodes.min(cfg.nodes) as u32).collect();
    for i in 0..moving.len() {
        let j = rng.choose_usize(moving.len());
        moving.swap(i, j);
    }

    let mut out_ports: Vec<u32> = Vec::new();
    let mut out_edges: Vec<u32> = Vec::new();
    let mut out_nodes: Vec<u32> = Vec::new();
    let mut edge_update_scratch: Vec<u32> = Vec::new();

    let mut query_candidates_ports = 0usize;
    let mut query_candidates_edges = 0usize;
    let mut query_candidates_nodes = 0usize;
    let mut max_ports = 0usize;
    let mut max_edges = 0usize;
    let mut max_nodes = 0usize;

    let mut refine_evals = 0usize;
    let mut refine_hits = 0usize;

    let mut t_update = Duration::default();
    let mut t_query = Duration::default();
    let mut t_refine = Duration::default();

    #[cfg(feature = "kurbo")]
    let mut t_refine_polyline = Duration::default();
    #[cfg(feature = "kurbo")]
    let mut t_refine_kurbo = Duration::default();

    #[cfg(feature = "kurbo")]
    let mut compare_total = 0usize;
    #[cfg(feature = "kurbo")]
    let mut compare_disagree = 0usize;
    #[cfg(feature = "kurbo")]
    let mut compare_false_pos = 0usize;
    #[cfg(feature = "kurbo")]
    let mut compare_false_neg = 0usize;
    #[cfg(feature = "kurbo")]
    let mut compare_hits_polyline = 0usize;
    #[cfg(feature = "kurbo")]
    let mut compare_hits_kurbo = 0usize;
    #[cfg(feature = "kurbo")]
    let mut compare_abs_d_err_sum = 0f64;
    #[cfg(feature = "kurbo")]
    let mut compare_abs_d_err_max = 0f32;

    #[cfg(feature = "kurbo")]
    let mut compare_boundary_total = 0usize;
    #[cfg(feature = "kurbo")]
    let mut compare_boundary_disagree = 0usize;
    #[cfg(feature = "kurbo")]
    let mut compare_boundary_false_pos = 0usize;
    #[cfg(feature = "kurbo")]
    let mut compare_boundary_false_neg = 0usize;
    #[cfg(feature = "kurbo")]
    let mut compare_boundary_abs_err_sum = 0f64;
    #[cfg(feature = "kurbo")]
    let mut compare_boundary_abs_err_max = 0f32;

    #[cfg(feature = "kurbo")]
    let mut poly_d2_scratch: Vec<f32> = Vec::new();

    let mut rng_q = Rng::new(cfg.seed ^ 0x9e3779b97f4a7c15);
    let steps = DEFAULT_BEZIER_HIT_TEST_STEPS;
    let hit_w2 = cfg.edge_hit_width * cfg.edge_hit_width;
    #[cfg(feature = "kurbo")]
    let hit_w_screen = cfg.edge_hit_width * zoom;
    let compare_band_screen_px = cfg.compare_band_screen_px;

    #[cfg(feature = "kurbo")]
    let compare_kurbo = cfg.compare_kurbo;
    #[cfg(not(feature = "kurbo"))]
    let compare_kurbo = false;
    let polyline_adaptive = cfg.adaptive_polyline;
    let polyline_tol_screen_px = cfg.polyline_tol_screen_px;

    for frame in 0..cfg.frames {
        let dx = (frame as f32 * 0.73).sin() * cfg.move_delta;
        let dy = (frame as f32 * 0.41).cos() * cfg.move_delta;

        let t1 = Instant::now();
        for &node_id in &moving {
            let idx = node_id as usize;
            let r = nodes[idx].rect;
            let moved = Rect::new(
                Point::new(Px(r.origin.x.0 + dx), Px(r.origin.y.0 + dy)),
                r.size,
            );
            nodes[idx].rect = moved;
            node_index.update_rect(node_id, moved);

            edge_update_scratch.clear();

            for port_idx in 0..cfg.ports_per_side {
                for side in 0..=1u8 {
                    let pid = port_id(node_id, side, port_idx as u16);
                    let pr = decode_port_id(pid);
                    let vi = port_vec_index(cfg, pr);

                    let port = recompute_port(cfg, moved, port_idx, side);
                    ports[vi] = port;
                    port_index.update_rect(vi as u32, port.rect);

                    edge_update_scratch.extend_from_slice(&edges_by_port[vi]);
                }
            }

            // Update adjacent edges once per moved node.
            edge_update_scratch.sort_unstable();
            edge_update_scratch.dedup();
            for &edge_id in &edge_update_scratch {
                let e = edges[edge_id as usize];
                let from = ports[port_vec_index(cfg, decode_port_id(e.from))].center;
                let to = ports[port_vec_index(cfg, decode_port_id(e.to))].center;
                edge_index.update_rect(edge_id, wire_aabb(from, to, zoom, cfg.edge_pad));
            }
        }
        t_update += t1.elapsed();

        let t2 = Instant::now();
        for _ in 0..cfg.port_queries {
            let p = Point::new(
                Px(rng_q.range_f32(0.0, cfg.world)),
                Px(rng_q.range_f32(0.0, cfg.world)),
            );
            port_index.query_radius(p, cfg.port_radius, &mut out_ports);
            out_ports.sort_unstable();
            out_ports.dedup();
            query_candidates_ports += out_ports.len();
            max_ports = max_ports.max(out_ports.len());
        }

        for _ in 0..cfg.edge_queries {
            let p = Point::new(
                Px(rng_q.range_f32(0.0, cfg.world)),
                Px(rng_q.range_f32(0.0, cfg.world)),
            );
            edge_index.query_radius(p, cfg.edge_radius, &mut out_edges);
            out_edges.sort_unstable();
            out_edges.dedup();
            query_candidates_edges += out_edges.len();
            max_edges = max_edges.max(out_edges.len());

            if compare_kurbo {
                #[cfg(feature = "kurbo")]
                {
                    poly_d2_scratch.clear();
                    poly_d2_scratch.reserve(out_edges.len());

                    let tr = Instant::now();
                    for &edge_id in &out_edges {
                        let e = edges[edge_id as usize];
                        let from = ports[port_vec_index(cfg, decode_port_id(e.from))].center;
                        let to = ports[port_vec_index(cfg, decode_port_id(e.to))].center;
                        if polyline_adaptive {
                            poly_d2_scratch.push(bezier_wire_distance2_polyline_adaptive(
                                p,
                                from,
                                to,
                                zoom,
                                polyline_tol_screen_px,
                            ));
                        } else {
                            poly_d2_scratch
                                .push(bezier_wire_distance2_polyline(p, from, to, zoom, steps));
                        }
                    }
                    t_refine_polyline += tr.elapsed();

                    let tr = Instant::now();
                    for (i, &edge_id) in out_edges.iter().enumerate() {
                        let e = edges[edge_id as usize];
                        let from = ports[port_vec_index(cfg, decode_port_id(e.from))].center;
                        let to = ports[port_vec_index(cfg, decode_port_id(e.to))].center;
                        let d2_k = bezier_wire_distance2_kurbo(p, from, to, zoom, steps);
                        let d2_p = poly_d2_scratch[i];

                        let hit_p = d2_p <= hit_w2;
                        let hit_k = d2_k <= hit_w2;

                        compare_total += 1;
                        if hit_p {
                            compare_hits_polyline += 1;
                        }
                        if hit_k {
                            compare_hits_kurbo += 1;
                        }
                        if hit_p != hit_k {
                            compare_disagree += 1;
                            if hit_k && !hit_p {
                                compare_false_pos += 1;
                            } else if hit_p && !hit_k {
                                compare_false_neg += 1;
                            }
                        }

                        let d_p = d2_p.sqrt();
                        let d_k = d2_k.sqrt();
                        if d_p.is_finite() && d_k.is_finite() {
                            let d_p_screen = d_p * zoom;
                            let d_k_screen = d_k * zoom;
                            let abs_screen = (d_p_screen - d_k_screen).abs();
                            compare_abs_d_err_sum += abs_screen as f64;
                            compare_abs_d_err_max = compare_abs_d_err_max.max(abs_screen);

                            let boundary = ((d_p_screen - hit_w_screen).abs())
                                .min((d_k_screen - hit_w_screen).abs())
                                <= compare_band_screen_px;
                            if boundary {
                                compare_boundary_total += 1;
                                compare_boundary_abs_err_sum += abs_screen as f64;
                                compare_boundary_abs_err_max =
                                    compare_boundary_abs_err_max.max(abs_screen);

                                if hit_p != hit_k {
                                    compare_boundary_disagree += 1;
                                    if hit_k && !hit_p {
                                        compare_boundary_false_pos += 1;
                                    } else if hit_p && !hit_k {
                                        compare_boundary_false_neg += 1;
                                    }
                                }
                            }
                        }
                    }
                    t_refine_kurbo += tr.elapsed();
                }
            } else {
                let tr = Instant::now();
                for &edge_id in &out_edges {
                    let e = edges[edge_id as usize];
                    let from = ports[port_vec_index(cfg, decode_port_id(e.from))].center;
                    let to = ports[port_vec_index(cfg, decode_port_id(e.to))].center;
                    let d2 = if polyline_adaptive {
                        bezier_wire_distance2_polyline_adaptive(
                            p,
                            from,
                            to,
                            zoom,
                            polyline_tol_screen_px,
                        )
                    } else {
                        bezier_wire_distance2(p, from, to, zoom, steps)
                    };
                    refine_evals += 1;
                    if d2 <= hit_w2 {
                        refine_hits += 1;
                    }
                }
                t_refine += tr.elapsed();
            }
        }

        for _ in 0..cfg.rect_queries {
            let x = rng_q.range_f32(0.0, cfg.world);
            let y = rng_q.range_f32(0.0, cfg.world);
            let rect = Rect::new(
                Point::new(Px(x), Px(y)),
                Size::new(Px(cfg.viewport_w), Px(cfg.viewport_h)),
            );
            node_index.query_rect(rect, &mut out_nodes);
            out_nodes.sort_unstable();
            out_nodes.dedup();
            query_candidates_nodes += out_nodes.len();
            max_nodes = max_nodes.max(out_nodes.len());
        }
        t_query += t2.elapsed();
    }

    let frames = cfg.frames.max(1) as f64;
    let pq = (cfg.frames * cfg.port_queries).max(1) as f64;
    let eq = (cfg.frames * cfg.edge_queries).max(1) as f64;
    let rq = (cfg.frames * cfg.rect_queries).max(1) as f64;
    let re = refine_evals.max(1) as f64;

    println!("backend={backend}");
    println!(
        "build: {} (nodes={}, edges={}, ports_per_side={}, edge_near_k={})",
        fmt_dur(build),
        cfg.nodes,
        cfg.edges,
        cfg.ports_per_side,
        cfg.edge_near_k
    );
    println!(
        "update: {} (frames={}, moving_nodes={})",
        fmt_dur(t_update),
        cfg.frames,
        moving.len()
    );
    println!("query: {} (frames={})", fmt_dur(t_query), cfg.frames);
    println!(
        "  ports: avg_candidates={:.2} max_candidates={} (total_queries={})",
        (query_candidates_ports as f64) / pq,
        max_ports,
        (cfg.frames * cfg.port_queries)
    );
    println!(
        "  edges: avg_candidates={:.2} max_candidates={} (total_queries={})",
        (query_candidates_edges as f64) / eq,
        max_edges,
        (cfg.frames * cfg.edge_queries)
    );
    println!(
        "  nodes(rect): avg_candidates={:.2} max_candidates={} (total_queries={})",
        (query_candidates_nodes as f64) / rq,
        max_nodes,
        (cfg.frames * cfg.rect_queries)
    );
    if !compare_kurbo {
        println!(
            "refine(bezier): {} avg_evals_per_edge_query={:.2} hit_rate={:.2}%",
            fmt_dur(t_refine),
            (refine_evals as f64) / eq,
            100.0 * (refine_hits as f64) / re
        );
    }
    #[cfg(feature = "kurbo")]
    if compare_kurbo {
        let ct = compare_total.max(1) as f64;
        println!(
            "refine(polyline:{}): {} avg_evals_per_edge_query={:.2} hit_rate={:.2}%",
            if polyline_adaptive {
                "adaptive"
            } else {
                "fixed"
            },
            fmt_dur(t_refine_polyline),
            ct / eq,
            100.0 * (compare_hits_polyline as f64) / ct
        );
        println!(
            "refine(kurbo): {} avg_evals_per_edge_query={:.2} hit_rate={:.2}%",
            fmt_dur(t_refine_kurbo),
            ct / eq,
            100.0 * (compare_hits_kurbo as f64) / ct
        );
        println!(
            "compare(kurbo): disagree={:.2}% fp={} fn={} avg_abs_err={:.3} max_abs_err={:.3}",
            100.0 * (compare_disagree as f64) / ct,
            compare_false_pos,
            compare_false_neg,
            compare_abs_d_err_sum / ct,
            compare_abs_d_err_max
        );
        let bt = compare_boundary_total.max(1) as f64;
        println!(
            "compare(kurbo,boundary): band_screen_px={:.2} samples={:.2}% disagree={:.2}% fp={} fn={} avg_abs_err={:.3} max_abs_err={:.3}",
            compare_band_screen_px,
            100.0 * (compare_boundary_total as f64) / ct,
            100.0 * (compare_boundary_disagree as f64) / bt,
            compare_boundary_false_pos,
            compare_boundary_false_neg,
            compare_boundary_abs_err_sum / bt,
            compare_boundary_abs_err_max
        );
    }
    println!(
        "config: scenario={:?} world={:.0} zoom={:.2} cell={:.0} viewport={:.0}x{:.0}",
        cfg.scenario, cfg.world, zoom, cfg.cell, cfg.viewport_w, cfg.viewport_h
    );
    println!(
        "config(polyline): mode={} tol_screen_px={:.2}",
        if polyline_adaptive {
            "adaptive"
        } else {
            "fixed"
        },
        polyline_tol_screen_px
    );
    if compare_kurbo {
        println!(
            "config(compare): band_screen_px={:.2}",
            compare_band_screen_px
        );
    }
    println!(
        "timing_per_frame: update={} query={} refine={}",
        fmt_dur(Duration::from_secs_f64(t_update.as_secs_f64() / frames)),
        fmt_dur(Duration::from_secs_f64(t_query.as_secs_f64() / frames)),
        fmt_dur(Duration::from_secs_f64(t_refine.as_secs_f64() / frames))
    );
}
