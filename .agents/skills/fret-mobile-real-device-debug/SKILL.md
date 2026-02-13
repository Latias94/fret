---
name: fret-mobile-real-device-debug
description: "Real-device mobile debugging workflow for Fret (Android + iOS): run the smallest mobile target, verify Vulkan/Metal constraints, capture diagnostics bundles, and archive evidence for ADRs/workstreams."
---

# Fret mobile real-device debug (Android + iOS)

## When to use

- You need **real-device evidence** for a mobile contract/ADR (gfx backend selection, insets, IME, lifecycle).
- Android Emulator is unstable or misleading (e.g. Vulkan/gfxstream crashes, missing required downlevel flags).
- You are validating “hard-to-change” mobile surfaces (IME composing + cursor rect, keyboard occlusion, safe-area).

## Quick start

Android (real device recommended):

- `adb devices -l`
- `tools/mobile/run.sh android --app ui-gallery -d <serial> --diag --no-logcat`

iOS (simulator recommended on macOS for fast iteration):

- `tools/mobile/run.sh ios --app ui-gallery --sim`

## Workflow

1) Pick the smallest runnable target

- Prefer `ui-gallery` first (it exercises text/overlays/insets early).
- Android wrapper entrypoint: `tools/mobile/run.sh`

2) Verify the emulator vs real-device decision

Practical default:

- If you are validating **Vulkan-first** posture and downlevel requirements, use a **real Android device**.
- If you want a fast “mobile-ish” UI loop on macOS, prefer **iOS Simulator** (Metal is the first-class path).

Notes:

- Android Emulator can expose Vulkan features but still be unstable for real-world wgpu usage (driver/translation
  layers differ from physical devices).
- Host OS matters (macOS → Metal-based virtualization; Windows → typically DX12-based virtualization). Treat this
  as “worth trying”, not as a correctness baseline: always accept/reject workstreams using **bundle evidence**
  captured on at least one real device.

3) Understand Fret’s current Vulkan constraints (what we require)

Fret currently requires:

- `DownlevelFlags::VERTEX_STORAGE` (renderer uses storage buffers in vertex shaders).
- Backend defaults: Android → Vulkan, iOS → Metal (unless overridden).

If the selected backend cannot satisfy required downlevel flags, the current posture is to fail fast rather than
silently degrade (unless you explicitly opt into fallback in debug builds).

4) Run the app with diagnostics enabled

Android (GameActivity wrapper):

- `tools/mobile/run.sh android --app ui-gallery -d <serial> --diag --no-logcat`
- Optional override:
  - `--backend vk` (force Vulkan)
  - `--backend gl` (force GL; expected to fail on many emulators due to missing `VERTEX_STORAGE`)

iOS:

- `tools/mobile/run.sh ios --app ui-gallery --sim`
- For real devices, use `tools/mobile/run.sh ios --app ui-gallery --device <udid> --team <team-id>`

5) Capture a diagnostics bundle (Android filesystem transport)

The Android wrapper sets `FRET_DIAG_DIR` to `files/fret-diag` by default (inside app storage).

To trigger a dump, bump the trigger stamp (value must change):

- `adb -s <serial> shell "run-as dev.fret.ui_gallery sh -c 'echo 1 > files/fret-diag/trigger.touch'"`
- `adb -s <serial> shell "run-as dev.fret.ui_gallery sh -c 'echo 2 > files/fret-diag/trigger.touch'"`

Then resolve and pull `bundle.json`:

- `ts="$(adb -s <serial> shell run-as dev.fret.ui_gallery cat files/fret-diag/latest.txt | tr -d '\r')"`
- `adb -s <serial> exec-out run-as dev.fret.ui_gallery cat "files/fret-diag/${ts}/bundle.json" > "target/fret-diag-mobile/${ts}.bundle.json"`

If `latest.txt` never appears, check:

- The app is actually rendering frames (if wgpu init fails, the frame loop may stop, and triggers won’t be polled).
- The app didn’t crash (check logcat for `Fatal signal` / wgpu init errors).

6) Extract Vulkan/driver evidence (what to archive)

From the captured bundle:

- `jq '.windows[0].snapshots[-1].wgpu_adapter | {selected_backend, adapter_name, driver, driver_info, downlevel_flags, is_webgpu_compliant, init_attempts}' target/fret-diag-mobile/<ts>.bundle.json`

Archive the JSON snippet under the relevant workstream milestone document.

## Evidence anchors

- Backend defaults + downlevel requirements: `crates/fret-render-wgpu/src/lib.rs`
- Android intent extras → env injection: `apps/fret-ui-gallery-mobile/android/app/src/main/java/dev/fret/ui_gallery/MainActivity.java`
- Android runner wrapper (build/install/start): `tools/mobile/android_game_activity_run.sh`
- Mobile run entrypoint: `tools/mobile/run.sh`
- Workstream smoke recipe (example evidence format): `docs/workstreams/mobile-gfx-backend-v1/m3-real-device-smoke-oppo-plg110.md`

## Common pitfalls

- `adb: device offline`: toggle USB/Wireless debugging and retry; avoid multi-device installs by passing `-d <serial>`.
- Emulator “supports Vulkan” but crashes after selecting Vulkan adapter: treat as emulator/driver instability; validate on real device.
- Forcing `--backend gl`: often fails required `DownlevelFlags::VERTEX_STORAGE` (expected with current contract).
- Trigger file exists but no bundle appears: the app may not be polling triggers (no frames due to wgpu init failure).

## Related skills

- `fret-diag-workflow` (scripted repros + gates + bundles)
- `fret-framework-maintainer-guide` (ADR/evidence discipline for hard-to-change contracts)
