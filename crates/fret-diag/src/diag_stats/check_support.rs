#[derive(Debug, Clone, Copy)]
pub(crate) enum StatsLiteCheckKind {
    /// Works directly from `frames.index.json` (may require aggregates/features).
    FramesIndex,
    /// Works via streaming reads from the bundle artifact (does not materialize the whole JSON).
    StreamingBundle,
    /// Works on the (potentially stats-lite) `BundleStatsReport` only.
    ReportOnly,
    /// Works from out-dir artifacts, not the bundle JSON.
    OutDirOnly,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StatsLiteCheckSupport {
    pub(crate) check_name: &'static str,
    pub(crate) kind: StatsLiteCheckKind,
    pub(crate) note: &'static str,
}

pub(crate) const STATS_LITE_SUPPORTED_CHECKS: &[StatsLiteCheckSupport] = &[
    StatsLiteCheckSupport {
        check_name: "check-hover-layout-max",
        kind: StatsLiteCheckKind::ReportOnly,
        note: "derived from the stats report",
    },
    StatsLiteCheckSupport {
        check_name: "check-pixels-changed",
        kind: StatsLiteCheckKind::OutDirOnly,
        note: "uses out-dir artifacts",
    },
    StatsLiteCheckSupport {
        check_name: "check-view-cache-reuse-min",
        kind: StatsLiteCheckKind::FramesIndex,
        note: "uses frames.index.json window aggregates",
    },
    StatsLiteCheckSupport {
        check_name: "check-viewport-input-min",
        kind: StatsLiteCheckKind::FramesIndex,
        note: "uses frames.index.json window aggregates",
    },
    StatsLiteCheckSupport {
        check_name: "check-dock-drag-min",
        kind: StatsLiteCheckKind::FramesIndex,
        note: "uses frames.index.json window aggregates",
    },
    StatsLiteCheckSupport {
        check_name: "check-viewport-capture-min",
        kind: StatsLiteCheckKind::FramesIndex,
        note: "uses frames.index.json window aggregates",
    },
    StatsLiteCheckSupport {
        check_name: "check-overlay-synthesis-min",
        kind: StatsLiteCheckKind::FramesIndex,
        note: "uses frames.index.json window aggregates",
    },
    StatsLiteCheckSupport {
        check_name: "check-wheel-scroll",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; resolves semantics via schema2 tables when needed",
    },
    StatsLiteCheckSupport {
        check_name: "check-wheel-scroll-hit-changes",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; resolves semantics via schema2 tables when needed",
    },
    StatsLiteCheckSupport {
        check_name: "check-notify-hotspot-file-max",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.notify_requests only",
    },
];
