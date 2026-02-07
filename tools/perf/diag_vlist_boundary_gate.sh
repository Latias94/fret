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
    [--retained <0|1>] \
    [--prefetch-max <n>] \
    [--escape-max <n>] \
    [--non-retained-max <n>] \
    [--max-cache-key-mismatch <n>] \
    [--max-needs-rerender <n>]

Notes:
  - Runs VirtualList window-boundary crossing gate repeatedly.
  - Default profile is retained (`--retained 1`).
  - Common env profile:
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
retained=1
prefetch_max=3
escape_max=0
non_retained_max=0
max_cache_key_mismatch=0
max_needs_rerender=0

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
    --retained)
      retained="$2"
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
    --max-cache-key-mismatch)
      max_cache_key_mismatch="$2"
      shift 2
      ;;
    --max-needs-rerender)
      max_needs_rerender="$2"
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

if [[ "$retained" != "0" && "$retained" != "1" ]]; then
  echo "error: --retained must be 0 or 1 (got: $retained)" >&2
  exit 2
fi

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
  )

  if [[ "$retained" == "0" ]]; then
    cmd+=(--env FRET_UI_GALLERY_VLIST_RETAINED=0)
  fi

  cmd+=(--launch -- "$launch_bin")

  echo "[run] ${i}/${runs} -> ${run_dir} (retained=${retained})"
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
  cache_key_mismatch_max=0
  needs_rerender_max=0
  cache_key_budget_ok=1
  needs_rerender_budget_ok=1

  [[ -f "$explainable_file" ]] && total_shifts="$(jq '.total_shifts // 0' "$explainable_file")"
  [[ -f "$prefetch_file" ]] && prefetch="$(jq '.total_kind_shifts // 0' "$prefetch_file")"
  [[ -f "$escape_file" ]] && escape="$(jq '.total_kind_shifts // 0' "$escape_file")"
  [[ -f "$non_retained_file" ]] && non_retained="$(jq '.total_non_retained_shifts // 0' "$non_retained_file")"
  [[ -f "$explainable_file" ]] && explainable_failures="$(jq '.failures | length' "$explainable_file")"
  [[ -f "$prepaint_file" ]] && prepaint_failures="$(jq '.failures | length' "$prepaint_file")"

  if [[ -f "$run_dir/latest.txt" ]]; then
    latest_dir="$(cat "$run_dir/latest.txt")"
    bundle_path="$run_dir/$latest_dir/bundle.json"
    if [[ -f "$bundle_path" ]]; then
      cache_key_mismatch_max="$(jq '[.windows[]?.snapshots[]?.debug?.stats?.view_cache_roots_cache_key_mismatch // 0] | max // 0' "$bundle_path")"
      needs_rerender_max="$(jq '[.windows[]?.snapshots[]?.debug?.stats?.view_cache_roots_needs_rerender // 0] | max // 0' "$bundle_path")"
    fi
  fi

  if (( cache_key_mismatch_max > max_cache_key_mismatch )); then
    cache_key_budget_ok=0
  fi
  if (( needs_rerender_max > max_needs_rerender )); then
    needs_rerender_budget_ok=0
  fi

  check_failures=$((explainable_failures + prepaint_failures))
  gate_pass=1
  if [[ "$rc" -ne 0 || "$check_failures" -ne 0 || "$cache_key_budget_ok" -ne 1 || "$needs_rerender_budget_ok" -ne 1 ]]; then
    gate_pass=0
    run_failures=$((run_failures + 1))
  fi

  echo "  rc=${rc} shifts=${total_shifts} prefetch=${prefetch} escape=${escape} non_retained=${non_retained} cache_key_mismatch_max=${cache_key_mismatch_max} needs_rerender_max=${needs_rerender_max} check_failures=${check_failures}"

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
    --argjson cache_key_mismatch_max "$cache_key_mismatch_max" \
    --argjson needs_rerender_max "$needs_rerender_max" \
    --argjson cache_key_budget_ok "$cache_key_budget_ok" \
    --argjson needs_rerender_budget_ok "$needs_rerender_budget_ok" \
    '. + [{
      run_dir:$run_dir,
      exit_code:$rc,
      gate_pass:($gate_pass==1),
      total_shifts:$total_shifts,
      prefetch:$prefetch,
      escape:$escape,
      non_retained:$non_retained,
      explainable_failures:$explainable_failures,
      prepaint_failures:$prepaint_failures,
      cache_key_mismatch_max:$cache_key_mismatch_max,
      needs_rerender_max:$needs_rerender_max,
      cache_key_budget_ok:($cache_key_budget_ok==1),
      needs_rerender_budget_ok:($needs_rerender_budget_ok==1)
    }]' \
    <<<"$results")"
done

summary_path="$out_dir/summary.json"
summary="$(jq -n \
  --arg script "$script_path" \
  --arg launch_bin "$launch_bin" \
  --argjson retained "$retained" \
  --argjson runs "$runs" \
  --argjson prefetch_max "$prefetch_max" \
  --argjson escape_max "$escape_max" \
  --argjson non_retained_max "$non_retained_max" \
  --argjson max_cache_key_mismatch "$max_cache_key_mismatch" \
  --argjson max_needs_rerender "$max_needs_rerender" \
  --argjson run_failures "$run_failures" \
  --argjson results "$results" \
  '{
    schema_version:1,
    script:$script,
    launch_bin:$launch_bin,
    profile:{retained:($retained==1)},
    thresholds:{
      prefetch_max:$prefetch_max,
      escape_max:$escape_max,
      non_retained_max:$non_retained_max,
      max_cache_key_mismatch:$max_cache_key_mismatch,
      max_needs_rerender:$max_needs_rerender
    },
    runs:$runs,
    run_failures:$run_failures,
    pass:($run_failures==0),
    results:$results
  }')"

echo "$summary" > "$summary_path"
echo "[summary] $summary_path"
jq '. | {pass, run_failures, profile, thresholds, runs, sample: (.results | map({exit_code, total_shifts, prefetch, escape, non_retained, cache_key_mismatch_max, needs_rerender_max, gate_pass}))}' "$summary_path"

if [[ "$run_failures" -ne 0 ]]; then
  exit 1
fi
