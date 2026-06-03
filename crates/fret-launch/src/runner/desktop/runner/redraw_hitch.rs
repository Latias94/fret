use std::sync::{Mutex, OnceLock};

use fret_core::Rect;
use fret_core::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub(super) struct RedrawHitchConfig {
    pub(super) hitch_ms: u64,
}

pub(super) fn redraw_hitch_config() -> Option<RedrawHitchConfig> {
    static CONFIG: OnceLock<Option<RedrawHitchConfig>> = OnceLock::new();
    *CONFIG.get_or_init(|| {
        let enabled = std::env::var_os("FRET_REDRAW_HITCH_LOG").is_some_and(|v| !v.is_empty());
        if !enabled {
            return None;
        }

        let hitch_ms = std::env::var("FRET_REDRAW_HITCH_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);

        Some(RedrawHitchConfig { hitch_ms })
    })
}

fn redraw_hitch_log_paths() -> impl Iterator<Item = std::path::PathBuf> {
    let mut paths = Vec::new();

    if let Some(custom) = std::env::var_os("FRET_REDRAW_HITCH_LOG_PATH")
        && !custom.is_empty()
    {
        let mut path = std::path::PathBuf::from(custom);
        if path.is_relative()
            && let Some(diag_dir) = std::env::var_os("FRET_DIAG_DIR")
            && !diag_dir.is_empty()
        {
            path = std::path::Path::new(&diag_dir).join(path);
        }
        paths.push(path);
    }

    paths.push(std::path::Path::new(".fret").join("redraw_hitches.log"));

    let tmp = std::env::temp_dir();
    if !tmp.as_os_str().is_empty() {
        paths.push(tmp.join("fret").join("redraw_hitches.log"));
    }
    paths.into_iter()
}

pub(super) fn quantize_logical_px(value: f32) -> f32 {
    if !value.is_finite() || value <= 0.0 {
        return 0.0;
    }
    let quantum = 64.0f32;
    (value * quantum).round() / quantum
}

struct HitchLogWriter {
    file: std::io::BufWriter<std::fs::File>,
}

struct HitchLogState {
    writers: Vec<HitchLogWriter>,
    writes_since_flush: u32,
    last_flush: Instant,
}

impl HitchLogState {
    fn new() -> Self {
        let mut writers = Vec::new();
        for path in redraw_hitch_log_paths() {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                writers.push(HitchLogWriter {
                    file: std::io::BufWriter::with_capacity(16 * 1024, file),
                });
            }
        }

        Self {
            writers,
            writes_since_flush: 0,
            last_flush: Instant::now(),
        }
    }

    fn write_line(&mut self, msg: &str) {
        use std::io::Write as _;

        let mut i = 0;
        while i < self.writers.len() {
            let ok = self.writers[i].file.write_all(msg.as_bytes()).is_ok();
            if ok {
                i += 1;
            } else {
                self.writers.swap_remove(i);
            }
        }

        self.writes_since_flush = self.writes_since_flush.saturating_add(1);
        let should_flush =
            self.writes_since_flush >= 64 || self.last_flush.elapsed().as_millis() >= 250;
        if should_flush {
            for w in self.writers.iter_mut() {
                let _ = w.file.flush();
            }
            self.writes_since_flush = 0;
            self.last_flush = Instant::now();
        }
    }
}

pub(super) fn write_redraw_hitch_log(line: &str) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let thread_id = format!("{:?}", std::thread::current().id());
    let msg = format!("[{ts}] [thread={thread_id}] {line}\n");

    static STATE: OnceLock<Mutex<HitchLogState>> = OnceLock::new();
    let state = STATE.get_or_init(|| Mutex::new(HitchLogState::new()));
    let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
    state.write_line(&msg);
}

#[derive(Debug, Clone, Copy)]
pub(super) enum RedrawPhase {
    Prepare,
    Render { bounds: Rect, scale_factor: f32 },
    Record { scene_ops: usize },
    Present,
    RenderScene,
}

impl RedrawPhase {
    fn make_span(self) -> tracing::Span {
        match self {
            Self::Prepare => tracing::info_span!("fret.runner.prepare"),
            Self::Render {
                bounds,
                scale_factor,
            } => tracing::info_span!(
                "fret.runner.render",
                bounds = ?bounds,
                scale_factor = scale_factor,
            ),
            Self::Record { scene_ops } => {
                tracing::info_span!("fret.runner.record", scene_ops = scene_ops,)
            }
            Self::Present => tracing::info_span!("fret.runner.present"),
            Self::RenderScene => tracing::info_span!("fret.runner.render_scene"),
        }
    }
}

pub(super) fn measure_redraw_phase<T>(
    phase: RedrawPhase,
    time_enabled: bool,
    f: impl FnOnce() -> T,
) -> (T, Option<Duration>) {
    fret_perf::measure_span(
        time_enabled,
        tracing::enabled!(tracing::Level::INFO),
        || phase.make_span(),
        f,
    )
}
