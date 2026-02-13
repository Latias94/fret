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

Diagnostics output is written inside the app sandbox under `tmp/fret-diag`.

1) Pick a booted simulator UDID:

```bash
xcrun simctl list devices booted
```

2) Launch with diagnostics enabled:

```bash
SIMCTL_CHILD_FRET_DIAG=1 xcrun simctl launch --terminate-running-process <udid> dev.fret.ui-gallery
```

3) Locate the sandbox on the host and inspect the diagnostics directory:

```bash
container="$(xcrun simctl get_app_container <udid> dev.fret.ui-gallery data)"
ls -la "${container}/tmp/fret-diag"
```

Expected files (at minimum):

- `ready.touch`
- `capabilities.json`

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

