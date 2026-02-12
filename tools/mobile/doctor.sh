#!/usr/bin/env bash
set -euo pipefail

platform="all"

usage() {
  cat <<EOF
Usage: $(basename "$0") [android|ios|all]

Prints a best-effort checklist for the local mobile toolchain.

Examples:
  tools/mobile/doctor.sh
  tools/mobile/doctor.sh android
  tools/mobile/doctor.sh ios
EOF
}

if [[ $# -gt 1 ]]; then
  usage >&2
  exit 2
fi

if [[ $# -eq 1 ]]; then
  platform="$1"
fi

ok() { printf "ok   %s\n" "$1"; }
warn() { printf "warn %s\n" "$1"; }
fail() { printf "fail %s\n" "$1"; }

have() { command -v "$1" >/dev/null 2>&1; }

check_cmd() {
  local name="$1"
  local hint="$2"
  if have "${name}"; then
    ok "${name} ($(command -v "${name}"))"
  else
    fail "${name} (missing) — ${hint}"
  fi
}

print_android() {
  echo "== Android =="
  check_cmd cargo "install Rust toolchain"
  check_cmd adb "install Android platform-tools"
  check_cmd java "install JDK (17 recommended for the Gradle wrapper)"

  if have cargo; then
    if cargo ndk --help >/dev/null 2>&1; then
      ok "cargo-ndk (available via cargo subcommand)"
    else
      fail "cargo-ndk (missing) — install with: cargo install cargo-ndk"
    fi
  fi

  if [[ -z "${ANDROID_SDK_ROOT:-}" && -n "${ANDROID_HOME:-}" ]]; then
    warn "ANDROID_SDK_ROOT is not set (ANDROID_HOME is set; scripts may still work)"
  elif [[ -z "${ANDROID_SDK_ROOT:-}" ]]; then
    fail "ANDROID_SDK_ROOT is not set (or ANDROID_HOME)"
  else
    ok "ANDROID_SDK_ROOT=${ANDROID_SDK_ROOT}"
  fi

  local ndk_root="${ANDROID_NDK_ROOT:-}"
  if [[ -z "${ndk_root}" && -n "${ANDROID_SDK_ROOT:-}" && -d "${ANDROID_SDK_ROOT}/ndk" ]]; then
    ndk_root="${ANDROID_SDK_ROOT}/ndk/$(ls -1 "${ANDROID_SDK_ROOT}/ndk" 2>/dev/null | sort -V | tail -n 1 || true)"
  fi
  if [[ -n "${ndk_root}" && -d "${ndk_root}" ]]; then
    ok "NDK=${ndk_root}"
  else
    warn "NDK not detected (ANDROID_NDK_ROOT unset). The run script will try SDK auto-detect."
  fi
}

print_ios() {
  echo "== iOS =="
  check_cmd cargo "install Rust toolchain"
  check_cmd xcrun "install Xcode or Xcode Command Line Tools"
  check_cmd python3 "required by tools/mobile/ios_sim_run.sh to pick a simulator"

  if have rustup; then
    local target="aarch64-apple-ios-sim"
    if rustup target list --installed 2>/dev/null | rg -q "^${target}$"; then
      ok "rust target ${target}"
    else
      warn "rust target ${target} not installed — run: rustup target add ${target}"
    fi
  else
    warn "rustup not found (cannot check installed iOS targets)"
  fi
}

case "${platform}" in
  all)
    print_android
    echo
    print_ios
    ;;
  android)
    print_android
    ;;
  ios)
    print_ios
    ;;
  -h|--help)
    usage
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac

