#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  tools/perf/diag_resize_probes_gate.sh \
    [--out-dir <path>] \
    [--baseline <path>] \
    [--launch-bin <path>] \
    [--timeout-ms <n>] \
    [--repeat <n>] \
    [--warmup-frames <n>]

Notes:
  - Runs the `ui-resize-probes` perf suite via `fretboard diag perf`.
  - Intended to be a lightweight "P0 resize must not regress" gate.
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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

out_dir="target/fret-diag-resize-probes-gate-$(date +%s)"
baseline="docs/workstreams/perf-baselines/ui-resize-probes.macos-m4.v1.json"
launch_bin="target/release/fret-ui-gallery"
timeout_ms=300000
repeat=7
warmup_frames=5

while [[ $# -gt 0 ]]; do
  case "$1" in
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

  cmd=(
  cargo run -q -p fretboard --
  diag perf ui-resize-probes
  --dir "$out_dir"
  --timeout-ms "$timeout_ms"
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

echo "[gate] ui-resize-probes -> ${out_dir}"
echo "[gate] baseline: ${baseline}"
echo "[gate] launch-bin: ${launch_bin}"
printf '[gate] cmd: %q ' "${cmd[@]}"
echo

rc=0
if ! "${cmd[@]}" > "$out_dir/stdout.json" 2> "$out_dir/stderr.log"; then
  rc=$?
fi

pass=true
if [[ "$rc" -ne 0 ]]; then
  pass=false
fi

jq -n \
  --arg out_dir "$out_dir" \
  --arg baseline "$baseline" \
  --arg launch_bin "$launch_bin" \
  --argjson pass "$pass" \
  --arg stdout "$out_dir/stdout.json" \
  --arg stderr "$out_dir/stderr.log" \
  --arg repeat "$repeat" \
  --arg warmup_frames "$warmup_frames" \
  '{
    kind: "resize_probes_gate_summary",
    pass: $pass,
    out_dir: $out_dir,
    baseline: $baseline,
    launch_bin: $launch_bin,
    repeat: ($repeat | tonumber),
    warmup_frames: ($warmup_frames | tonumber),
    stdout: $stdout,
    stderr: $stderr
  }' > "$out_dir/summary.json"

if [[ "$pass" != "true" ]]; then
  echo "[gate] FAIL (rc=$rc). See: $out_dir/summary.json" >&2
  exit "$rc"
fi

echo "[gate] PASS. Summary: $out_dir/summary.json"
