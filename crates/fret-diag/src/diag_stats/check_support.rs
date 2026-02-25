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

pub(crate) fn stats_lite_support_for(check_name: &str) -> Option<StatsLiteCheckSupport> {
    STATS_LITE_SUPPORTED_CHECKS
        .iter()
        .copied()
        .find(|c| c.check_name == check_name)
}

pub(crate) const STATS_LITE_SUPPORTED_CHECKS: &[StatsLiteCheckSupport] = &[
    StatsLiteCheckSupport {
        check_name: "check-hover-layout-max",
        kind: StatsLiteCheckKind::ReportOnly,
        note: "derived from the stats report",
    },
    StatsLiteCheckSupport {
        check_name: "check-stale-paint",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; resolves semantics via schema2 tables when needed",
    },
    StatsLiteCheckSupport {
        check_name: "check-stale-scene",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; resolves semantics via schema2 tables when needed",
    },
    StatsLiteCheckSupport {
        check_name: "check-idle-no-paint-min",
        kind: StatsLiteCheckKind::FramesIndex,
        note: "uses frames.index.json window aggregates (idle streak tail/max)",
    },
    StatsLiteCheckSupport {
        check_name: "check-pixels-changed",
        kind: StatsLiteCheckKind::OutDirOnly,
        note: "uses out-dir artifacts",
    },
    StatsLiteCheckSupport {
        check_name: "check-semantics-changed-repainted",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; loads schema2 semantics table nodes for diffs",
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
    StatsLiteCheckSupport {
        check_name: "check-gc-sweep-liveness",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.removed_subtrees + element_runtime lengths",
    },
    StatsLiteCheckSupport {
        check_name: "check-retained-vlist-reconcile-no-notify-min",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; scans retained virtual-list reconciles + dirty_views",
    },
    StatsLiteCheckSupport {
        check_name: "check-retained-vlist-keep-alive-reuse-min",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; scans reconcile records for keep-alive reuse",
    },
    StatsLiteCheckSupport {
        check_name: "check-retained-vlist-attach-detach-max",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; sums attach/detach deltas (record or stats fallback)",
    },
    StatsLiteCheckSupport {
        check_name: "check-view-cache-reuse-stable-min",
        kind: StatsLiteCheckKind::FramesIndex,
        note: "uses frames.index.json window aggregates (reuse streak tail/max)",
    },
    StatsLiteCheckSupport {
        check_name: "check-drag-cache-root-paint-only",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; resolves semantics via schema2 tables when needed",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_lite_support_for_returns_some_for_known_checks() {
        assert!(stats_lite_support_for("check-hover-layout-max").is_some());
        assert!(stats_lite_support_for("check-idle-no-paint-min").is_some());
        assert!(stats_lite_support_for("check-retained-vlist-reconcile-no-notify-min").is_some());
        assert!(stats_lite_support_for("check-retained-vlist-keep-alive-reuse-min").is_some());
        assert!(stats_lite_support_for("check-retained-vlist-attach-detach-max").is_some());
        assert!(stats_lite_support_for("check-view-cache-reuse-min").is_some());
        assert!(stats_lite_support_for("check-wheel-scroll").is_some());
        assert!(stats_lite_support_for("check-drag-cache-root-paint-only").is_some());
        assert!(stats_lite_support_for("check-not-a-real-check").is_none());
    }

    #[test]
    fn all_diag_stats_checks_are_stats_lite_supported() {
        // Policy: any `diag stats --check-*` flag should keep working even when the stats report is
        // derived from `frames.index.json` (bundle too large to materialize).
        let checks = [
            "check-stale-paint",
            "check-stale-scene",
            "check-idle-no-paint-min",
            "check-pixels-changed",
            "check-semantics-changed-repainted",
            "check-wheel-scroll",
            "check-wheel-scroll-hit-changes",
            "check-drag-cache-root-paint-only",
            "check-hover-layout-max",
            "check-gc-sweep-liveness",
            "check-notify-hotspot-file-max",
            "check-view-cache-reuse-stable-min",
            "check-view-cache-reuse-min",
            "check-overlay-synthesis-min",
            "check-viewport-input-min",
            "check-dock-drag-min",
            "check-viewport-capture-min",
            "check-retained-vlist-reconcile-no-notify-min",
            "check-retained-vlist-attach-detach-max",
            "check-retained-vlist-keep-alive-reuse-min",
        ];
        for name in checks {
            assert!(
                stats_lite_support_for(name).is_some(),
                "expected `{name}` to be stats-lite supported"
            );
        }
    }
}
