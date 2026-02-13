#!/usr/bin/env bash
set -euo pipefail

package="fret-ui-gallery"
bin="fret-ui-gallery"
profile="debug"
target="aarch64-apple-ios-sim"
bundle_id="dev.fret.ui-gallery"
app_name="FretUIGallery"

usage() {
  cat <<EOF
Usage: $(basename "$0") [--package <name>] [--bin <name>] [--release] [--bundle-id <id>] [--app-name <name>] [--udid <sim-udid>]

Builds a Rust iOS simulator binary, bundles it into a minimal .app, installs it to an iOS Simulator, and launches it.

Defaults:
  --package   ${package}
  --bin       ${bin}
  --bundle-id ${bundle_id}
  --app-name  ${app_name}
EOF
}

udid="${IOS_SIM_UDID:-}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --package|-p) package="$2"; shift 2 ;;
    --bin|-b) bin="$2"; shift 2 ;;
    --bundle-id) bundle_id="$2"; shift 2 ;;
    --app-name) app_name="$2"; shift 2 ;;
    --udid) udid="$2"; shift 2 ;;
    --release) profile="release"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

if ! command -v xcrun >/dev/null 2>&1; then
  echo "xcrun not found. Install Xcode or Xcode Command Line Tools." >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 not found (required to pick a simulator device)." >&2
  exit 1
fi

if [[ -z "${udid}" ]]; then
  udid="$(python3 - <<'PY'
import json, subprocess, sys

def simctl_json(*args):
    return json.loads(subprocess.check_output(["xcrun", "simctl", *args, "-j"], text=True))

data = simctl_json("list", "devices")
devices_by_runtime = data.get("devices", {})

booted = []
available = []

for runtime, devices in devices_by_runtime.items():
    for d in devices:
        if not d.get("isAvailable", False):
            continue
        if d.get("state") == "Booted":
            booted.append((runtime, d))
        available.append((runtime, d))

def runtime_key(runtime):
    # Prefer iOS runtimes, then higher versions.
    # Example: com.apple.CoreSimulator.SimRuntime.iOS-17-2
    if ".iOS-" in runtime:
        prefix, ver = runtime.split(".iOS-", 1)
        parts = tuple(int(p) for p in ver.split("-") if p.isdigit())
        return (1, parts)
    return (0, ())

def pick(candidates):
    candidates = sorted(candidates, key=lambda t: runtime_key(t[0]), reverse=True)
    # Prefer "iPhone 15" if available.
    for runtime, d in candidates:
        if d.get("name") == "iPhone 15":
            return d.get("udid")
    for runtime, d in candidates:
        name = d.get("name", "")
        if name.startswith("iPhone"):
            return d.get("udid")
    return candidates[0][1].get("udid") if candidates else None

udid = pick(booted) or pick(available)
if not udid:
    print("No available iOS Simulator devices found.", file=sys.stderr)
    sys.exit(1)
print(udid)
PY
)"
fi

echo "[ios-sim] target=${target} profile=${profile} package=${package} bin=${bin}"
echo "[ios-sim] udid=${udid}"

release_flag=""
if [[ "${profile}" == "release" ]]; then
  release_flag="--release"
fi

cargo build -p "${package}" --bin "${bin}" --target "${target}" ${release_flag}

bin_path="target/${target}/${profile}/${bin}"
if [[ ! -f "${bin_path}" ]]; then
  echo "Built binary not found at ${bin_path}" >&2
  exit 1
fi

app_dir="target/${target}/${profile}/${app_name}.app"
mkdir -p "${app_dir}"

cp -f "${bin_path}" "${app_dir}/${bin}"
chmod +x "${app_dir}/${bin}"

cat > "${app_dir}/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>${bin}</string>
  <key>CFBundleIdentifier</key>
  <string>${bundle_id}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>${app_name}</string>
  <key>CFBundleDisplayName</key>
  <string>${app_name}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSRequiresIPhoneOS</key>
  <true/>
  <key>UIRequiresFullScreen</key>
  <true/>
  <key>UISupportedInterfaceOrientations</key>
  <array>
    <string>UIInterfaceOrientationPortrait</string>
    <string>UIInterfaceOrientationLandscapeLeft</string>
    <string>UIInterfaceOrientationLandscapeRight</string>
  </array>
  <key>UILaunchStoryboardName</key>
  <string></string>
</dict>
</plist>
EOF

if command -v codesign >/dev/null 2>&1; then
  codesign --force --sign - --timestamp=none "${app_dir}" >/dev/null 2>&1 || true
fi

echo "[ios-sim] booting simulator (if needed)…"
xcrun simctl boot "${udid}" >/dev/null 2>&1 || true
xcrun simctl bootstatus "${udid}" -b >/dev/null 2>&1 || true

echo "[ios-sim] installing ${bundle_id}…"
xcrun simctl uninstall "${udid}" "${bundle_id}" >/dev/null 2>&1 || true
xcrun simctl install "${udid}" "${app_dir}"

echo "[ios-sim] launching ${bundle_id}…"
xcrun simctl launch "${udid}" "${bundle_id}"
