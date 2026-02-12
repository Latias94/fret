#!/usr/bin/env bash
set -euo pipefail

profile="debug"
device=""
no_logcat=""

usage() {
  cat <<EOF
Usage: $(basename "$0") [--release] [--device <serial>] [--no-logcat]

Builds and runs the Android APK using the Gradle GameActivity wrapper:
  apps/fret-ui-gallery-mobile/android

This path is required for reliable IME (soft keyboard) support.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --release) profile="release"; shift ;;
    --device|-d) device="$2"; shift 2 ;;
    --no-logcat) no_logcat="1"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found." >&2
  exit 1
fi

if ! cargo ndk --help >/dev/null 2>&1; then
  echo "cargo-ndk is required. Install with: cargo install cargo-ndk" >&2
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

if [[ -z "${ANDROID_SDK_ROOT:-}" ]]; then
  echo "ANDROID_SDK_ROOT (or ANDROID_HOME) must be set." >&2
  exit 1
fi

if [[ -z "${ANDROID_NDK_ROOT:-}" ]]; then
  if [[ -d "${ANDROID_SDK_ROOT}/ndk" ]]; then
    ndk="$(ls -1 "${ANDROID_SDK_ROOT}/ndk" 2>/dev/null | sort -V | tail -n 1 || true)"
    if [[ -n "${ndk}" && -d "${ANDROID_SDK_ROOT}/ndk/${ndk}" ]]; then
      export ANDROID_NDK_ROOT="${ANDROID_SDK_ROOT}/ndk/${ndk}"
    fi
  fi
fi

if [[ -n "${ANDROID_NDK_ROOT:-}" && -z "${ANDROID_NDK_HOME:-}" ]]; then
  export ANDROID_NDK_HOME="${ANDROID_NDK_ROOT}"
fi

gradle_dir="apps/fret-ui-gallery-mobile/android"
if [[ ! -d "${gradle_dir}" ]]; then
  echo "Gradle project not found: ${gradle_dir}" >&2
  exit 1
fi

# Create local.properties to point Gradle at the SDK, if missing.
local_props="${gradle_dir}/local.properties"
if [[ ! -f "${local_props}" ]]; then
  sdk_dir="${ANDROID_SDK_ROOT//\\/\\\\}"
  sdk_dir="${sdk_dir//:/\\:}"
  printf "sdk.dir=%s\n" "${sdk_dir}" > "${local_props}"
fi

adb_args=()
if [[ -n "${device}" ]]; then
  adb_args=(-s "${device}")
fi

gradle_task="installDebug"
if [[ "${profile}" == "release" ]]; then
  gradle_task="assembleRelease"
fi

set -x
(cd "${gradle_dir}" && ./gradlew ":app:${gradle_task}")

if [[ "${profile}" == "debug" ]]; then
  adb "${adb_args[@]}" shell am start -n "dev.fret.ui_gallery/dev.fret.ui_gallery.MainActivity" >/dev/null
fi

if [[ -z "${no_logcat}" ]]; then
  adb "${adb_args[@]}" logcat -s "fret:*" "*:S"
fi

