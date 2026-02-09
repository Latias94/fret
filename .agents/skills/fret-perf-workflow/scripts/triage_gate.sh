#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  .agents/skills/fret-perf-workflow/scripts/triage_gate.sh <out-dir>

Prints a compact triage summary for perf gate output produced by:
  tools/perf/diag_resize_probes_gate.sh

It reports:
  - which attempts passed/failed,
  - which script/metric exceeded thresholds,
  - and the worst bundle for each failing script (by top_total_time_us).

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

require_cmd jq
require_cmd awk

summary_json="$out_dir/summary.json"
if [[ ! -f "$summary_json" ]]; then
  echo "error: missing summary: $summary_json" >&2
  exit 2
fi

echo "== gate summary =="
jq -r '"pass=\(.pass) suite=\(.suite) attempts=\(.attempts) pass_attempts=\(.pass_attempts) fail_attempts=\(.fail_attempts) selected=\(.selected_attempt_dir)"' "$summary_json"

echo
echo "== attempts =="
jq -r '.attempt_summaries[] | "\(.attempt_dir)\tpass=\(.pass)\tfailures=\(.check.failures // "missing")\trc=\(.rc)"' "$summary_json"

attempt_dirs="$(jq -r '.attempt_summaries[].attempt_dir' "$summary_json")"

echo
for attempt_dir in $attempt_dirs; do
  check_json="$attempt_dir/check.perf_thresholds.json"
  if [[ ! -f "$check_json" ]]; then
    continue
  fi

  failures_len="$(jq -r '.failures | length' "$check_json" 2>/dev/null || echo "")"
  if [[ -z "$failures_len" || "$failures_len" == "0" ]]; then
    continue
  fi

  echo "== FAIL: $attempt_dir ($failures_len threshold(s)) =="
  jq -r '.failures[] | "- \(.script) :: \(.metric) actual=\(.actual_us)us threshold=\(.threshold_us)us"' "$check_json"

  stdout_path="$attempt_dir/stdout.json"
  if [[ ! -f "$stdout_path" ]]; then
    echo "- note: missing stdout.json (cannot resolve worst bundles)"
    echo
    continue
  fi

  # Extract the JSON payload (skip leading log lines).
  payload="$(
    awk 'BEGIN{f=0} {if(!f && $0 ~ /^\\{/){f=1} if(f){print}}' "$stdout_path"
  )"

  if [[ -z "$payload" ]]; then
    echo "- note: empty JSON payload (cannot resolve worst bundles)"
    echo
    continue
  fi

  # For each failing script, resolve the bundle of the worst run by total time.
  scripts="$(
    jq -r '.failures[].script' "$check_json" | awk '!seen[$0]++'
  )"
  for script in $scripts; do
    script_name="$(basename "$script")"
    worst_bundle="$(
      jq -r --arg script_name "$script_name" '
        .rows[]
        | select(.script | endswith($script_name))
        | (.runs | max_by(.top_total_time_us) | .bundle)
      ' <<<"$payload" 2>/dev/null || true
    )"
    worst_total="$(
      jq -r --arg script_name "$script_name" '
        .rows[]
        | select(.script | endswith($script_name))
        | (.runs | max_by(.top_total_time_us) | .top_total_time_us)
      ' <<<"$payload" 2>/dev/null || true
    )"
    if [[ -n "$worst_bundle" && "$worst_bundle" != "null" ]]; then
      echo "- worst bundle ($script_name): $worst_bundle (top_total_time_us=$worst_total)"
    else
      echo "- note: could not resolve worst bundle for $script_name"
    fi
  done
  echo
done

