#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  tools/perf/diag_vlist_boundary_gate.sh \
    [--runs <n>] \
    [--out-dir <path>] \
    [--script <path>] \
    [--launch-bin <path>] \
    [--timeout-ms <n>] \
    [--prefetch-max <n>] \
    [--escape-max <n>] \
    [--non-retained-max <n>]

Notes:
  - Runs the retained VirtualList window-boundary crossing gate repeatedly.
  - Uses env profile:
      FRET_UI_GALLERY_VIEW_CACHE=1
      FRET_UI_GALLERY_VIEW_CACHE_SHELL=1
      FRET_UI_GALLERY_VLIST_MINIMAL=1
      FRET_DIAG_SCRIPT_AUTO_DUMP=0
USAGE
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: required command not found: $cmd" >&2
    exit 2
  fi
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

runs=3
out_dir="target/fret-diag-vlist-boundary-gate-$(date +%s)"
script_path="tools/diag-scripts/ui-gallery-virtual-list-window-boundary-crossing-steady.json"
launch_bin="target/release/fret-ui-gallery"
timeout_ms=300000
prefetch_max=3
escape_max=0
non_retained_max=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --runs)
      runs="$2"
      shift 2
      ;;
    --out-dir)
      out_dir="$2"
      shift 2
      ;;
    --script)
      script_path="$2"
      shift 2
      ;;
    --launch-bin)
      launch_bin="$2"
      shift 2
      ;;
    --timeout-ms)
      timeout_ms="$2"
      shift 2
      ;;
    --prefetch-max)
      prefetch_max="$2"
      shift 2
      ;;
    --escape-max)
      escape_max="$2"
      shift 2
      ;;
    --non-retained-max)
      non_retained_max="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

require_cmd cargo
require_cmd jq

cd "$WORKSPACE_ROOT"
mkdir -p "$out_dir"

results='[]'
run_failures=0

for ((i=1; i<=runs; i++)); do
  run_dir="$out_dir/run-${i}"
  mkdir -p "$run_dir"

  cmd=(
    cargo run -q -p fretboard --
    diag run "$script_path"
    --dir "$run_dir"
    --timeout-ms "$timeout_ms"
    --check-vlist-window-shifts-explainable
    --check-vlist-window-shifts-have-prepaint-actions
    --check-vlist-window-shifts-non-retained-max "$non_retained_max"
    --check-vlist-window-shifts-prefetch-max "$prefetch_max"
    --check-vlist-window-shifts-escape-max "$escape_max"
    --env FRET_UI_GALLERY_VIEW_CACHE=1
    --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1
    --env FRET_UI_GALLERY_VLIST_MINIMAL=1
    --env FRET_DIAG_SCRIPT_AUTO_DUMP=0
    --launch -- "$launch_bin"
  )

  echo "[run] ${i}/${runs} -> ${run_dir}"
  rc=0
  if ! "${cmd[@]}" > "$run_dir/stdout.log" 2> "$run_dir/stderr.log"; then
    rc=$?
  fi

  explainable_file="$run_dir/check.vlist_window_shifts_explainable.json"
  prepaint_file="$run_dir/check.vlist_window_shifts_have_prepaint_actions.json"
  prefetch_file="$run_dir/check.vlist_window_shifts_prefetch_max.json"
  escape_file="$run_dir/check.vlist_window_shifts_escape_max.json"
  non_retained_file="$run_dir/check.vlist_window_shifts_non_retained_max.json"

  total_shifts=0
  prefetch=0
  escape=0
  non_retained=0
  explainable_failures=0
  prepaint_failures=0

  [[ -f "$explainable_file" ]] && total_shifts="$(jq '.total_shifts // 0' "$explainable_file")"
  [[ -f "$prefetch_file" ]] && prefetch="$(jq '.total_kind_shifts // 0' "$prefetch_file")"
  [[ -f "$escape_file" ]] && escape="$(jq '.total_kind_shifts // 0' "$escape_file")"
  [[ -f "$non_retained_file" ]] && non_retained="$(jq '.total_non_retained_shifts // 0' "$non_retained_file")"
  [[ -f "$explainable_file" ]] && explainable_failures="$(jq '.failures | length' "$explainable_file")"
  [[ -f "$prepaint_file" ]] && prepaint_failures="$(jq '.failures | length' "$prepaint_file")"

  check_failures=$((explainable_failures + prepaint_failures))
  gate_pass=1
  if [[ "$rc" -ne 0 || "$check_failures" -ne 0 ]]; then
    gate_pass=0
    run_failures=$((run_failures + 1))
  fi

  echo "  rc=${rc} shifts=${total_shifts} prefetch=${prefetch} escape=${escape} non_retained=${non_retained} check_failures=${check_failures}"

  results="$(jq \
    --arg run_dir "$run_dir" \
    --argjson rc "$rc" \
    --argjson gate_pass "$gate_pass" \
    --argjson total_shifts "$total_shifts" \
    --argjson prefetch "$prefetch" \
    --argjson escape "$escape" \
    --argjson non_retained "$non_retained" \
    --argjson explainable_failures "$explainable_failures" \
    --argjson prepaint_failures "$prepaint_failures" \
    '. + [{
      run_dir:$run_dir,
      exit_code:$rc,
      gate_pass:($gate_pass==1),
      total_shifts:$total_shifts,
      prefetch:$prefetch,
      escape:$escape,
      non_retained:$non_retained,
      explainable_failures:$explainable_failures,
      prepaint_failures:$prepaint_failures
    }]' \
    <<<"$results")"
done

summary_path="$out_dir/summary.json"
summary="$(jq -n \
  --arg script "$script_path" \
  --arg launch_bin "$launch_bin" \
  --argjson runs "$runs" \
  --argjson prefetch_max "$prefetch_max" \
  --argjson escape_max "$escape_max" \
  --argjson non_retained_max "$non_retained_max" \
  --argjson run_failures "$run_failures" \
  --argjson results "$results" \
  '{
    schema_version:1,
    script:$script,
    launch_bin:$launch_bin,
    thresholds:{prefetch_max:$prefetch_max,escape_max:$escape_max,non_retained_max:$non_retained_max},
    runs:$runs,
    run_failures:$run_failures,
    pass:($run_failures==0),
    results:$results
  }')"

echo "$summary" > "$summary_path"
echo "[summary] $summary_path"
jq '. | {pass, run_failures, thresholds, runs, sample: (.results | map({exit_code, total_shifts, prefetch, escape, non_retained, gate_pass}))}' "$summary_path"

if [[ "$run_failures" -ne 0 ]]; then
  exit 1
fi
