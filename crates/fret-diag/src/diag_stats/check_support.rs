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

impl StatsLiteCheckKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            StatsLiteCheckKind::FramesIndex => "frames-index",
            StatsLiteCheckKind::StreamingBundle => "streaming",
            StatsLiteCheckKind::ReportOnly => "report-only",
            StatsLiteCheckKind::OutDirOnly => "out-dir",
        }
    }
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
        check_name: "check-asset-load-missing-bundle-assets-max",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.resource_loading.asset_load counters only",
    },
    StatsLiteCheckSupport {
        check_name: "check-asset-load-stale-manifest-max",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.resource_loading.asset_load counters only",
    },
    StatsLiteCheckSupport {
        check_name: "check-asset-load-unsupported-file-max",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.resource_loading.asset_load counters only",
    },
    StatsLiteCheckSupport {
        check_name: "check-asset-load-unsupported-url-max",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.resource_loading.asset_load counters only",
    },
    StatsLiteCheckSupport {
        check_name: "check-asset-load-external-reference-unavailable-max",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.resource_loading.asset_load counters only",
    },
    StatsLiteCheckSupport {
        check_name: "check-asset-load-io-max",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.resource_loading.asset_load counters only",
    },
    StatsLiteCheckSupport {
        check_name: "check-asset-load-revision-changes-max",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.resource_loading.asset_load counters only",
    },
    StatsLiteCheckSupport {
        check_name: "check-bundled-font-baseline-source",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.resource_loading.font_environment only",
    },
    StatsLiteCheckSupport {
        check_name: "check-asset-reload-epoch-min",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.resource_loading.asset_reload only",
    },
    StatsLiteCheckSupport {
        check_name: "check-asset-reload-configured-backend",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.resource_loading.asset_reload only",
    },
    StatsLiteCheckSupport {
        check_name: "check-asset-reload-active-backend",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.resource_loading.asset_reload only",
    },
    StatsLiteCheckSupport {
        check_name: "check-asset-reload-fallback-reason",
        kind: StatsLiteCheckKind::StreamingBundle,
        note: "streams bundle JSON; parses debug.resource_loading.asset_reload only",
    },
    StatsLiteCheckSupport {
        check_name: "check-pixels-changed",
        kind: StatsLiteCheckKind::OutDirOnly,
        note: "uses out-dir artifacts",
    },
    StatsLiteCheckSupport {
        check_name: "check-pixels-unchanged",
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

pub(crate) fn stats_lite_support_matrix_json_value() -> serde_json::Value {
    let mut rows: Vec<StatsLiteCheckSupport> = STATS_LITE_SUPPORTED_CHECKS.to_vec();
    rows.sort_by(|a, b| a.check_name.cmp(b.check_name));

    let checks: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "check_name": c.check_name,
                "kind": c.kind.as_str(),
                "note": c.note,
            })
        })
        .collect();

    serde_json::json!({
        "schema_version": 1u64,
        "checks": checks,
    })
}

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
        assert!(stats_lite_support_for("check-asset-load-io-max").is_some());
        assert!(stats_lite_support_for("check-asset-reload-epoch-min").is_some());
        assert!(stats_lite_support_for("check-wheel-scroll").is_some());
        assert!(stats_lite_support_for("check-pixels-unchanged").is_some());
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
            "check-asset-load-missing-bundle-assets-max",
            "check-asset-load-stale-manifest-max",
            "check-asset-load-unsupported-file-max",
            "check-asset-load-unsupported-url-max",
            "check-asset-load-external-reference-unavailable-max",
            "check-asset-load-io-max",
            "check-asset-load-revision-changes-max",
            "check-bundled-font-baseline-source",
            "check-asset-reload-epoch-min",
            "check-asset-reload-configured-backend",
            "check-asset-reload-active-backend",
            "check-asset-reload-fallback-reason",
            "check-pixels-changed",
            "check-pixels-unchanged",
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

    #[test]
    fn stats_lite_support_matrix_json_value_is_sorted_and_complete() {
        let v = stats_lite_support_matrix_json_value();
        assert_eq!(
            v.get("schema_version").and_then(|v| v.as_u64()),
            Some(1),
            "expected schema_version == 1"
        );
        let checks = v
            .get("checks")
            .and_then(|v| v.as_array())
            .expect("expected checks array");
        assert_eq!(checks.len(), STATS_LITE_SUPPORTED_CHECKS.len());

        let mut names: Vec<&str> = Vec::with_capacity(checks.len());
        for c in checks {
            let name = c
                .get("check_name")
                .and_then(|v| v.as_str())
                .expect("expected check_name string");
            let kind = c
                .get("kind")
                .and_then(|v| v.as_str())
                .expect("expected kind string");
            let note = c
                .get("note")
                .and_then(|v| v.as_str())
                .expect("expected note string");
            assert!(!note.is_empty(), "expected non-empty note for {name}");
            assert!(
                matches!(
                    kind,
                    "frames-index" | "streaming" | "report-only" | "out-dir"
                ),
                "unexpected kind for {name}: {kind}"
            );
            names.push(name);
        }

        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted, "expected checks to be sorted by check_name");
    }
}
