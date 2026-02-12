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

- Use `cargo-apk` + `NativeActivity` to package a `cdylib`:
  - Entry crate: `apps/fret-ui-gallery-mobile`
  - Script: `tools/mobile/android_apk_run.sh`

Prereqs (typical):

- Android SDK + platform-tools (`adb`)
- Android NDK (cargo-apk will try to discover it from the SDK install)

Notes:

- The upstream tool is deprecated in favour of `xbuild`, but it is still useful for a quick smoke test.
- When we need Java/Kotlin integration (GameActivity / IME improvements / clipboard / share sheet),
  we should revisit the packaging choice.

## iOS (real device)

We currently have a fast simulator loop (`tools/mobile/ios_sim_run.sh`).
For real devices we need **codesigning**, which typically implies an Xcode project.

### Option A: Xcode wrapper app (recommended for now)

Use `apps/fret-ui-gallery-mobile` as the Rust entrypoint and call into it from Swift/ObjC.
That crate exports:

- `fret_ui_gallery_ios_main()` (C ABI, no args)

High-level steps:

1. Create an Xcode iOS App project (Swift UI or UIKit).
2. Add the Rust static library output to the project:
   - Build Rust for device: `cargo build -p fret-ui-gallery-mobile --target aarch64-apple-ios --release`
   - The artifact is `target/aarch64-apple-ios/release/libfret_ui_gallery_mobile.a`
3. Add a bridging header (or module map) that declares:
   - `void fret_ui_gallery_ios_main(void);`
4. Call `fret_ui_gallery_ios_main()` from the app’s startup path **on the main thread**.
5. Enable Automatic Signing in Xcode and run on device.

Notes:

- Winit’s iOS backend must run on the main thread. Keep the call site in `@main` / `UIApplicationMain` flow.
- This wrapper app is intentionally minimal; once stable we can automate it (XcodeGen / tuist / xbuild).

### Option B: Adopt a generator tool

Candidates (evaluate later):

- `xbuild` (rust-mobile)
- `cargo-mobile2`

Tradeoffs:

- Faster “one command” packaging, but introduces a toolchain dependency and a generated-project story.

