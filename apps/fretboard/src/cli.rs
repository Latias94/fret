use std::path::PathBuf;
use std::process::ExitCode;

pub(crate) fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        return help();
    };

    match cmd.as_str() {
        "help" | "-h" | "--help" => help(),
        "assets" => crate::assets::assets_cmd(args.collect()),
        "init" => crate::scaffold::init_cmd(args.collect()),
        "new" => crate::scaffold::new_cmd(args.collect()),
        "config" => crate::config::config_cmd(args.collect()),
        "theme" => crate::theme::theme_cmd(args.collect()),
        "hotpatch" => crate::hotpatch::hotpatch_cmd(args.collect()),
        "diag" => crate::diag::diag_cmd(args.collect()),
        "list" => match args.next().as_deref() {
            Some("native-demos") => crate::demos::list_native_demos(args.collect()),
            Some("web-demos") => crate::demos::list_web_demos(args.collect()),
            Some("cookbook-examples") => crate::demos::list_cookbook_examples(args.collect()),
            Some(other) => Err(format!("unknown list target: {other}")),
            None => Err("missing list target (try: list native-demos)".to_string()),
        },
        "dev" => match args.next().as_deref() {
            Some("native") => crate::dev::dev_native(args.collect()),
            Some("web") => crate::dev::dev_web(args.collect()),
            Some(other) => Err(format!("unknown dev target: {other}")),
            None => Err("missing dev target (try: dev native)".to_string()),
        },
        other => Err(format!("unknown command: {other}")),
    }
}

pub(crate) fn help() -> Result<(), String> {
    println!(
        r#"fretboard dev tooling for the Fret workspace

Usage:
  fretboard help
  fretboard assets manifest write --dir <dir> --out <path> (--app-bundle <name> | --package-bundle <name> | --bundle <id>) [--force]
  fretboard assets rust write --dir <dir> --out <path> (--app-bundle <name> | --package-bundle <name> | --bundle <id>) [--surface <fret|framework>] [--crate-root <dir>] [--force]
  fretboard new [template] [--path <path>] [--name <name>] [--ui-assets] [--icons <lucide|radix|none>] [--command-palette] [--no-check]
  fretboard new             # interactive wizard
  fretboard new todo        # non-interactive (template shortcut)
  fretboard new simple-todo # non-interactive (template shortcut)
  fretboard new hello       # non-interactive (template shortcut)
  fretboard new empty       # minimal Cargo-like project
  fretboard init <template> [...]    # alias for `new` (compat)
  fretboard config menubar [--path <path>] [--force]
  fretboard theme import-vscode <theme.json> [--out <path>] [--base <path>] [--all-tags] [--map <path>] [--set <key=value>...] [--report <path>] [--force]
  fretboard hotpatch poke [--path <path>]        # dev-only (experimental)
  fretboard hotpatch path [--path <path>]        # dev-only (experimental)
  fretboard hotpatch status [--tail <n>]         # dev-only (experimental)
  fretboard hotpatch watch [--path <path>...] [--trigger-path <path>] [--poll-ms <ms>] [--debounce-ms <ms>]  # dev-only (experimental)
  fretboard diag path [--trigger-path <path>] [--dir <dir>]
  fretboard diag poke [--trigger-path <path>] [--dir <dir>] [--label <label>] [--max-snapshots <n>] [--wait] [--record-run] [--run-id <id>]
  fretboard diag latest [--dir <dir>]               # prints the latest bundle dir (session-aware when <dir>/sessions/* exists)
  fretboard diag resolve latest [--dir <base_or_session_dir>] [--within-session <id|latest>] [--json]
  fretboard diag list scripts [--contains <needle>] [--all] [--top <n>] [--case-sensitive] [--json]
  fretboard diag list sessions [--dir <dir>] [--contains <needle>] [--all] [--top <n>] [--case-sensitive] [--json]
  fretboard diag sessions clean [--dir <dir>] --keep <n> [--older-than-days <n>] [--top <n>] [--apply] [--json]
  fretboard diag summarize [<dir|summary.json>...] [--dir <dir>] [--json]
  fretboard diag dashboard [<dir|regression.index.json>] [--dir <dir>] [--top <n>] [--json]
  fretboard diag doctor [<base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>] [--check|--strict] [--fix|--fix-dry-run] [--fix-schema2] [--json]
  fretboard diag doctor scripts [--max-examples <n>] [--json]
  fretboard diag doctor campaigns [--strict] [--json]
  fretboard diag registry <check|write|print> [--path <path>] [--json]
  fretboard diag trace <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json> [--trace-out <path>]
  fretboard diag pack [<base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>] [--dir <dir>] [--pack-out <path>] [--ai-packet] [--ai-only] [--include-all] [--include-root-artifacts] [--include-triage] [--include-screenshots]
  fretboard diag triage <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json> [--top <n>] [--sort <key>] [--warmup-frames <n>] [--json] [--out <path>]
    fretboard diag lint <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json> [--warmup-frames <n>] [--all-test-ids] [--lint-eps-px <px>] [--json] [--out <path>]
    fretboard diag artifact lint [<run_dir|out_dir|manifest.json|script.result.json>] [--warmup-frames <n>] [--json] [--out <path>]
    fretboard diag hotspots [<base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>] [--hotspots-top <n>] [--max-depth <n>] [--min-bytes <n>] [--force] [--json] [--out <path>]
    fretboard diag bundle-v2 [<base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>] [--mode <all|changed|last|off>] [--pretty] [--force] [--json] [--out <path>]
    fretboard diag meta <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json> [--warmup-frames <n>] [--json] [--out <path>]
    fretboard diag index <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json> [--warmup-frames <n>] [--json] [--out <path>]
    fretboard diag test-ids <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json> [--warmup-frames <n>] [--max-test-ids <n>] [--json] [--out <path>]
    fretboard diag memory-summary [<base_or_session_out_dir>] [--within-session <id|latest|all>] [--top-sessions <n>] [--sort-key <key>] [--fit-linear <y_key>:<x_key>] [--top <n>] [--vmmap-regions-sorted-top] [--vmmap-regions-sorted-agg] [--vmmap-regions-sorted-agg-top <n>] [--vmmap-regions-sorted-detail-agg] [--vmmap-regions-sorted-detail-agg-top <n>] [--footprint-categories-agg] [--footprint-categories-agg-top <n>] [--no-recursive] [--max-depth <n>] [--max-samples <n>] [--json] [--out <path>]
    fretboard diag windows <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json> [--warmup-frames <n>] [--json]
    fretboard diag dock-routing <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json> [--warmup-frames <n>] [--json]
    fretboard diag screenshots <out_dir|bundle_dir|bundle.json|bundle.schema2.json> [--json]
    fretboard diag layout-sidecar [<base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>] [--print] [--json] [--out <path>]
    fretboard diag extensions [<base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>] [--key <k>] [--print] [--warmup-frames <n>] [--json] [--out <path>]
    fretboard diag layout-perf-summary [<base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>] [--top <n>] [--warmup-frames <n>] [--json] [--out <path>]
  fretboard diag ai-packet [<base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>] [--test-id <test_id>] [--packet-out <dir>] [--sidecars-only] [--include-triage] [--warmup-frames <n>]
  fretboard diag query test-id [<base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>] <pattern> [--mode <contains|prefix|glob>] [--top <n>] [--case-sensitive] [--json] [--out <path>]
  fretboard diag query scroll-extents-observation [<base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>] [--window <id>] [--top <n>] [--all] [--deep-scan] [--timeline] [--json] [--out <path>]
  fretboard diag slice [<base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>] --test-id <test_id> [--frame-id <n>] [--snapshot-seq <n>] [--window <id>] [--max-matches <n>] [--max-ancestors <n>] [--json] [--out <path>]
  fretboard diag inspect on|off|toggle|status [--consume-clicks <bool>]
  fretboard diag pick-arm
  fretboard diag pick
  fretboard diag pick-script [--pick-script-out <path>]
  fretboard diag pick-apply <script.json> --ptr <json_pointer> [--out <path>]
  fretboard diag script <script.json> [--dir <dir>] [--script-path <path>] [--script-trigger-path <path>]
    Note: for tool-launched runs (`--launch`), prefer `--session-auto` to isolate artifacts under `<dir>/sessions/<session_id>/` (safe for multiple concurrent terminals/AI agents).
  fretboard diag run <script.json|script_id> [--dir <dir>] [--timeout-ms <ms>] [--poll-ms <ms>] [--exit-after-run] [--touch-exit-after-run] [--keep-open] [--script-path <path>] [--script-trigger-path <path>] [--script-result-path <path>] [--script-result-trigger-path <path>] [--devtools-ws-url <ws://...>] [--devtools-token <token>] [--devtools-session-id <id>] [--ai-packet] [--pack] [--pack-out <path>] [--include-all] [--include-root-artifacts] [--include-triage] [--include-screenshots] [--json] [--check-stale-paint <test_id>] [--check-stale-paint-eps <px>] [--check-stale-scene <test_id>] [--check-stale-scene-eps <px>] [--check-idle-no-paint-min <n>] [--check-asset-load-missing-bundle-assets-max <n>] [--check-asset-load-stale-manifest-max <n>] [--check-asset-load-unsupported-file-max <n>] [--check-asset-load-unsupported-url-max <n>] [--check-asset-load-external-reference-unavailable-max <n>] [--check-asset-load-revision-changes-max <n>] [--check-bundled-font-baseline-source <none|bundled_profile>] [--check-triage-hint-absent <code>] [--check-pixels-changed <test_id>] [--check-semantics-changed-repainted] [--dump-semantics-changed-repainted-json] [--check-wheel-scroll <test_id>] [--check-prepaint-actions-min <n>] [--check-chart-sampling-window-shifts-min <n>] [--check-node-graph-cull-window-shifts-min <n>] [--check-node-graph-cull-window-shifts-max <n>] [--check-vlist-visible-range-refreshes-min <n>] [--check-vlist-visible-range-refreshes-max <n>] [--check-vlist-window-shifts-explainable] [--check-vlist-window-shifts-non-retained-max <n>] [--check-vlist-window-shifts-prefetch-max <n>] [--check-vlist-window-shifts-escape-max <n>] [--check-drag-cache-root-paint-only <test_id>] [--check-hover-layout] [--check-hover-layout-max <n>] [--check-gc-sweep-liveness] [--check-view-cache-reuse-min <n>] [--check-view-cache-reuse-stable-min <n>] [--check-overlay-synthesis-min <n>] [--check-viewport-input-min <n>] [--check-dock-drag-min <n>] [--check-viewport-capture-min <n>] [--check-retained-vlist-reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>] [--fixed-frame-delta-ms <ms>] [--env <KEY=VALUE>...] [--launch -- <cmd...>]
  fretboard diag repro <ui-gallery|docking-arbitration|docking-motion-pilot|script.json...> [--dir <dir>] [--timeout-ms <ms>] [--poll-ms <ms>] [--max-working-set-bytes <n>] [--max-peak-working-set-bytes <n>] [--max-cpu-avg-percent-total-cores <pct>] [--script-path <path>] [--script-trigger-path <path>] [--script-result-path <path>] [--script-result-trigger-path <path>] [--pack-out <path>] [--ai-packet] [--ai-only] [--include-all] [--include-root-artifacts] [--include-triage] [--include-screenshots] [--json] [--check-stale-paint <test_id>] [--check-stale-paint-eps <px>] [--check-stale-scene <test_id>] [--check-stale-scene-eps <px>] [--check-idle-no-paint-min <n>] [--check-asset-load-missing-bundle-assets-max <n>] [--check-asset-load-stale-manifest-max <n>] [--check-asset-load-unsupported-file-max <n>] [--check-asset-load-unsupported-url-max <n>] [--check-asset-load-external-reference-unavailable-max <n>] [--check-asset-load-revision-changes-max <n>] [--check-bundled-font-baseline-source <none|bundled_profile>] [--check-triage-hint-absent <code>] [--check-pixels-changed <test_id>] [--check-semantics-changed-repainted] [--dump-semantics-changed-repainted-json] [--check-wheel-scroll <test_id>] [--check-prepaint-actions-min <n>] [--check-chart-sampling-window-shifts-min <n>] [--check-node-graph-cull-window-shifts-min <n>] [--check-node-graph-cull-window-shifts-max <n>] [--check-vlist-visible-range-refreshes-min <n>] [--check-vlist-visible-range-refreshes-max <n>] [--check-vlist-window-shifts-explainable] [--check-vlist-window-shifts-non-retained-max <n>] [--check-vlist-window-shifts-prefetch-max <n>] [--check-vlist-window-shifts-escape-max <n>] [--check-drag-cache-root-paint-only <test_id>] [--check-hover-layout] [--check-hover-layout-max <n>] [--check-gc-sweep-liveness] [--check-view-cache-reuse-min <n>] [--check-view-cache-reuse-stable-min <n>] [--check-overlay-synthesis-min <n>] [--check-viewport-input-min <n>] [--check-dock-drag-min <n>] [--check-viewport-capture-min <n>] [--check-retained-vlist-reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>] [--with <tracy|renderdoc>] [--renderdoc-after-frames <n>] [--renderdoc-marker <substring>] [--renderdoc-no-outputs-png] [--fixed-frame-delta-ms <ms>] [--env <KEY=VALUE>...] [--launch -- <cmd...>]
  fretboard diag script normalize <script.json> [--write|--check]
  fretboard diag script validate <script.json>... [--check-out <path>] [--json]
  fretboard diag script lint <script.json>... [--check-out <path>] [--json]
  fretboard diag script shrink <script.json> [--shrink-out <path>] [--shrink-any-fail] [--shrink-match-reason-code <code>] [--shrink-match-reason <reason>] [--shrink-min-steps <n>] [--shrink-max-iters <n>] [--reuse-launch] [--launch -- <cmd...>]
  fretboard diag run <script.json|script_id> [--dir <dir>] [--timeout-ms <ms>] [--poll-ms <ms>] [--script-path <path>] [--script-trigger-path <path>] [--script-result-path <path>] [--script-result-trigger-path <path>] [--devtools-ws-url <ws://...>] [--devtools-token <token>] [--devtools-session-id <id>] [--ai-packet] [--pack] [--pack-out <path>] [--include-all] [--include-root-artifacts] [--include-triage] [--include-screenshots] [--json] [--check-stale-paint <test_id>] [--check-stale-paint-eps <px>] [--check-stale-scene <test_id>] [--check-stale-scene-eps <px>] [--check-idle-no-paint-min <n>] [--check-asset-load-missing-bundle-assets-max <n>] [--check-asset-load-stale-manifest-max <n>] [--check-asset-load-unsupported-file-max <n>] [--check-asset-load-unsupported-url-max <n>] [--check-asset-load-external-reference-unavailable-max <n>] [--check-asset-load-revision-changes-max <n>] [--check-bundled-font-baseline-source <none|bundled_profile>] [--check-pixels-changed <test_id>] [--check-semantics-changed-repainted] [--dump-semantics-changed-repainted-json] [--check-wheel-scroll <test_id>] [--check-prepaint-actions-min <n>] [--check-chart-sampling-window-shifts-min <n>] [--check-node-graph-cull-window-shifts-min <n>] [--check-node-graph-cull-window-shifts-max <n>] [--check-vlist-visible-range-refreshes-min <n>] [--check-vlist-visible-range-refreshes-max <n>] [--check-vlist-window-shifts-explainable] [--check-vlist-window-shifts-non-retained-max <n>] [--check-vlist-window-shifts-prefetch-max <n>] [--check-vlist-window-shifts-escape-max <n>] [--check-drag-cache-root-paint-only <test_id>] [--check-hover-layout] [--check-hover-layout-max <n>] [--check-gc-sweep-liveness] [--check-view-cache-reuse-min <n>] [--check-view-cache-reuse-stable-min <n>] [--check-overlay-synthesis-min <n>] [--check-viewport-input-min <n>] [--check-dock-drag-min <n>] [--check-viewport-capture-min <n>] [--check-retained-vlist-reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>] [--env <KEY=VALUE>...] [--launch -- <cmd...>]
  fretboard diag repeat <script.json> [--repeat <n>] [--dir <dir>] [--timeout-ms <ms>] [--poll-ms <ms>] [--warmup-frames <n>] [--compare-eps-px <px>] [--compare-ignore-bounds] [--compare-ignore-scene-fingerprint] [--check-memory-p90-max <key>:<bytes>]... [--no-compare] [--json] [--env <KEY=VALUE>...] [--launch -- <cmd...>]
  fretboard diag repro <ui-gallery|docking-arbitration|docking-motion-pilot|script.json...> [--dir <dir>] [--timeout-ms <ms>] [--poll-ms <ms>] [--max-working-set-bytes <n>] [--max-peak-working-set-bytes <n>] [--max-cpu-avg-percent-total-cores <pct>] [--script-path <path>] [--script-trigger-path <path>] [--script-result-path <path>] [--script-result-trigger-path <path>] [--pack-out <path>] [--ai-packet] [--ai-only] [--include-all] [--include-root-artifacts] [--include-triage] [--include-screenshots] [--json] [--check-stale-paint <test_id>] [--check-stale-paint-eps <px>] [--check-stale-scene <test_id>] [--check-stale-scene-eps <px>] [--check-idle-no-paint-min <n>] [--check-asset-load-missing-bundle-assets-max <n>] [--check-asset-load-stale-manifest-max <n>] [--check-asset-load-unsupported-file-max <n>] [--check-asset-load-unsupported-url-max <n>] [--check-asset-load-external-reference-unavailable-max <n>] [--check-asset-load-revision-changes-max <n>] [--check-bundled-font-baseline-source <none|bundled_profile>] [--check-pixels-changed <test_id>] [--check-semantics-changed-repainted] [--dump-semantics-changed-repainted-json] [--check-wheel-scroll <test_id>] [--check-prepaint-actions-min <n>] [--check-chart-sampling-window_shifts-min <n>] [--check-node-graph-cull-window_shifts-min <n>] [--check-node-graph-cull-window_shifts-max <n>] [--check-vlist-visible-range_refreshes-min <n>] [--check-vlist-visible-range_refreshes-max <n>] [--check-vlist-window_shifts-explainable] [--check-vlist-window_shifts-non_retained-max <n>] [--check-vlist-window_shifts-prefetch-max <n>] [--check-vlist-window_shifts-escape-max <n>] [--check-drag-cache-root-paint_only <test_id>] [--check-hover-layout] [--check-hover-layout-max <n>] [--check-gc-sweep_liveness] [--check-view-cache-reuse-min <n>] [--check-view-cache-reuse-stable-min <n>] [--check-overlay_synthesis-min <n>] [--check-viewport_input-min <n>] [--check-dock-drag-min <n>] [--check-viewport_capture-min <n>] [--check-retained-vlist_reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>] [--with <tracy|renderdoc>] [--renderdoc-after-frames <n>] [--renderdoc-marker <substring>] [--renderdoc-no-outputs-png] [--env <KEY=VALUE>...] [--launch -- <cmd...>]
  fretboard diag suite <ui-gallery|ui-gallery-lite-smoke|ui-gallery-overlay-steady|ui-gallery-motion-pilot|ui-gallery-layout|ui-gallery-date-picker|ui-gallery-text-ime|ui-gallery-text-wrap|ui-gallery-combobox|ui-gallery-select|ui-gallery-shadcn-conformance|ui-gallery-cache005|ui-gallery-virt-retained|ui-gallery-tree-retained|ui-gallery-data-table-retained|ui-gallery-table-retained|ui-gallery-retained-measured|ui-gallery-ai-transcript-retained|ui-gallery-canvas-cull|ui-gallery-node-graph-cull|ui-gallery-node-graph-cull-window-shifts|ui-gallery-node-graph-cull-window-no-shifts-small-pan|ui-gallery-chart-torture|ui-gallery-vlist-window-boundary|ui-gallery-vlist-window-boundary-retained|ui-gallery-vlist-no-window-shifts-small-scroll|ui-gallery-ui-kit-list-retained|docking-arbitration|docking-motion-pilot|components-gallery-file-tree|components-gallery-table|script.json...> [--script-dir <dir>] [--glob <pattern>] [--dir <dir>] [--timeout-ms <ms>] [--poll-ms <ms>] [--script-path <path>] [--script-trigger-path <path>] [--script-result-path <path>] [--script-result-trigger-path <path>] [--json] [--no-lint] [--all-test-ids] [--lint-eps-px <px>] [--check-stale-paint <test_id>] [--check-stale-paint-eps <px>] [--check-stale-scene <test_id>] [--check-stale-scene-eps <px>] [--check-idle-no-paint-min <n>] [--check-asset-load-missing-bundle-assets-max <n>] [--check-asset-load-stale-manifest-max <n>] [--check-asset-load-unsupported-file-max <n>] [--check-asset-load-unsupported-url-max <n>] [--check-asset-load-external-reference-unavailable-max <n>] [--check-asset-load-revision-changes-max <n>] [--check-bundled-font-baseline-source <none|bundled_profile>] [--check-triage-hint-absent <code>] [--check-pixels-changed <test_id>] [--check-semantics-changed-repainted] [--dump-semantics-changed-repainted-json] [--check-wheel-scroll <test_id>] [--check-prepaint-actions-min <n>] [--check-chart-sampling-window-shifts-min <n>] [--check-node-graph-cull-window-shifts-min <n>] [--check-node-graph-cull-window-shifts-max <n>] [--check-vlist-visible-range-refreshes-min <n>] [--check-vlist-visible-range-refreshes-max <n>] [--check-vlist-window-shifts-explainable] [--check-vlist-window-shifts-non-retained-max <n>] [--check-vlist-window-shifts-prefetch-max <n>] [--check-vlist-window-shifts-escape-max <n>] [--check-drag-cache-root-paint-only <test_id>] [--check-hover-layout] [--check-hover-layout-max <n>] [--check-gc-sweep-liveness] [--check-view-cache-reuse-min <n>] [--check-view-cache-reuse-stable-min <n>] [--check-overlay-synthesis-min <n>] [--check-viewport-input-min <n>] [--check-dock-drag-min <n>] [--check-viewport-capture-min <n>] [--check-retained-vlist-reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>] [--fixed-frame-delta-ms <ms>] [--env <KEY=VALUE>...] [--launch -- <cmd...>]
  fretboard diag stats <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json> [--diff <bundle_a> <bundle_b>] [--top <n>] [--sort <key>] [--verbose] [--json] [--check-stale-paint <test_id>] [--check-stale-paint-eps <px>] [--check-stale-scene <test_id>] [--check-stale-scene-eps <px>] [--check-idle-no-paint-min <n>] [--check-asset-load-missing-bundle-assets-max <n>] [--check-asset-load-stale-manifest-max <n>] [--check-asset-load-unsupported-file-max <n>] [--check-asset-load-unsupported-url-max <n>] [--check-asset-load-external-reference-unavailable-max <n>] [--check-asset-load-revision-changes-max <n>] [--check-bundled-font-baseline-source <none|bundled_profile>] [--check-triage-hint-absent <code>] [--check-pixels-changed <test_id>] [--check-semantics-changed-repainted] [--dump-semantics-changed-repainted-json] [--check-wheel-scroll <test_id>] [--check-prepaint-actions-min <n>] [--check-chart-sampling-window-shifts-min <n>] [--check-node-graph-cull-window-shifts-min <n>] [--check-node-graph-cull-window-shifts-max <n>] [--check-vlist-visible-range-refreshes-min <n>] [--check-vlist-visible-range-refreshes-max <n>] [--check-vlist-window-shifts-explainable] [--check-vlist-window-shifts-non-retained-max <n>] [--check-vlist-window-shifts-prefetch-max <n>] [--check-vlist-window-shifts-escape-max <n>] [--check-drag-cache-root-paint-only <test_id>] [--check-hover-layout] [--check-hover-layout-max <n>] [--check-gc-sweep-liveness] [--check-view-cache-reuse-min <n>] [--check-view-cache-reuse-stable-min <n>] [--check-overlay-synthesis-min <n>] [--check-viewport-input-min <n>] [--check-dock-drag-min <n>] [--check-viewport-capture-min <n>] [--check-retained-vlist-reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>]
  fretboard diag matrix ui-gallery [--dir <dir>] [--timeout-ms <ms>] [--poll-ms <ms>] [--warmup-frames <n>] [--compare-eps-px <px>] [--compare-ignore-bounds] [--compare-ignore-scene-fingerprint] [--check-view-cache-reuse-min <n>] [--check-view-cache-reuse-stable-min <n>] [--check-overlay-synthesis-min <n>] [--check-viewport-input-min <n>] [--check-dock-drag-min <n>] [--check-viewport-capture-min <n>] [--check-retained-vlist-reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>] [--check-vlist-window-shifts-non-retained-max <n>] [--check-vlist-window-shifts-prefetch-max <n>] [--check-vlist-window-shifts-escape-max <n>] [--env <KEY=VALUE>...] [--launch -- <cmd...>] [--json]
  fretboard diag compare <base_or_session_out_dir|bundle_a|dir> <base_or_session_out_dir|bundle_b|dir> [--footprint] [--warmup-frames <n>] [--compare-eps-px <px>] [--compare-ignore-bounds] [--compare-ignore-scene-fingerprint] [--json]
  fretboard diag perf <ui-gallery|script.json...> [--top <n>] [--sort <invalidation|time>] [--repeat <n>] [--warmup-frames <n>] [--suite-prewarm <script.json>...] [--suite-prelude <script.json>...] [--suite-prelude-each-run] [--trace] [--check-perf-hints] [--check-perf-hints-min-severity <info|warn|error>] [--check-perf-hints-deny <codes>] [--max-top-total-us <n>] [--max-top-layout-us <n>] [--max-top-solve-us <n>] [--max-pointer-move-dispatch-us <n>] [--max-pointer-move-hit-test-us <n>] [--max-pointer-move-global-changes <n>] [--min-run-paint-cache-hit-test-only-replay-allowed-max <n>] [--max-run-paint-cache-hit-test-only-replay-rejected-key-mismatch-max <n>] [--perf-baseline <path>] [--perf-baseline-out <path>] [--perf-baseline-headroom-pct <n>] [--perf-baseline-seed-preset <path>...] [--perf-baseline-seed <scope@metric=max|p90|p95>...] [--timeout-ms <ms>] [--poll-ms <ms>] [--dir <dir>] [--json] [--check-stale-paint <test_id>] [--check-stale-paint-eps <px>] [--check-stale-scene <test_id>] [--check-stale-scene-eps <px>] [--check-pixels-changed <test_id>] [--check-wheel-scroll <test_id>] [--check-prepaint-actions-min <n>] [--check-chart-sampling-window-shifts-min <n>] [--check-node-graph-cull-window-shifts-min <n>] [--check-node-graph-cull-window-shifts-max <n>] [--check-vlist-visible-range-refreshes-min <n>] [--check-vlist-visible-range-refreshes-max <n>] [--check-vlist-window-shifts-explainable] [--check-vlist-window-shifts-non-retained-max <n>] [--check-vlist-window-shifts-prefetch-max <n>] [--check-vlist-window-shifts-escape-max <n>] [--check-drag-cache-root-paint-only <test_id>] [--check-hover-layout] [--check-hover-layout-max <n>] [--check-gc-sweep-liveness] [--check-view-cache-reuse-min <n>] [--check-view-cache-reuse-stable-min <n>] [--check-overlay-synthesis-min <n>] [--check-viewport-input-min <n>] [--check-dock-drag-min <n>] [--check-viewport-capture-min <n>] [--check-retained-vlist-reconcile-no-notify <n>] [--check-retained-vlist-attach-detach-max <n>] [--env <KEY=VALUE>...] [--launch -- <cmd...>]
  fretboard list native-demos [--all]
  fretboard list web-demos
  fretboard list cookbook-examples
  fretboard dev native [--bin <name> | --choose [--all]] [--profile <cargo_profile>] [--hotpatch] [--hotpatch-reload] [--hotpatch-trigger-path <path>] [--hotpatch-poll-ms <ms>] [-- <args...>]
  fretboard dev native --demo <demo> [--profile <cargo_profile>] [--dev-state-reset] [--hotpatch|--watch] [-- <args...>]
  fretboard dev native --example <name> [--profile <cargo_profile>] [-- <args...>]
  fretboard dev native [--bin <name> | --choose [--all]] [--profile <cargo_profile>] [--hotpatch] [--no-supervise] [-- <args...>]
  fretboard dev native [--bin <name> | --choose [--all]] [--profile <cargo_profile>] [--hotpatch] [--watch] [--watch-poll-ms <ms>] [--no-watch] [--dev-state-reset] [-- <args...>]
  fretboard dev native [--bin <name> | --choose [--all]] [--profile <cargo_profile>] --hotpatch-devserver <ws_endpoint> [--hotpatch-build-id <auto|none|u64>] [-- <args...>]
  fretboard dev native [--bin <name> | --choose [--all]] [--profile <cargo_profile>] --hotpatch-dx [--hotpatch-dx-ws <ws_endpoint>] [--hotpatch-build-id <auto|none|u64>] [-- <args...>]
  fretboard dev web [--no-open] [--port <port>] [--demo <demo> | --choose] [--devtools-ws-url <ws://.../> --devtools-token <token>]
  fretboard dev web --open [--no-open] [--port <port>] [--demo <demo> | --choose]

Examples:
  fretboard assets manifest write --dir assets --out assets.manifest.json --app-bundle my-todo
  fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle my-todo
    # `--surface fret` modules expose both `register(app)` and `mount(builder)`
  fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle my-todo --surface framework
    # `--surface framework` modules expose `register(host)` for direct runtime mounting
  fretboard new todo --name my-todo
  fretboard new simple-todo --name my-simple-todo
  fretboard new hello --name hello-world
  fretboard new hello --name hello-world --command-palette
  fretboard new todo --name my-todo --icons none
  fretboard new empty --name my-app
  fretboard config menubar --path .
  fretboard dev native --bin components_gallery
  fretboard dev native --bin todo_demo
  fretboard dev native --bin assets_demo
  fretboard dev native --bin hotpatch_smoke_demo
  fretboard dev native --demo simple-todo
  fretboard dev native --demo simple-todo --hotpatch
  fretboard dev native --choose
  fretboard dev native --bin image_upload_demo -- --help
  fretboard dev native --hotpatch --choose   # hotpatch (prefers `dx serve --hotpatch` when `dx` is available; otherwise falls back to reload-boundary mode)
  fretboard dev native --hotpatch-reload --choose   # file-triggered runner reload boundary (default: `.fret/hotpatch.touch`)
  fretboard hotpatch poke                   # updates `.fret/hotpatch.touch` (triggers a reload)
  fretboard hotpatch status                 # prints hotpatch-related log tails (runner/bootstrap)
  fretboard hotpatch watch                  # polls workspace sources and auto-pokes on change
  fretboard diag poke                      # touches `target/fret-diag/trigger.touch` (dumps diagnostics when enabled)
  fretboard diag poke --wait               # waits for `latest.txt` to update and prints the dump directory
  fretboard diag poke --wait --record-run  # also writes a per-run manifest under `target/fret-diag/<run_id>/manifest.json` (tooling-only)
  fretboard diag latest                    # prints the most recent diagnostics bundle path
  fretboard diag pack                      # zips a bundle directory for sharing (default: latest)
  fretboard diag triage                    # prints a machine-readable triage JSON (built from bundle stats)
  fretboard diag script ./script.json      # writes `target/fret-diag/script.json` and touches `target/fret-diag/script.touch`
  fretboard diag run ./script.json         # pushes script and waits for `script.result.json` (exit 0 on pass, 1 on fail/timeout)
  fretboard diag run tools/diag-scripts/todo-baseline.json --dir target/fret-diag-todo-auto --launch -- cargo run -p fret-demo --bin todo_demo
  fretboard diag run tools/diag-scripts/ui-gallery-intro-idle-screenshot.json --pack --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag repeat tools/diag-scripts/ui-gallery-select-trigger-toggle-close.json --repeat 7 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag run tools/diag-scripts/ui-gallery-web-markdown-editor-source-ime-bridge-attach-baseline.json --devtools-ws-url ws://127.0.0.1:7331/ --devtools-token <token> --check-ui-gallery-web-ime-bridge-enabled
  fretboard diag repro tools/diag-scripts/ui-gallery-code-view-scroll-refresh-pixels-changed.json --check-pixels-changed ui-gallery-code-view-root --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag suite ui-gallery          # runs `tools/diag-scripts/ui-gallery-*.json` sequentially (app must be running)
  fretboard diag suite ui-gallery-layout   # runs layout-focused UI gallery repro scripts (includes a small page sweep)
  fretboard diag suite ui-gallery-date-picker  # runs deterministic date picker regression scripts
  fretboard diag suite ui-gallery-text-ime  # runs deterministic IME injection scripts (shortcut routing + composition)
  fretboard diag suite ui-gallery-text-wrap  # runs text wrap/baseline screenshot gates
  fretboard diag suite ui-gallery-combobox  # runs combobox conformance scripts (overlay + focus + keyboard + typeahead)
  fretboard diag suite ui-gallery-shadcn-conformance  # runs shadcn-focused conformance scripts (behavior + hit-testing + layout)
  fretboard diag suite --glob tools/diag-scripts/ui-gallery-select-*.json  # run a globbed suite (app must be running)
  fretboard diag stats ./target/fret-diag/1234  # summarizes invalidation + other frame stats from a bundle artifact (`bundle.json` or `bundle.schema2.json`)
  fretboard diag perf ui-gallery --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf ui-gallery --warmup-frames 5 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf ui-gallery --repeat 5 --warmup-frames 5 --max-top-total-us 25000 --max-top-layout-us 15000 --max-top-solve-us 8000 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf ui-gallery-steady --repeat 5 --warmup-frames 5 --perf-baseline-out .fret/perf.baseline.json --perf-baseline-headroom-pct 20 --perf-baseline-seed ui-gallery-steady@top_total_time_us=p90 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf ui-gallery-steady --repeat 5 --warmup-frames 5 --perf-baseline-out .fret/perf.baseline.json --perf-baseline-seed this-suite@top_layout_time_us=p90 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf ui-gallery-steady --repeat 5 --warmup-frames 5 --perf-baseline-seed-preset docs/workstreams/perf-baselines/policies/ui-gallery-steady.v1.json --perf-baseline-seed this-suite@top_layout_time_us=p90 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf ui-gallery --repeat 5 --warmup-frames 5 --perf-baseline .fret/perf.baseline.json --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf-baseline-from-bundles tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json .fret/diag/exports/1234 --perf-baseline-out .fret/perf.web.baseline.json
  fretboard diag perf tools/diag-scripts/ui-gallery-hit-test-only-paint-cache-probe-sweep.json --repeat 3 --min-run-paint-cache-hit-test-only-replay-allowed-max 10 --max-run-paint-cache-hit-test-only-replay-rejected-key-mismatch-max 0 --env FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY=1 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf ui-gallery --repeat 7 --warmup-frames 5 --sort time --json --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag perf tools/diag-scripts/ui-gallery-overlay-torture.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --warmup-frames 5 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag run tools/diag-scripts/ui-gallery-modal-barrier-underlay-block.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --check-view-cache-reuse-min 1 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag run tools/diag-scripts/ui-gallery-modal-barrier-underlay-block.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --check-view-cache-reuse-stable-min 10 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag run tools/diag-scripts/ui-gallery-modal-barrier-underlay-block.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --check-view-cache-reuse-min 1 --check-overlay-synthesis-min 1 --warmup-frames 5 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag matrix ui-gallery --dir target/fret-diag --warmup-frames 5 --compare-ignore-bounds --compare-ignore-scene-fingerprint --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag matrix ui-gallery --dir target/fret-diag --warmup-frames 5 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag compare ./target/fret-diag/uncached ./target/fret-diag/cached --warmup-frames 5 --compare-ignore-bounds --compare-ignore-scene-fingerprint --json
  fretboard diag summarize --dir target/fret-diag/campaigns/ui-gallery-pr
  fretboard diag dashboard --dir target/fret-diag/campaigns/ui-gallery-pr --top 10
  fretboard diag campaign list --lane smoke --tag ui-gallery --platform native
  fretboard diag campaign validate
  fretboard diag campaign validate tools/diag-campaigns/ui-gallery-smoke.json --json
  fretboard diag doctor campaigns --json
  fretboard diag campaign run --lane smoke --tag ui-gallery --platform native --launch -- cargo run -p fret-ui-gallery --release
  fretboard diag campaign share target/fret-diag/campaign-batches/filtered-lane-smoke-tag-ui-gallery-platform-native-2-campaigns/1234
  fretboard dev native --hotpatch-devserver ws://127.0.0.1:8080/_dioxus
  fretboard dev native --bin hotpatch_smoke_demo --hotpatch
  fretboard dev native --bin hotpatch_smoke_demo --hotpatch-dx
  fretboard dev web --demo plot_demo
  fretboard dev web --demo custom_effect_v2_web_demo
"#
    );
    Ok(())
}

pub(crate) fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for dir in cwd.ancestors() {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir.to_path_buf());
        }
    }
    Err("failed to locate workspace root (Cargo.toml not found in ancestors)".to_string())
}
