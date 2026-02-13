---
title: Mobile GFX Backend Selection v1 - M3 iOS Simulator Smoke
status: draft
date: 2026-02-13
platform: iOS Simulator
---

# M3 iOS Simulator smoke

Goal: capture a repeatable iOS Simulator run that proves backend selection is observable and that
the app can render frames without crashing during init.

This is **supporting evidence** for the Android-first M3 gate (real device remains the primary
acceptance signal).

## Prerequisites

- Xcode + iOS Simulator installed and usable (first-run prompts completed).
- A booted simulator device.

## Build + install

Build, bundle, install, and launch the iOS Simulator app:

```bash
tools/mobile/run.sh ios --app ui-gallery --sim
```

## Enable diagnostics (filesystem transport)

Diagnostics output is written inside the app sandbox. Prefer setting an explicit `FRET_DIAG_DIR`
via `SIMCTL_CHILD_FRET_DIAG_DIR` so the output location is stable even if `HOME` is not populated
as expected in the simulator process.

1) Pick a booted simulator UDID:

```bash
xcrun simctl list devices booted
```

2) Launch with diagnostics enabled:

```bash
container="$(xcrun simctl get_app_container <udid> dev.fret.ui-gallery data)"
diag_dir="${container}/tmp/fret-diag"

SIMCTL_CHILD_FRET_DIAG=1 \
SIMCTL_CHILD_FRET_DIAG_DIR="${diag_dir}" \
xcrun simctl launch --terminate-running-process <udid> dev.fret.ui-gallery
```

3) Locate the sandbox on the host and inspect the diagnostics directory:

```bash
ls -la "${container}/tmp/fret-diag"
```

Expected files (at minimum):

- `ready.touch`
- `capabilities.json`

## Run the keyboard-avoidance diag script (optional but recommended)

This validates that scripted `WindowMetricsSetInsets` (safe-area + occlusion) can drive a stable
gate on iOS Simulator, including screenshot capture.

1) Enable diagnostics + screenshots:

```bash
SIMCTL_CHILD_FRET_DIAG=1 \
SIMCTL_CHILD_FRET_DIAG_DIR="${diag_dir}" \
SIMCTL_CHILD_FRET_DIAG_SCREENSHOTS=1 \
xcrun simctl launch --terminate-running-process <udid> dev.fret.ui-gallery
```

2) Copy the script into the diagnostics directory:

```bash
cp -f tools/diag-scripts/ui-gallery-window-insets-safe-area-and-keyboard-avoidance.json "${diag_dir}/script.json"
```

3) Bump the script trigger stamp (note: the first observed value is treated as a baseline, so bump twice):

```bash
echo "$(date +%s%3N)" > "${diag_dir}/script.touch"
sleep 1
echo "$(date +%s%3N)" > "${diag_dir}/script.touch"
```

4) Inspect the result:

```bash
jq '{stage, reason_code, reason, step_index, step_name}' "${diag_dir}/script.result.json"
```

On success, expect:

- `${diag_dir}/<ts>-ui-gallery-safe-area-and-keyboard-avoidance/bundle.json`
- `${diag_dir}/screenshots/<ts>-ui-gallery-safe-area-and-keyboard-avoidance/*.png`

## Capture evidence

Recommended minimal evidence anchors:

1) Backend selection log excerpt (from `--console` or `--console-pty`):

```bash
SIMCTL_CHILD_FRET_DIAG=1 xcrun simctl launch --console --terminate-running-process <udid> dev.fret.ui-gallery
```

2) Diagnostics “ready” proof:

- `${container}/tmp/fret-diag/ready.touch` exists and contains a timestamp.

## Notes

- It is expected for the iOS Simulator GPU to be reported as downlevel (not fully WebGPU
  compliant). This should not be treated as a regression by itself as long as the renderer’s
  required downlevel flags are present.
