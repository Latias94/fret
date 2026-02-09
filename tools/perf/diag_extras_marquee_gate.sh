#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  tools/perf/diag_extras_marquee_gate.sh \
    --baseline <path> \
    [--out-dir <path>] \
    [--launch-bin <path>] \
    [--timeout-ms <n>] \
    [--repeat <n>] \
    [--warmup-frames <n>]

Notes:
  - Runs the `extras-marquee-steady` perf suite via `fretboard diag perf`.
  - Baselines are machine-dependent; generate one via:
      tools/perf/diag_perf_baseline_select.sh \
        --suite extras-marquee-steady \
        --preset docs/workstreams/perf-baselines/policies/extras-marquee-steady.v1.json \
        --baseline-out docs/workstreams/perf-baselines/extras-marquee-steady.<machine-tag>.v1.json \
        --candidates 2 --validate-runs 3 --repeat 7 --warmup-frames 5 \
        --work-dir target/fret-diag-baseline-select-extras-marquee-steady-v1 \
        --launch-bin target/release/extras_marquee_perf_demo
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

out_dir="target/fret-diag-perf/extras-marquee-steady.$(date +%s)"
baseline=""
launch_bin="target/release/extras_marquee_perf_demo"
timeout_ms=300000
repeat=7
warmup_frames=5

while [[ $# -gt 0 ]]; do
  case "$1" in
    --baseline)
      baseline="$2"
      shift 2
      ;;
    --out-dir)
      out_dir="$2"
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
    --repeat)
      repeat="$2"
      shift 2
      ;;
    --warmup-frames)
      warmup_frames="$2"
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

if [[ -z "$baseline" ]]; then
  echo "error: --baseline is required for this gate" >&2
  echo >&2
  usage >&2
  exit 2
fi

echo "[gate] extras-marquee-steady -> ${out_dir}"
echo "[gate] baseline: ${baseline}"
echo "[gate] launch-bin: ${launch_bin}"

# Ensure the demo binary exists (fast no-op if already built).
cargo build -q -p fret-demo --release --bin extras_marquee_perf_demo

cmd=(
  cargo run -q -p fretboard --
  diag perf extras-marquee-steady
  --dir "$out_dir"
  --timeout-ms "$timeout_ms"
  --reuse-launch
  --repeat "$repeat" --warmup-frames "$warmup_frames"
  --sort time --top 15 --json
  --perf-baseline "$baseline"
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0
  --env FRET_DIAG_SEMANTICS=0
  --launch -- "$launch_bin"
)

printf '[gate] cmd: %q ' "${cmd[@]}"
echo

rc=0
if ! "${cmd[@]}" > "$out_dir/stdout.json" 2> "$out_dir/stderr.log"; then
  rc=$?
fi

check_file="$out_dir/check.perf_thresholds.json"
if [[ "$rc" -ne 0 ]]; then
  echo "FAIL (rc=$rc). See: $out_dir/stderr.log" >&2
  exit "$rc"
fi

if [[ ! -f "$check_file" ]]; then
  echo "FAIL (missing $check_file). See: $out_dir/stderr.log" >&2
  exit 1
fi

failures_count="$(jq -r '.failures | length' "$check_file")"
if [[ "$failures_count" != "0" ]]; then
  echo "FAIL (perf threshold failures=$failures_count). See: $check_file" >&2
  exit 1
fi

echo "PASS (extras-marquee-steady)"

