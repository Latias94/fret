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

NOTE: The workspace now targets GameActivity for Android IME support. For a working
device loop (including soft keyboard), use:
  tools/mobile/android_game_activity_run.sh

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

if [[ -z "${ANDROID_SDK_ROOT:-}" ]]; then
  if [[ -n "${ANDROID_HOME:-}" ]]; then
    export ANDROID_SDK_ROOT="${ANDROID_HOME}"
  fi
fi

if [[ -z "${ANDROID_NDK_ROOT:-}" ]]; then
  if [[ -n "${ANDROID_HOME:-}" && -d "${ANDROID_HOME}/ndk" ]]; then
    ndk="$(ls -1 "${ANDROID_HOME}/ndk" 2>/dev/null | sort -V | tail -n 1 || true)"
    if [[ -n "${ndk}" && -d "${ANDROID_HOME}/ndk/${ndk}" ]]; then
      export ANDROID_NDK_ROOT="${ANDROID_HOME}/ndk/${ndk}"
    fi
  fi
fi

if [[ -z "${ANDROID_NDK_ROOT:-}" ]]; then
  echo "ANDROID_NDK_ROOT is not set and NDK was not auto-detected." >&2
  echo "Install an NDK and set ANDROID_NDK_ROOT (or ANDROID_HOME)." >&2
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
