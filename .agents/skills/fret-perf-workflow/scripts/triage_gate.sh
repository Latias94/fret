#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  .agents/skills/fret-perf-workflow/scripts/triage_gate.sh <out-dir> [--all] [--script <script-path-suffix>] [--app-snapshot]

Prints a compact triage summary for perf gate output produced by:
  tools/perf/diag_resize_probes_gate.sh

It reports:
  - which attempts passed/failed,
  - which script/metric exceeded thresholds,
  - and the worst bundle for each failing script (by top_total_time_us).

Options:
  --all
      Also print worst bundles for passing attempts (useful for perf logs).

  --script <script-path-suffix>
      Filter to scripts whose path ends with the given suffix (e.g. "ui-code-editor-resize-probes.json").

  --app-snapshot
      For each reported worst bundle, print a small, jq-derived summary of the worst frame's
      `debug.stats` and (when available) `app_snapshot.code_editor.torture.paint_perf`.

Notes:
  - Gate attempt stdout may include log lines before the JSON payload.
  - The worst bundle lookup uses the JSON payload's `runs[].top_total_time_us`.
USAGE
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: required command not found: $cmd" >&2
    exit 2
  fi
}

out_dir="${1:-}"
if [[ -z "$out_dir" || "$out_dir" == "-h" || "$out_dir" == "--help" ]]; then
  usage
  exit 0
fi
shift || true

print_all=0
script_suffix=""
print_app_snapshot=0
while [[ "${1:-}" != "" ]]; do
  case "$1" in
    --all)
      print_all=1
      shift
      ;;
    --script)
      script_suffix="${2:-}"
      if [[ -z "$script_suffix" ]]; then
        echo "error: --script requires a suffix (e.g. ui-code-editor-resize-probes.json)" >&2
        exit 2
      fi
      shift 2
      ;;
    --app-snapshot)
      print_app_snapshot=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown flag: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

require_cmd jq
require_cmd awk

summary_json="$out_dir/summary.json"
if [[ ! -f "$summary_json" ]]; then
  echo "error: missing summary: $summary_json" >&2
  exit 2
fi

out_dir_abs="$(cd "$out_dir" && pwd)"
summary_out_dir_rel="$(jq -r '.out_dir // ""' "$summary_json" 2>/dev/null || echo "")"

resolve_attempt_dir() {
  local attempt_dir="$1"

  if [[ -z "$attempt_dir" ]]; then
    return 1
  fi

  # 1) absolute path
  if [[ "$attempt_dir" = /* ]]; then
    echo "$attempt_dir"
    return 0
  fi

  # 2) relative to the current working directory
  if [[ -d "$attempt_dir" ]]; then
    echo "$attempt_dir"
    return 0
  fi

  # 3) relative to the gate out_dir (common when summary stores workspace-relative paths)
  if [[ -n "$summary_out_dir_rel" && "$attempt_dir" == "$summary_out_dir_rel"* ]]; then
    local suffix="${attempt_dir#"$summary_out_dir_rel"}"
    suffix="${suffix#/}"
    echo "$out_dir_abs/$suffix"
    return 0
  fi

  # Fall back to the raw value (will likely fail the existence checks).
  echo "$attempt_dir"
}

echo "== gate summary =="
jq -r '"pass=\(.pass) suite=\(.suite) attempts=\(.attempts) pass_attempts=\(.pass_attempts) fail_attempts=\(.fail_attempts) selected=\(.selected_attempt_dir)"' "$summary_json"

echo
echo "== attempts =="
jq -r '.attempt_summaries[] | "\(.attempt_dir)\tpass=\(.pass)\tfailures=\(.check.failures // "missing")\trc=\(.rc)"' "$summary_json"

attempt_dirs="$(jq -r '.attempt_summaries[].attempt_dir' "$summary_json")"

bundle_snapshot_summary() {
  local bundle_json="$1"
  if [[ ! -f "$bundle_json" ]]; then
    return 0
  fi

  # Prefer the "worst frame" snapshot by total_time_us.
  jq -r '
    def snap_total:
      ((.debug.stats.dispatch_time_us // 0)
      + (.debug.stats.layout_time_us // 0)
      + (.debug.stats.prepaint_time_us // 0)
      + (.debug.stats.paint_time_us // 0));
    def max_snap: (.windows[0].snapshots | max_by(snap_total));
    max_snap as $s
    | "  - max frame: total=\($s | snap_total)us paint=\($s.debug.stats.paint_time_us)us layout=\($s.debug.stats.layout_time_us)us prepaint=\($s.debug.stats.prepaint_time_us)us"
  ' "$bundle_json" 2>/dev/null || true

  # Optional code-editor paint perf attribution (only present when the app exposes it).
  jq -r '
    def snap_total:
      ((.debug.stats.dispatch_time_us // 0)
      + (.debug.stats.layout_time_us // 0)
      + (.debug.stats.prepaint_time_us // 0)
      + (.debug.stats.paint_time_us // 0));
    def max_snap: (.windows[0].snapshots | max_by(snap_total));
    (max_snap.app_snapshot.code_editor.torture.paint_perf // null) as $p
    | if $p == null then empty
      else "  - paint_perf: us_total=\($p.us_total)us us_syntax_spans=\($p.us_syntax_spans)us us_text_draw=\($p.us_text_draw)us rows=\($p.rows_painted)"
      end
  ' "$bundle_json" 2>/dev/null || true

  jq -r '
    def snap_total:
      ((.debug.stats.dispatch_time_us // 0)
      + (.debug.stats.layout_time_us // 0)
      + (.debug.stats.prepaint_time_us // 0)
      + (.debug.stats.paint_time_us // 0));
    def max_snap: (.windows[0].snapshots | max_by(snap_total));
    (max_snap.app_snapshot.code_editor.torture.cache_stats // null) as $c
    | if $c == null then empty
      else "  - cache_stats: syntax_resets=\($c.syntax_resets) row_rich_hits=\($c.row_rich_hits) row_rich_misses=\($c.row_rich_misses)"
      end
  ' "$bundle_json" 2>/dev/null || true
}

echo
for attempt_dir in $attempt_dirs; do
  attempt_dir_resolved="$(resolve_attempt_dir "$attempt_dir")"
  check_json="$attempt_dir_resolved/check.perf_thresholds.json"
  if [[ ! -f "$check_json" ]]; then
    continue
  fi

  failures_len="$(jq -r '.failures | length' "$check_json" 2>/dev/null || echo "")"
  if [[ -z "$failures_len" || "$failures_len" == "0" ]]; then
    if [[ "$print_all" != "1" ]]; then
      continue
    fi
  fi

  if [[ "$failures_len" == "0" ]]; then
    echo "== PASS: $attempt_dir =="
  else
    echo "== FAIL: $attempt_dir ($failures_len threshold(s)) =="
    jq -r '.failures[] | "- \(.script) :: \(.metric) actual=\(.actual_us)us threshold=\(.threshold_us)us"' "$check_json"
  fi

  stdout_path="$attempt_dir_resolved/stdout.json"
  if [[ ! -f "$stdout_path" ]]; then
    echo "- note: missing stdout.json (cannot resolve worst bundles)"
    echo
    continue
  fi

  # Extract the JSON payload (skip leading log lines).
  payload="$(
    awk 'BEGIN{f=0} {if(!f && substr($0,1,1)=="{"){f=1} if(f){print}}' "$stdout_path"
  )"

  if [[ -z "$payload" ]]; then
    echo "- note: empty JSON payload (cannot resolve worst bundles)"
    echo
    continue
  fi

  scripts=""
  if [[ "$failures_len" != "0" ]]; then
    scripts="$(jq -r '.failures[].script' "$check_json" | awk '!seen[$0]++')"
  else
    scripts="$(jq -r '.rows[].script' <<<"$payload" 2>/dev/null | awk '!seen[$0]++' || true)"
  fi

  for script in $scripts; do
    if [[ -n "$script_suffix" && "$script" != *"$script_suffix" ]]; then
      continue
    fi
    script_name="$(basename "$script")"
    worst_bundle="$(
      jq -r --arg script_suffix "$script" '
        .rows[]
        | select(.script | endswith($script_suffix))
        | (.runs | max_by(.top_total_time_us) | .bundle)
      ' <<<"$payload" 2>/dev/null || true
    )"
    worst_total="$(
      jq -r --arg script_suffix "$script" '
        .rows[]
        | select(.script | endswith($script_suffix))
        | (.runs | max_by(.top_total_time_us) | .top_total_time_us)
      ' <<<"$payload" 2>/dev/null || true
    )"
    if [[ -n "$worst_bundle" && "$worst_bundle" != "null" ]]; then
      echo "- worst bundle ($script_name): $worst_bundle (top_total_time_us=$worst_total)"
      if [[ "$print_app_snapshot" == "1" ]]; then
        bundle_snapshot_summary "$worst_bundle"
      fi
    else
      echo "- note: could not resolve worst bundle for $script_name"
    fi
  done
  echo
done
