#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  tools/perf/diag_perf_baseline_select.sh \
    --baseline-out <path> \
    [--suite <name>] \
    [--preset <path>]... \
    [--candidates <n>] \
    [--validate-runs <n>] \
    [--repeat <n>] \
    [--warmup-frames <n>] \
    [--headroom-pct <n>] \
    [--threshold-surface <ui|renderer|all>] \
    [--work-dir <path>] \
    [--launch-bin <path>] \
    [--prewarm-script <path>] \
    [--prelude-script <path>] \
    [--reuse-launch-per-script] \
    [--no-default-suite-hooks] \
    [--allow-failures]

Notes:
  - Designed for Fret `diag perf` baseline generation/selection.
  - By default, applies the font prewarm and reset-diagnostics prelude hooks used by the perf workstream.
  - By default, uses the UI threshold surface; renderer timings stay measured but are not hard
    thresholds unless --threshold-surface renderer/all is passed.
  - Validation repeats use the same repeat count as baseline generation.
  - The selected candidate must have zero validation failures unless --allow-failures is passed.
  - Candidate winner priority:
      1) fewer validation failures
      2) lower suite p90 (sum of top_total_time_us)
      3) lower sum of max_top_total_us thresholds
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

suite="ui-gallery-steady"
baseline_out=""
candidates=2
validate_runs=3
repeat=7
warmup_frames=5
headroom_pct=20
threshold_surface="ui"
work_dir="target/fret-diag-baseline-select-$(date +%s)"
launch_bin="target/release/fret-ui-gallery"
default_suite_hooks=true
reuse_launch_per_script=false
allow_failures=false
declare -a preset_paths=()
declare -a prewarm_scripts=()
declare -a prelude_scripts=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --baseline-out)
      baseline_out="$2"
      shift 2
      ;;
    --suite)
      suite="$2"
      shift 2
      ;;
    --preset)
      preset_paths+=("$2")
      shift 2
      ;;
    --candidates)
      candidates="$2"
      shift 2
      ;;
    --validate-runs)
      validate_runs="$2"
      shift 2
      ;;
    --repeat)
      repeat="$2"
      shift 2
      ;;
    --warmup-frames)
      warmup_frames="$2"
      shift 2
      ;;
    --headroom-pct)
      headroom_pct="$2"
      shift 2
      ;;
    --threshold-surface)
      threshold_surface="$2"
      shift 2
      ;;
    --work-dir)
      work_dir="$2"
      shift 2
      ;;
    --launch-bin)
      launch_bin="$2"
      shift 2
      ;;
    --prewarm-script)
      prewarm_scripts+=("$2")
      shift 2
      ;;
    --prelude-script)
      prelude_scripts+=("$2")
      shift 2
      ;;
    --reuse-launch-per-script)
      reuse_launch_per_script=true
      shift
      ;;
    --no-default-suite-hooks)
      default_suite_hooks=false
      shift
      ;;
    --allow-failures)
      allow_failures=true
      shift
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

if [[ -z "$baseline_out" ]]; then
  echo "error: --baseline-out is required" >&2
  usage
  exit 2
fi

require_cmd cargo
require_cmd jq

cd "$WORKSPACE_ROOT"

mkdir -p "$work_dir"
baseline_out_abs="$WORKSPACE_ROOT/$baseline_out"
if [[ "$baseline_out" = /* ]]; then
  baseline_out_abs="$baseline_out"
fi

if [[ "$default_suite_hooks" == "true" ]]; then
  prewarm_scripts=("tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json" "${prewarm_scripts[@]}")
  prelude_scripts=("tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json" "${prelude_scripts[@]}")
fi

for hook in "${prewarm_scripts[@]}" "${prelude_scripts[@]}"; do
  if [[ ! -f "$hook" ]]; then
    echo "error: suite hook script not found: $hook" >&2
    exit 2
  fi
done

printf '[select] prewarm:'
printf ' %s' "${prewarm_scripts[@]}"
echo
printf '[select] prelude:'
printf ' %s' "${prelude_scripts[@]}"
echo

candidate_results_path="$work_dir/candidate-results.json"
candidate_results_payload='[]'

best_candidate=""
best_failures=999999
best_resize_p90=999999999999
best_threshold_sum=999999999999

run_baseline() {
  local candidate_idx="$1"
  local candidate_baseline="$2"
  local candidate_out_dir="$3"

  local cmd=(
    cargo run -q -p fretboard --
    diag perf "$suite"
    --dir "$candidate_out_dir"
    --timeout-ms 300000
  )
  for script in "${prewarm_scripts[@]}"; do
    cmd+=(--prewarm-script "$script")
  done
  for script in "${prelude_scripts[@]}"; do
    cmd+=(--prelude-script "$script")
  done
  cmd+=(
    --reuse-launch
  )
  if [[ "$reuse_launch_per_script" == "true" ]]; then
    cmd+=(--reuse-launch-per-script)
  fi
  cmd+=(
    --repeat "$repeat"
    --warmup-frames "$warmup_frames"
    --sort time
    --top 5
    --json
    --perf-baseline-out "$candidate_baseline"
    --perf-baseline-headroom-pct "$headroom_pct"
    --perf-baseline-threshold-surface "$threshold_surface"
  )

  if ((${#preset_paths[@]})); then
    for preset in "${preset_paths[@]}"; do
      cmd+=(--perf-baseline-seed-preset "$preset")
    done
  fi

  cmd+=(
    --env FRET_UI_GALLERY_VIEW_CACHE=1
    --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1
    --env FRET_DIAG_SCRIPT_AUTO_DUMP=0
    --env FRET_DIAG_SEMANTICS=0
    --launch -- "$launch_bin"
  )

  echo "[baseline] candidate=${candidate_idx} out=${candidate_baseline}"
  "${cmd[@]}" > "$candidate_out_dir/stdout.json" 2> "$candidate_out_dir/stderr.log"
}

run_validation() {
  local candidate_idx="$1"
  local validation_idx="$2"
  local candidate_baseline="$3"
  local validation_out_dir="$4"

  local cmd=(
    cargo run -q -p fretboard --
    diag perf "$suite"
    --dir "$validation_out_dir"
    --timeout-ms 300000
  )
  for script in "${prewarm_scripts[@]}"; do
    cmd+=(--prewarm-script "$script")
  done
  for script in "${prelude_scripts[@]}"; do
    cmd+=(--prelude-script "$script")
  done
  cmd+=(
    --reuse-launch
  )
  if [[ "$reuse_launch_per_script" == "true" ]]; then
    cmd+=(--reuse-launch-per-script)
  fi
  cmd+=(
    --repeat "$repeat"
    --warmup-frames "$warmup_frames"
    --sort time
    --top 3
    --json
    --perf-baseline "$candidate_baseline"
    --env FRET_UI_GALLERY_VIEW_CACHE=1
    --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1
    --env FRET_DIAG_SCRIPT_AUTO_DUMP=0
    --env FRET_DIAG_SEMANTICS=0
    --launch -- "$launch_bin"
  )

  echo "[validate] candidate=${candidate_idx} run=${validation_idx}"
  if "${cmd[@]}" > "$validation_out_dir/stdout.json" 2> "$validation_out_dir/stderr.log"; then
    return 0
  fi
  return 1
}

for ((i=1; i<=candidates; i++)); do
  candidate_name="candidate-${i}"
  candidate_baseline="$work_dir/${candidate_name}.baseline.json"
  candidate_baseline_out_dir="$work_dir/${candidate_name}-baseline"
  mkdir -p "$candidate_baseline_out_dir"

  run_baseline "$i" "$candidate_baseline" "$candidate_baseline_out_dir"

  fail_total=0
  fail_details='[]'

  for ((j=1; j<=validate_runs; j++)); do
    validation_out_dir="$work_dir/${candidate_name}-validate-${j}"
    mkdir -p "$validation_out_dir"

    validation_rc=0
    if ! run_validation "$i" "$j" "$candidate_baseline" "$validation_out_dir"; then
      validation_rc=1
    fi

    check_file="$validation_out_dir/check.perf_thresholds.json"
    if [[ ! -f "$check_file" ]]; then
      echo "error: missing validation report: $check_file" >&2
      exit 3
    fi

    failures="$(jq '.failures | length' "$check_file")"
    fail_total=$((fail_total + failures))

    fail_details="$(jq \
      --arg out "$validation_out_dir" \
      --argjson rc "$validation_rc" \
      --argjson failures "$failures" \
      '. + [{out_dir:$out, exit_code:$rc, failures:$failures}]' \
      <<<"$fail_details")"
  done

  resize_p90="$(jq -r '[.rows[].measured_p90.top_total_time_us // 0] | add' "$candidate_baseline")"
  threshold_sum="$(jq '[.rows[].thresholds.max_top_total_us] | add' "$candidate_baseline")"

  echo "[candidate] name=${candidate_name} fail_total=${fail_total} resize_p90=${resize_p90} threshold_sum=${threshold_sum}"

  candidate_results_payload="$(jq \
    --arg name "$candidate_name" \
    --arg baseline "$candidate_baseline" \
    --argjson fail_total "$fail_total" \
    --argjson resize_p90 "$resize_p90" \
    --argjson threshold_sum "$threshold_sum" \
    --argjson validate "$fail_details" \
    '. + [{name:$name, baseline:$baseline, fail_total:$fail_total, resize_p90:$resize_p90, threshold_sum:$threshold_sum, validate_runs:$validate}]' \
    <<<"$candidate_results_payload")"

  if (( fail_total < best_failures )); then
    best_candidate="$candidate_baseline"
    best_failures=$fail_total
    best_resize_p90=$resize_p90
    best_threshold_sum=$threshold_sum
  elif (( fail_total == best_failures )); then
    if (( resize_p90 < best_resize_p90 )); then
      best_candidate="$candidate_baseline"
      best_resize_p90=$resize_p90
      best_threshold_sum=$threshold_sum
    elif (( resize_p90 == best_resize_p90 )); then
      if (( threshold_sum < best_threshold_sum )); then
        best_candidate="$candidate_baseline"
        best_threshold_sum=$threshold_sum
      fi
    fi
  fi
done

if [[ -z "$best_candidate" ]]; then
  echo "error: no candidate selected" >&2
  exit 3
fi

printf '%s\n' "$candidate_results_payload" > "$candidate_results_path"

prewarm_scripts_json="[]"
if [[ "${#prewarm_scripts[@]}" -gt 0 ]]; then
  prewarm_scripts_json="$(printf '%s\n' "${prewarm_scripts[@]}" | jq -R . | jq -s .)"
fi
prelude_scripts_json="[]"
if [[ "${#prelude_scripts[@]}" -gt 0 ]]; then
  prelude_scripts_json="$(printf '%s\n' "${prelude_scripts[@]}" | jq -R . | jq -s .)"
fi

summary_file="$work_dir/selection-summary.json"
jq -n \
  --arg suite "$suite" \
  --arg baseline_out "$baseline_out_abs" \
  --arg best_candidate "$best_candidate" \
  --arg threshold_surface "$threshold_surface" \
  --argjson best_failures "$best_failures" \
  --argjson best_resize_p90 "$best_resize_p90" \
  --argjson best_threshold_sum "$best_threshold_sum" \
  --argjson prewarm_scripts "$prewarm_scripts_json" \
  --argjson prelude_scripts "$prelude_scripts_json" \
  --argjson default_suite_hooks "$default_suite_hooks" \
  --argjson reuse_launch_per_script "$reuse_launch_per_script" \
  --argjson repeat "$repeat" \
  --argjson allow_failures "$([[ "$allow_failures" == "true" ]] && echo true || echo false)" \
  --argjson candidate_results "$candidate_results_payload" \
  '{
    schema_version: 1,
    kind: "perf_baseline_selection",
    suite: $suite,
    baseline_out: $baseline_out,
    suite_hooks: {
      prewarm: $prewarm_scripts,
      prelude: $prelude_scripts,
      reuse_launch_per_script: $reuse_launch_per_script,
      default_suite_hooks: $default_suite_hooks
    },
    threshold_surface: $threshold_surface,
    validate_repeat: $repeat,
    allow_failures: $allow_failures,
    best_candidate: {
      path: $best_candidate,
      fail_total: $best_failures,
      resize_p90_top_total_us: $best_resize_p90,
      suite_p90_total_time_us_sum: $best_resize_p90,
      threshold_sum_max_top_total_us: $best_threshold_sum
    },
    candidates: $candidate_results
  }' > "$summary_file"

if ((best_failures != 0)) && [[ "$allow_failures" != "true" ]]; then
  echo "error: selected candidate still has validation failures (fail_total=${best_failures}). See: ${summary_file}" >&2
  exit 4
fi

mkdir -p "$(dirname "$baseline_out_abs")"
cp "$best_candidate" "$baseline_out_abs"

echo "[done] baseline_out=${baseline_out_abs}"
echo "[done] candidate_results=${candidate_results_path}"
echo "[done] summary=${summary_file}"
