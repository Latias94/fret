#!/usr/bin/env bash
set -euo pipefail

profile="debug"
device_udid=""
team_id="${IOS_TEAM_ID:-}"

usage() {
  cat <<EOF
Usage: $(basename "$0") [--release] [--device <udid>] [--team <team-id>]

Builds the Rust static library for iOS (device), copies it into the Xcode wrapper,
then builds the wrapper app with xcodebuild.

If --device is provided, this script will try to install + launch using 'xcrun devicectl'
when available.

Codesigning:
  - For real device install/launch, set IOS_TEAM_ID or pass --team.
  - If no team is provided and --device is omitted, this script builds with codesigning disabled.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --release) profile="release"; shift ;;
    --device|-d) device_udid="$2"; shift 2 ;;
    --team|-t) team_id="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

if ! command -v xcodebuild >/dev/null 2>&1; then
  echo "xcodebuild not found. Install Xcode." >&2
  exit 1
fi

if ! command -v xcrun >/dev/null 2>&1; then
  echo "xcrun not found. Install Xcode or Xcode Command Line Tools." >&2
  exit 1
fi

target="aarch64-apple-ios"
release_flag=""
config="Debug"
if [[ "${profile}" == "release" ]]; then
  release_flag="--release"
  config="Release"
fi

echo "[ios-device] target=${target} profile=${profile}"

cargo build -p fret-ui-gallery-mobile --target "${target}" ${release_flag}

lib_path="target/${target}/${profile}/libfret_ui_gallery_mobile.a"
if [[ ! -f "${lib_path}" ]]; then
  echo "Built staticlib not found at ${lib_path}" >&2
  exit 1
fi

wrapper_dir="apps/fret-ui-gallery-mobile/ios"
rust_libs_dir="${wrapper_dir}/RustLibs"
mkdir -p "${rust_libs_dir}"
cp -f "${lib_path}" "${rust_libs_dir}/"

project="${wrapper_dir}/FretUIGalleryMobile.xcodeproj"
scheme="FretUIGalleryMobile"
derived_data="target/ios-device-derived-data"

if [[ -n "${device_udid}" && -z "${team_id}" ]]; then
  echo "[ios-device] --device requires codesigning. Set IOS_TEAM_ID or pass --team." >&2
  exit 2
fi

xcodebuild_settings=()
if [[ -n "${team_id}" ]]; then
  xcodebuild_settings+=("DEVELOPMENT_TEAM=${team_id}")
else
  echo "[ios-device] no team provided; building with CODE_SIGNING_ALLOWED=NO (no install/launch)" >&2
  xcodebuild_settings+=("CODE_SIGNING_ALLOWED=NO" "CODE_SIGNING_REQUIRED=NO" "CODE_SIGN_IDENTITY=")
fi

set -x
xcodebuild \
  -project "${project}" \
  -scheme "${scheme}" \
  -configuration "${config}" \
  -sdk iphoneos \
  -derivedDataPath "${derived_data}" \
  "${xcodebuild_settings[@]}" \
  build
set +x

app_path="${derived_data}/Build/Products/${config}-iphoneos/${scheme}.app"
if [[ ! -d "${app_path}" ]]; then
  echo "Built .app not found at ${app_path}" >&2
  exit 1
fi

echo "[ios-device] built app: ${app_path}"

if [[ -z "${device_udid}" ]]; then
  echo "[ios-device] no --device provided; skipping install/launch"
  exit 0
fi

if ! xcrun devicectl --help >/dev/null 2>&1; then
  echo "[ios-device] devicectl is not available. Open the Xcode project and Run on device:" >&2
  echo "  ${project}" >&2
  exit 2
fi

bundle_id="dev.fret.ui-gallery-mobile"

set -x
xcrun devicectl device install app --device "${device_udid}" "${app_path}"
xcrun devicectl device process launch --device "${device_udid}" "${bundle_id}" || true
