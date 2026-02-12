# ADR 0260: Mobile Shell ↔ Runtime Bridge (v1)

Status: Proposed

## Context

Fret wants to support mobile platforms (Android + iOS) without turning the Rust workspace into a
platform-specific application repository.

In practice, mobile builds require “shell” projects:

- Android: an `Activity` (and related Gradle packaging) that hosts a native library.
- iOS: an Xcode project for codesigning and device installation.

We want a standardized developer experience and a stable integration seam, but we do not want to
lock ourselves into a specific build tool (Gradle vs generator tools) too early.

The hard-to-change part is not the packaging tool; it is the shell ↔ runtime contract:

- entrypoints (who owns the thread and event loop),
- diagnostics (logging + panics),
- platform service bridging (IME, insets, clipboard, file pickers),
- lifecycle + surface recreation policy,
- version/config injection semantics.

## Decision

### D1 — The “shell” is responsible for platform packaging and platform-only APIs

The mobile shell project is responsible for:

- platform packaging (APK/AAB, iOS app bundle),
- app identity (bundle id, display name, icons),
- codesigning (iOS) and signing (Android release),
- acquiring platform-only services (IME/keyboard, insets, clipboard, file pickers, share sheet),
- installing/initializing crash + logging backends appropriate for the platform.

The shell SHOULD be thin and treat the Rust runtime as the source of UI behavior.

### D2 — The Rust entrypoint must be a stable C ABI surface

The “mobile Rust entry crate” MUST expose a stable entrypoint per platform:

- Android: `android_main(app: AndroidApp)` (via `winit::platform::android::activity::AndroidApp`).
- iOS: `extern "C" fn <app>_ios_main()` (exact symbol name is app-specific but must be stable).

Rules:

- The iOS entrypoint MUST be called on the main thread.
- The Android entrypoint MUST build the `winit` event loop using the provided `AndroidApp`.

### D3 — Diagnostics must be initialized before the event loop is built

The Android entrypoint MUST initialize a logcat-capable tracing/log backend before creating the
event loop so early failures are observable.

The iOS entrypoint MUST initialize a platform-appropriate logging backend as early as possible.

The runtime MUST treat the logging backend as best-effort: initialization may fail and should not
crash the app.

### D4 — Platform service bridging is contract-driven and layered

Mobile-specific platform APIs are exposed to the Rust runtime via explicit services/effects, not by
ad-hoc direct calls from ecosystem component code.

Layering intent:

- `crates/fret-ui`: mechanism-only behavior.
- `ecosystem/*`: policy/components that can depend on the stable contract surfaces.
- runner/platform crates: translate contract effects into platform calls.

For v1 mobile bring-up, the minimum platform service bridge we must support is:

- IME: enable/disable + cursor rect updates (ADR 0012 + ADR 0261).
- Insets: safe-area + occlusion (keyboard) insets committed into window metrics (ADR 0232).
- Lifecycle + surfaces: suspended/resumed + surface error recovery policy (ADR 0262).

Pointer/touch semantics are a separate but required “contract baseline” for mobile readiness:

- see ADR 0263 (and ADR 0150/0151/0238/0243).

## Consequences

- We can standardize “one command to run on device” without freezing the packaging toolchain.
- The shell stays replaceable (Gradle today, generator tomorrow) as long as the entrypoints and
  service seams are preserved.
- We have an explicit place to evolve mobile capability surfaces without violating framework
  layering rules.

## Non-goals

- This ADR does not prescribe a specific build system (e.g. `cargo-apk`, `xbuild`, `tuist`).
- This ADR does not define a full mobile gesture arena contract.
- This ADR does not define distribution/release workflows beyond the responsibility boundaries.

## Contract anchors

- IME event model + caret anchoring: `docs/adr/0012-keyboard-ime-and-text-input.md`
- Platform text input client interop: `docs/adr/0261-platform-text-input-client-interop-v1.md`
- Environment queries + insets seam: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- Mobile lifecycle/surface policy: `docs/adr/0262-mobile-lifecycle-and-surface-policy-v1.md`
- Pointer/touch baseline: `docs/adr/0263-pointer-and-touch-semantics-baseline-v1.md`
