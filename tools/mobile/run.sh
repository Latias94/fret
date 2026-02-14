#!/usr/bin/env bash
set -euo pipefail

platform="${1:-}"
shift || true

usage() {
  cat <<EOF
Usage:
  tools/mobile/run.sh android --app <name> [--device <serial>] [--backend <vk|gl|...>] [--allow-fallback] [--diag] [--diag-dir <path>] [--release] [--no-logcat]
  tools/mobile/run.sh ios --app <name> [--sim] [--udid <sim-udid>] [--device <udid>] [--team <team-id>] [--release]
  tools/mobile/run.sh doctor [android|ios|all]

Notes:
  - Android uses the Gradle GameActivity wrapper.
  - iOS supports simulator and real devices (real devices require codesigning).

Examples:
  tools/mobile/run.sh doctor
  tools/mobile/run.sh android --app ui-gallery -d <serial>
  tools/mobile/run.sh ios --app ui-gallery --sim
  IOS_TEAM_ID=<team> tools/mobile/run.sh ios --app ui-gallery --device <udid>
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
backend=""
allow_fallback=""
diag=""
diag_dir=""
udid=""
ios_sim=""
team=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --app) app="$2"; shift 2 ;;
    --release) profile="release"; shift ;;
    --device|-d) device="$2"; shift 2 ;;
    --backend) backend="$2"; shift 2 ;;
    --allow-fallback) allow_fallback="--allow-fallback"; shift ;;
    --diag) diag="--diag"; shift ;;
    --diag-dir) diag_dir="$2"; shift 2 ;;
    --no-logcat) no_logcat="--no-logcat"; shift ;;
    --udid) udid="$2"; shift 2 ;;
    --sim) ios_sim="1"; shift ;;
    --team|-t) team="$2"; shift 2 ;;
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

    cmd=(tools/mobile/android_game_activity_run.sh)
    if [[ "${profile}" == "release" ]]; then
      cmd+=(--release)
    fi
    if [[ -n "${device}" ]]; then
      cmd+=(--device "${device}")
    fi
    if [[ -n "${backend}" ]]; then
      cmd+=(--backend "${backend}")
    fi
    if [[ -n "${allow_fallback}" ]]; then
      cmd+=(--allow-fallback)
    fi
    if [[ -n "${diag}" ]]; then
      cmd+=(--diag)
    fi
    if [[ -n "${diag_dir}" ]]; then
      cmd+=(--diag-dir "${diag_dir}")
    fi
    if [[ -n "${no_logcat}" ]]; then
      cmd+=(--no-logcat)
    fi

    exec "${cmd[@]}"
    ;;
  ios)
    if [[ "${app}" != "ui-gallery" ]]; then
      echo "Unknown iOS app: ${app} (supported: ui-gallery)" >&2
      exit 2
    fi
    if [[ -n "${ios_sim}" ]]; then
      cmd=(tools/mobile/ios_sim_run.sh)
      if [[ "${profile}" == "release" ]]; then
        cmd+=(--release)
      fi
      if [[ -n "${udid}" ]]; then
        cmd+=(--udid "${udid}")
      fi
      exec "${cmd[@]}"
    fi

    cmd=(tools/mobile/ios_device_run.sh)
    if [[ "${profile}" == "release" ]]; then
      cmd+=(--release)
    fi
    if [[ -n "${device}" ]]; then
      cmd+=(--device "${device}")
    fi
    if [[ -n "${team}" ]]; then
      cmd+=(--team "${team}")
    fi
    exec "${cmd[@]}"
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
