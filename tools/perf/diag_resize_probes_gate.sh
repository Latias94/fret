#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  tools/perf/diag_resize_probes_gate.sh \
    [--suite <name>] \
    [--out-dir <path>] \
    [--baseline <path>] \
    [--launch-bin <path>] \
    [--timeout-ms <n>] \
    [--attempts <n>] \
    [--repeat <n>] \
    [--warmup-frames <n>] \
    [--prewarm-script <path>] \
    [--prelude-script <path>] \
    [--no-default-suite-hooks]

Notes:
  - Runs a resize-focused perf suite via `fretboard-dev diag perf` (defaults to `ui-resize-probes`).
  - Intended to be a lightweight "P0 resize must not regress" gate.
  - If `--baseline` is omitted, selects the checked-in Windows RTX4090 or macOS baseline for the host platform.
  - By default, applies the font prewarm and reset-diagnostics prelude hooks used by the perf workstream.
  - `--attempts` reruns the suite to reduce flakiness from rare tail outliers.
    The gate passes if a strict majority of attempts pass.
  - Common env profile:
      FRET_UI_GALLERY_VIEW_CACHE=1
      FRET_UI_GALLERY_VIEW_CACHE_SHELL=1
      FRET_DIAG_SCRIPT_AUTO_DUMP=0
      FRET_DIAG_SEMANTICS=0
USAGE
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: required command not found: $cmd" >&2
    exit 2
  fi
}

host_platform_key() {
  case "$(uname -s 2>/dev/null || echo unknown)" in
    MINGW*|MSYS*|CYGWIN*|Windows_NT)
      echo "windows"
      ;;
    Darwin*)
      echo "macos"
      ;;
    *)
      echo "unknown"
      ;;
  esac
}

default_baseline_for_suite() {
  local suite_name="$1"
  local platform_key="$2"
  case "$suite_name:$platform_key" in
    ui-resize-probes:windows)
      echo "docs/workstreams/perf-baselines/ui-resize-probes.windows-rtx4090.v1.json"
      ;;
    ui-resize-probes:macos)
      echo "docs/workstreams/perf-baselines/ui-resize-probes.macos-m4.v3.json"
      ;;
    ui-code-editor-resize-probes:windows)
      echo "docs/workstreams/perf-baselines/ui-code-editor-resize-probes.windows-rtx4090.v1.json"
      ;;
    ui-code-editor-resize-probes:macos)
      echo "docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json"
      ;;
    *)
      return 1
      ;;
  esac
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

suite="ui-resize-probes"
out_dir="target/fret-diag-resize-probes-gate-$(date +%s)"
baseline=""
launch_bin="target/release/fret-ui-gallery"
timeout_ms=300000
attempts=1
repeat=7
warmup_frames=5
default_suite_hooks=true
prewarm_scripts=()
prelude_scripts=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --suite)
      suite="$2"
      shift 2
      ;;
    --out-dir)
      out_dir="$2"
      shift 2
      ;;
    --baseline)
      baseline="$2"
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
    --attempts)
      attempts="$2"
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
    --prewarm-script)
      prewarm_scripts+=("$2")
      shift 2
      ;;
    --prelude-script)
      prelude_scripts+=("$2")
      shift 2
      ;;
    --no-default-suite-hooks)
      default_suite_hooks=false
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

require_cmd cargo
require_cmd jq

cd "$WORKSPACE_ROOT"
mkdir -p "$out_dir"

if [[ "$attempts" -lt 1 ]]; then
  echo "error: --attempts must be >= 1" >&2
  exit 2
fi

if [[ -z "$baseline" ]]; then
  platform_key="$(host_platform_key)"
  if ! baseline="$(default_baseline_for_suite "$suite" "$platform_key")"; then
    echo "error: no default baseline for --suite '$suite' on platform '$platform_key' (provide --baseline explicitly)" >&2
    exit 2
  fi
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

echo "[gate] ${suite} -> ${out_dir} (attempts=${attempts})"
echo "[gate] baseline: ${baseline}"
echo "[gate] launch-bin: ${launch_bin}"
printf '[gate] prewarm:'
printf ' %s' "${prewarm_scripts[@]}"
echo
printf '[gate] prelude:'
printf ' %s' "${prelude_scripts[@]}"
echo

passes=0
fails=0
selected_attempt_dir=""
attempt_summaries_json="[]"

for ((i=1; i<=attempts; i++)); do
  attempt_dir="$out_dir/attempt-$i"
  mkdir -p "$attempt_dir"

  cmd=(
    cargo run -q -p fretboard --
    diag perf "$suite"
    --dir "$attempt_dir"
    --timeout-ms "$timeout_ms"
  )
  for script in "${prewarm_scripts[@]}"; do
    cmd+=(--prewarm-script "$script")
  done
  for script in "${prelude_scripts[@]}"; do
    cmd+=(--prelude-script "$script")
  done
  cmd+=(
    --reuse-launch
    --repeat "$repeat" --warmup-frames "$warmup_frames"
    --sort time --top 15 --json
    --perf-baseline "$baseline"
    --env FRET_UI_GALLERY_VIEW_CACHE=1
    --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1
    --env FRET_DIAG_SCRIPT_AUTO_DUMP=0
    --env FRET_DIAG_SEMANTICS=0
    --launch -- "$launch_bin"
  )

  echo "[gate] attempt $i/$attempts -> $attempt_dir"
  printf '[gate] cmd: %q ' "${cmd[@]}"
  echo

  rc=0
  if "${cmd[@]}" > "$attempt_dir/stdout.json" 2> "$attempt_dir/stderr.log"; then
    rc=0
  else
    rc=$?
  fi

  check_file="$attempt_dir/check.perf_thresholds.json"
  failures_count=""
  attempt_pass=true

  if [[ "$rc" -ne 0 ]]; then
    attempt_pass=false
  fi

  if [[ ! -f "$check_file" ]]; then
    attempt_pass=false
  else
    failures_count="$(jq -r '.failures | length' "$check_file" 2>/dev/null || true)"
    if [[ -z "$failures_count" ]]; then
      attempt_pass=false
    elif [[ "$failures_count" != "0" ]]; then
      attempt_pass=false
    fi
  fi

  if [[ "$attempt_pass" == "true" ]]; then
    passes=$((passes + 1))
    if [[ -z "$selected_attempt_dir" ]]; then
      selected_attempt_dir="$attempt_dir"
    fi
  else
    fails=$((fails + 1))
  fi

  attempt_summaries_json="$(
    jq -n \
      --argjson prev "$attempt_summaries_json" \
      --arg attempt_dir "$attempt_dir" \
      --argjson pass "$attempt_pass" \
      --arg rc "$rc" \
      --arg check_file "$check_file" \
      --arg failures_count "${failures_count:-}" \
      --arg stdout "$attempt_dir/stdout.json" \
      --arg stderr "$attempt_dir/stderr.log" \
      '$prev + [{
        attempt_dir: $attempt_dir,
        pass: $pass,
        rc: ($rc | tonumber),
        check: {
          perf_thresholds: $check_file,
          failures: (if ($failures_count | length) == 0 then null else ($failures_count | tonumber) end)
        },
        stdout: $stdout,
        stderr: $stderr
      }]'
  )"
done

majority_required=$((attempts / 2 + 1))
pass=false
if [[ "$passes" -ge "$majority_required" ]]; then
  pass=true
fi

if [[ -z "$selected_attempt_dir" ]]; then
  selected_attempt_dir="$out_dir/attempt-$attempts"
fi

prewarm_scripts_json="[]"
if [[ "${#prewarm_scripts[@]}" -gt 0 ]]; then
  prewarm_scripts_json="$(printf '%s\n' "${prewarm_scripts[@]}" | jq -R . | jq -s .)"
fi
prelude_scripts_json="[]"
if [[ "${#prelude_scripts[@]}" -gt 0 ]]; then
  prelude_scripts_json="$(printf '%s\n' "${prelude_scripts[@]}" | jq -R . | jq -s .)"
fi

# Preserve compatibility with downstream tooling by copying one attempt to the top-level paths.
cp -f "$selected_attempt_dir/stdout.json" "$out_dir/stdout.json" || true
cp -f "$selected_attempt_dir/stderr.log" "$out_dir/stderr.log" || true
cp -f "$selected_attempt_dir/check.perf_thresholds.json" "$out_dir/check.perf_thresholds.json" || true

jq -n \
  --arg out_dir "$out_dir" \
  --arg suite "$suite" \
  --arg baseline "$baseline" \
  --arg launch_bin "$launch_bin" \
  --argjson prewarm_scripts "$prewarm_scripts_json" \
  --argjson prelude_scripts "$prelude_scripts_json" \
  --argjson default_suite_hooks "$default_suite_hooks" \
  --argjson pass "$pass" \
  --arg attempts "$attempts" \
  --arg passes "$passes" \
  --arg fails "$fails" \
  --arg majority_required "$majority_required" \
  --arg selected_attempt_dir "$selected_attempt_dir" \
  --arg repeat "$repeat" \
  --arg warmup_frames "$warmup_frames" \
  --arg stdout "$out_dir/stdout.json" \
  --arg stderr "$out_dir/stderr.log" \
  --arg check_file "$out_dir/check.perf_thresholds.json" \
  --argjson attempt_summaries "$attempt_summaries_json" \
  '{
    kind: "resize_probes_gate_summary",
    pass: $pass,
    out_dir: $out_dir,
    suite: $suite,
    baseline: $baseline,
    launch_bin: $launch_bin,
    suite_hooks: {
      prewarm: $prewarm_scripts,
      prelude: $prelude_scripts,
      default_suite_hooks: $default_suite_hooks
    },
    attempts: ($attempts | tonumber),
    pass_attempts: ($passes | tonumber),
    fail_attempts: ($fails | tonumber),
    majority_required: ($majority_required | tonumber),
    selected_attempt_dir: $selected_attempt_dir,
    repeat: ($repeat | tonumber),
    warmup_frames: ($warmup_frames | tonumber),
    check: {
      perf_thresholds: $check_file,
      failures: null
    },
    stdout: $stdout,
    stderr: $stderr,
    attempt_summaries: $attempt_summaries
  }' > "$out_dir/summary.json"

if [[ "$pass" != "true" ]]; then
  echo "[gate] FAIL (passes=$passes/$attempts; required=$majority_required). See: $out_dir/summary.json" >&2
  exit 1
fi

echo "[gate] PASS (passes=$passes/$attempts; required=$majority_required). Summary: $out_dir/summary.json"
