---
title: Mobile GFX Backend Selection v1 - M3 Real Device Smoke (OPPO PLG110)
status: draft
date: 2026-02-12
scope: Android real-device evidence (wgpu backend selection + diagnostics bundle)
---

# M3 real-device smoke (OPPO PLG110)

This document records a minimal, repeatable **real-device** smoke recipe for the Android mobile
graphics path (preferred backend: Vulkan by default), plus the evidence anchors needed to accept
ADR 0268.

Workstream:

- `docs/workstreams/mobile-gfx-backend-v1/design.md`

Contract anchor:

- `docs/adr/0268-mobile-graphics-backend-selection-and-downlevel-policy-v1.md`

## Device under test

- Vendor/model: OPPO `PLG110` (Find X9)
- GPU class: Mali (Dimensity 9500)
- Transport: `adb` (USB or wireless debugging)

## Recipe (minimal)

1) Confirm the device is reachable:

```bash
adb devices -l
```

If using wireless debugging, the device serial is often an mDNS-derived name like:

- `adb-<serial>-<suffix>._adb-tls-connect._tcp`

If `adb devices` shows nothing but the phone is on the same LAN, you can also discover the current
service endpoint via:

```bash
adb mdns services
```

2) Run the mobile UI gallery with diagnostics enabled:

```bash
tools/mobile/run.sh android --app ui-gallery -d <serial> --diag --no-logcat
```

Notes:

- `--diag` sets `FRET_DIAG=1` (and defaults `FRET_DIAG_DIR` to `files/fret-diag` on Android).
- `--no-logcat` keeps the terminal quiet; use logcat manually if triaging crashes.
- If `adb install` fails with `device offline`, re-enable Wireless debugging on the phone (or use
  a USB transport) and re-run step (2).

3) Trigger a diagnostics bundle dump (filesystem transport):

```bash
adb -s <serial> shell "echo 1 | run-as dev.fret.ui_gallery tee files/fret-diag/trigger.touch >/dev/null"
```

4) Resolve the latest bundle directory name:

```bash
adb -s <serial> shell run-as dev.fret.ui_gallery cat files/fret-diag/latest.txt
```

5) Pull `bundle.json` locally:

```bash
ts="$(adb -s <serial> shell run-as dev.fret.ui_gallery cat files/fret-diag/latest.txt | tr -d '\r')"
mkdir -p target/fret-diag-mobile
adb -s <serial> exec-out run-as dev.fret.ui_gallery cat "files/fret-diag/${ts}/bundle.json" > "target/fret-diag-mobile/${ts}.bundle.json"
```

6) Extract the wgpu adapter snapshot (selected backend + driver + attempt history):

```bash
jq '.windows[0].snapshots[-1].wgpu_adapter | {
  requested_backend,
  requested_backend_is_override,
  allow_fallback,
  required_downlevel_flags,
  selected_backend,
  adapter_name,
  driver,
  driver_info,
  vendor,
  device,
  is_webgpu_compliant,
  downlevel_flags,
  init_attempts
}' "target/fret-diag-mobile/${ts}.bundle.json"
```

## Evidence (bundle anchors)

Paste the extracted JSON here once captured on-device.

Expected properties for M3 acceptance (default policy, no explicit override):

- `selected_backend == "Vulkan"`
- `requested_backend_is_override == false` (unless explicitly overridden)
- `is_webgpu_compliant == true` (preferred on real devices)
- `downlevel_flags` contains `VERTEX_STORAGE`
- `init_attempts` contains exactly one successful attempt by default (fail-fast posture)

Evidence (example):

```json
{
  "selected_backend": "Vulkan",
  "adapter_name": "<fill>",
  "driver": "<fill>",
  "driver_info": "<fill>",
  "is_webgpu_compliant": true,
  "downlevel_flags": "DownlevelFlags(VERTEX_STORAGE, ...)",
  "init_attempts": [
    { "backends": "vulkan", "ok": true, "selected_backend": "Vulkan", "adapter_name": "<fill>" }
  ]
}
```
