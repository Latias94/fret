use fret_core::time::{Duration, Instant};
use std::env;

use fret_canvas::spatial::DefaultIndexWithBackrefs;
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
    rect_min: f32,
    rect_max: f32,
    n: usize,
    q: usize,
    radius: f32,
    frames: usize,
    moving: usize,
    cell: f32,
    viewport_w: f32,
    viewport_h: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            seed: 1,
            scenario: Scenario::Uniform,
            world: 10_000.0,
            rect_min: 24.0,
            rect_max: 220.0,
            n: 10_000,
            q: 4_000,
            radius: 14.0,
            frames: 180,
            moving: 256,
            cell: 128.0,
            viewport_w: 1200.0,
            viewport_h: 800.0,
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
        // PCG-like LCG constants.
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
            "--rect-min" => cfg.rect_min = value.parse().unwrap_or(cfg.rect_min),
            "--rect-max" => cfg.rect_max = value.parse().unwrap_or(cfg.rect_max),
            "--n" => cfg.n = value.parse().unwrap_or(cfg.n),
            "--q" => cfg.q = value.parse().unwrap_or(cfg.q),
            "--radius" => cfg.radius = value.parse().unwrap_or(cfg.radius),
            "--frames" => cfg.frames = value.parse().unwrap_or(cfg.frames),
            "--moving" => cfg.moving = value.parse().unwrap_or(cfg.moving),
            "--cell" => cfg.cell = value.parse().unwrap_or(cfg.cell),
            "--viewport-w" => cfg.viewport_w = value.parse().unwrap_or(cfg.viewport_w),
            "--viewport-h" => cfg.viewport_h = value.parse().unwrap_or(cfg.viewport_h),
            _ => {}
        }
    }
    cfg
}

fn rect_from_center(cx: f32, cy: f32, w: f32, h: f32) -> Rect {
    Rect::new(
        Point::new(Px(cx - 0.5 * w), Px(cy - 0.5 * h)),
        Size::new(Px(w), Px(h)),
    )
}

fn gen_rects(cfg: Config) -> Vec<Rect> {
    let mut rng = Rng::new(cfg.seed);
    let mut rects = Vec::with_capacity(cfg.n);

    let cluster_centers: Vec<(f32, f32)> = match cfg.scenario {
        Scenario::Uniform => Vec::new(),
        Scenario::Cluster => (0..8)
            .map(|_| (rng.range_f32(0.0, cfg.world), rng.range_f32(0.0, cfg.world)))
            .collect(),
    };

    for _ in 0..cfg.n {
        let (cx, cy) = match cfg.scenario {
            Scenario::Uniform => (rng.range_f32(0.0, cfg.world), rng.range_f32(0.0, cfg.world)),
            Scenario::Cluster => {
                let (kx, ky) = cluster_centers[rng.choose_usize(cluster_centers.len())];
                let jitter = cfg.world * 0.03;
                (
                    (kx + rng.range_f32(-jitter, jitter)).clamp(0.0, cfg.world),
                    (ky + rng.range_f32(-jitter, jitter)).clamp(0.0, cfg.world),
                )
            }
        };
        let w = rng.range_f32(cfg.rect_min, cfg.rect_max);
        let h = rng.range_f32(cfg.rect_min, cfg.rect_max);
        rects.push(rect_from_center(cx, cy, w, h));
    }
    rects
}

fn fmt_dur(d: Duration) -> String {
    let us = d.as_secs_f64() * 1_000_000.0;
    if us < 1_000.0 {
        format!("{us:.1}µs")
    } else if us < 1_000_000.0 {
        format!("{:.2}ms", us / 1_000.0)
    } else {
        format!("{:.2}s", us / 1_000_000.0)
    }
}

fn main() {
    let cfg = parse_args();
    let rects = gen_rects(cfg);

    let mut index = DefaultIndexWithBackrefs::<u32>::new(cfg.cell);
    let backend = index.backend_name();

    let t0 = Instant::now();
    for (i, rect) in rects.iter().enumerate() {
        index.insert_rect(i as u32, *rect);
    }
    let build = t0.elapsed();

    let mut rng = Rng::new(cfg.seed ^ 0x9e3779b97f4a7c15);
    let mut out = Vec::new();

    let mut sum = 0usize;
    let mut max = 0usize;

    let t1 = Instant::now();
    for _ in 0..cfg.q {
        let p = Point::new(
            Px(rng.range_f32(0.0, cfg.world)),
            Px(rng.range_f32(0.0, cfg.world)),
        );
        index.query_radius(p, cfg.radius, &mut out);
        sum += out.len();
        max = max.max(out.len());
    }
    let query_radius = t1.elapsed();

    let mut sum_rect = 0usize;
    let mut max_rect = 0usize;
    let t2 = Instant::now();
    for _ in 0..(cfg.q / 2).max(1) {
        let x = rng.range_f32(0.0, cfg.world);
        let y = rng.range_f32(0.0, cfg.world);
        let rect = Rect::new(
            Point::new(Px(x), Px(y)),
            Size::new(Px(cfg.viewport_w), Px(cfg.viewport_h)),
        );
        index.query_rect(rect, &mut out);
        sum_rect += out.len();
        max_rect = max_rect.max(out.len());
    }
    let query_rect = t2.elapsed();

    let moving = cfg.moving.min(rects.len());
    let mut moving_ids: Vec<u32> = (0..moving as u32).collect();
    // Shuffle ids deterministically.
    for i in 0..moving_ids.len() {
        let j = rng.choose_usize(moving_ids.len());
        moving_ids.swap(i, j);
    }

    let mut rects = rects;
    let delta = cfg.world * 0.0008;
    let t3 = Instant::now();
    for frame in 0..cfg.frames {
        let dx = (frame as f32 * 0.73).sin() * delta;
        let dy = (frame as f32 * 0.41).cos() * delta;
        for &id in &moving_ids {
            let idx = id as usize;
            let r = rects[idx];
            let moved = Rect::new(
                Point::new(Px(r.origin.x.0 + dx), Px(r.origin.y.0 + dy)),
                r.size,
            );
            rects[idx] = moved;
            index.update_rect(id, moved);
        }
    }
    let update = t3.elapsed();

    let q = cfg.q.max(1) as f64;
    let qr = (cfg.q / 2).max(1) as f64;
    println!("backend={backend}");
    println!("build: {} (n={})", fmt_dur(build), cfg.n);
    println!(
        "query_radius: {} (q={}, radius={:.1}) avg_candidates={:.2} max_candidates={}",
        fmt_dur(query_radius),
        cfg.q,
        cfg.radius,
        (sum as f64) / q,
        max
    );
    println!(
        "query_rect: {} (q={}, viewport={:.0}x{:.0}) avg_candidates={:.2} max_candidates={}",
        fmt_dur(query_rect),
        (cfg.q / 2).max(1),
        cfg.viewport_w,
        cfg.viewport_h,
        (sum_rect as f64) / qr,
        max_rect
    );
    println!(
        "update_rect: {} (frames={}, moving={})",
        fmt_dur(update),
        cfg.frames,
        moving
    );
    println!(
        "config: scenario={:?} world={:.0} rect=[{:.0},{:.0}] cell={:.0}",
        cfg.scenario, cfg.world, cfg.rect_min, cfg.rect_max, cfg.cell
    );
}
