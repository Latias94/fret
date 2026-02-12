#!/usr/bin/env bash
set -euo pipefail

package="fret-ui-gallery-mobile"
profile="debug"
device=""
no_logcat=""

usage() {
  cat <<EOF
Usage: $(basename "$0") [--package <name>] [--release] [--device <serial>] [--no-logcat]

Builds and runs an Android APK on a connected device/emulator using cargo-apk.

Defaults:
  --package ${package}
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --package|-p) package="$2"; shift 2 ;;
    --release) profile="release"; shift ;;
    --device|-d) device="$2"; shift 2 ;;
    --no-logcat) no_logcat="--no-logcat"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found." >&2
  exit 1
fi

if ! cargo apk --help >/dev/null 2>&1; then
  echo "cargo-apk is required. Install with: cargo install cargo-apk" >&2
  exit 1
fi

if ! command -v adb >/dev/null 2>&1; then
  echo "adb not found. Install Android platform-tools and ensure adb is on PATH." >&2
  exit 1
fi

release_flag=""
if [[ "${profile}" == "release" ]]; then
  release_flag="--release"
fi

device_flag=()
if [[ -n "${device}" ]]; then
  device_flag=(--device "${device}")
fi

set -x
cargo apk run "${device_flag[@]}" ${no_logcat} -p "${package}" ${release_flag}

