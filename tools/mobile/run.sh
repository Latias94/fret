#!/usr/bin/env bash
set -euo pipefail

platform="${1:-}"
shift || true

usage() {
  cat <<EOF
Usage:
  tools/mobile/run.sh android --app <name> [--device <serial>] [--release] [--no-logcat]
  tools/mobile/run.sh ios --app <name> [--sim] [--udid <sim-udid>] [--release]
  tools/mobile/run.sh doctor [android|ios|all]

Notes:
  - Android uses the Gradle GameActivity wrapper.
  - iOS currently supports the simulator loop.

Examples:
  tools/mobile/run.sh doctor
  tools/mobile/run.sh android --app ui-gallery -d <serial>
  tools/mobile/run.sh ios --app ui-gallery --sim
EOF
}

if [[ -z "${platform}" || "${platform}" == "-h" || "${platform}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ "${platform}" == "doctor" ]]; then
  exec tools/mobile/doctor.sh "${@:-all}"
fi

app=""
profile="debug"
device=""
no_logcat=""
udid=""
ios_sim=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --app) app="$2"; shift 2 ;;
    --release) profile="release"; shift ;;
    --device|-d) device="$2"; shift 2 ;;
    --no-logcat) no_logcat="--no-logcat"; shift ;;
    --udid) udid="$2"; shift 2 ;;
    --sim) ios_sim="1"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

if [[ -z "${app}" ]]; then
  echo "--app is required." >&2
  usage >&2
  exit 2
fi

case "${platform}" in
  android)
    if [[ "${app}" != "ui-gallery" ]]; then
      echo "Unknown Android app: ${app} (supported: ui-gallery)" >&2
      exit 2
    fi

    args=()
    if [[ "${profile}" == "release" ]]; then
      args+=(--release)
    fi
    if [[ -n "${device}" ]]; then
      args+=(--device "${device}")
    fi
    if [[ -n "${no_logcat}" ]]; then
      args+=(--no-logcat)
    fi

    exec tools/mobile/android_game_activity_run.sh "${args[@]}"
    ;;
  ios)
    if [[ "${app}" != "ui-gallery" ]]; then
      echo "Unknown iOS app: ${app} (supported: ui-gallery)" >&2
      exit 2
    fi
    if [[ -z "${ios_sim}" ]]; then
      echo "Only the simulator loop is implemented right now. Pass --sim." >&2
      exit 2
    fi

    args=()
    if [[ "${profile}" == "release" ]]; then
      args+=(--release)
    fi
    if [[ -n "${udid}" ]]; then
      args+=(--udid "${udid}")
    fi

    exec tools/mobile/ios_sim_run.sh "${args[@]}"
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac

