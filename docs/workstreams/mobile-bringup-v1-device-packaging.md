---
title: Mobile Bring-up (v1) — Device Packaging Notes
status: draft
date: 2026-02-11
scope: Android + iOS real-device install/signing notes
---

# Mobile Bring-up (v1) — Device Packaging Notes

This is a pragmatic “how do we get something on a phone” note for `fret-ui-gallery`.
It is **not** a long-term packaging decision.

## Android (device/emulator)

Current bring-up loop:

- Use a minimal Gradle wrapper + `GameActivity` to package a `cdylib`:
  - Entry crate: `apps/fret-ui-gallery-mobile`
  - Gradle wrapper: `apps/fret-ui-gallery-mobile/android`
  - Script: `tools/mobile/run.sh android --app ui-gallery`

Prereqs (typical):

- Android SDK + platform-tools (`adb`)
- Android NDK (the helper script will try to discover it from the SDK install)
- `cargo-ndk` (`cargo install cargo-ndk`)

Notes:

- `NativeActivity` is still useful for low-friction rendering bring-up, but it is not sufficient for
  reliable IME / composing text input on modern devices.
- `GameActivity` is treated as the baseline for our MVP UX (scroll + input + keyboard avoidance).

## iOS (real device)

We currently have a fast simulator loop (`tools/mobile/ios_sim_run.sh`).
For real devices we need **codesigning**, which typically implies an Xcode project.

### Option A: Xcode wrapper app (recommended for now)

Use a thin Xcode wrapper that calls into the Rust entrypoint *before* `UIApplicationMain`.

Wrapper:

- Xcode project: `apps/fret-ui-gallery-mobile/ios`
- Entrypoint source: `apps/fret-ui-gallery-mobile/ios/FretUIGalleryMobile/main.m`

The Rust entry crate exports:

- `fret_ui_gallery_ios_main()` (C ABI, no args; expected to call into winit which calls `UIApplicationMain`).

High-level steps:

1. Build + copy the Rust static library:
   - `cargo build -p fret-ui-gallery-mobile --target aarch64-apple-ios --release`
   - Copy `target/aarch64-apple-ios/release/libfret_ui_gallery_mobile.a` into
     `apps/fret-ui-gallery-mobile/ios/RustLibs/`
2. Open `apps/fret-ui-gallery-mobile/ios/FretUIGalleryMobile.xcodeproj` in Xcode.
3. Enable Automatic Signing (select your team).
4. Run on device.

Scripted:

- `IOS_TEAM_ID=<team-id> tools/mobile/run.sh ios --app ui-gallery --device <udid> --release`

Notes:

- Winit’s iOS backend starts the application via `UIApplicationMain`. The wrapper must not call
  `UIApplicationMain` itself.
- The wrapper app is intentionally minimal; we can migrate to generated projects later if desired.

### Option B: Adopt a generator tool

Candidates (evaluate later):

- `xbuild` (rust-mobile)
- `cargo-mobile2`

Tradeoffs:

- Faster “one command” packaging, but introduces a toolchain dependency and a generated-project story.
