# ADR 0262: Mobile Lifecycle and Surface Policy (v1)

Status: Proposed

## Context

Fret’s core architecture assumes a runner-driven frame pipeline (ADR 0015) and effect-driven
platform integration (ADR 0003). On mobile, the platform lifecycle introduces a hard requirement:

- the app can be backgrounded/foregrounded at any time,
- the GPU surface can be dropped and later recreated (or become temporarily “lost/outdated”),
- and input services (IME/keyboard, insets) can change around these transitions.

If the runner does not provide a stable, minimal lifecycle/surface policy early, ecosystem code will
learn to “expect crashes” and will accumulate platform-specific hacks.

This ADR defines the v1 runner-level guarantees for Android/iOS bring-up and future mobile support.

## Goals

1. Define stable, debuggable behavior for `Suspended` / `Resumed` transitions.
2. Define a recovery policy for surface acquisition failures (`Lost`, `Outdated`, `Timeout`).
3. Keep app/component code insulated from surface recreation (no “surface lost” leaking upward).
4. Ensure the first post-resume frame presents without waiting for user input when possible.

## Non-goals (v1)

- Prescribing a specific mobile packaging toolchain (Gradle/Xcode). See ADR 0260.
- Defining a full “gesture arena” or mobile navigation stack.
- Guaranteeing perfect zero-jank resume; the goal is correctness + a stable seam.

## Decision

### D1 — Lifecycle is runner-owned and does not emit app-level events by default

The runner owns platform lifecycle handling. The default Fret runtime/app model MUST NOT require
apps to handle explicit lifecycle events to remain correct.

Rationale:

- Lifecycle handling is mostly about surfaces, rendering, and platform services.
- App state should remain consistent across suspend/resume without requiring special-case code.

Apps may still observe lifecycle indirectly via:

- lack of frames while suspended,
- platform capability changes committed by the runner (ADR 0054),
- environment snapshot changes (ADR 0232) after resume.

### D2 — Suspended: stop presenting and drop surface resources (best-effort)

When the platform reports `Suspended`:

1. The runner MUST stop presenting frames for affected windows.
2. The runner SHOULD drop/destroy platform surfaces and any surface-dependent GPU state (best-effort).
3. The runner MUST ensure the event loop does not spin aggressively while suspended.

Notes:

- “Destroy surfaces” is best-effort because some backends may not expose explicit teardown hooks.
- The runner may continue to drain non-render effects if doing so is safe, but it MUST NOT present.

### D3 — Resumed: recreate surfaces and present on the next turn

When the platform reports `Resumed`:

1. The runner MUST attempt to recreate surfaces eagerly when possible.
2. The runner MUST request a redraw for all windows so the first post-resume frame can present
   without waiting for additional user input.
3. The runner SHOULD drain effects after requesting redraw so that pending UI/runtime effects
   (e.g. IME allow/cursor-area updates, timers) do not lag.

### D4 — Surface acquisition failures are recoverable except OOM

When acquiring a surface frame fails during render/present, the runner applies the following policy:

- `wgpu::SurfaceError::Lost` or `wgpu::SurfaceError::Outdated`:
  - Treat as recoverable.
  - Resize/reconfigure the surface using the current window size.
  - Request a redraw (one-shot) so the window does not remain blank.

- `wgpu::SurfaceError::Timeout`:
  - Treat as transient.
  - Request a redraw (one-shot) to avoid waiting indefinitely for user input.

- `wgpu::SurfaceError::OutOfMemory`:
  - Treat as fatal.
  - The runner MUST shut down the app/event loop cleanly (best-effort).

Other/unknown surface errors MAY be ignored (best-effort), but runners SHOULD log in debug builds.

### D5 — Insets + IME are re-synchronized after resume

After resume (and after any surface recreation), the runner SHOULD re-apply platform service state
based on the latest committed snapshots/effects:

- IME enablement + caret anchoring (ADR 0012 + ADR 0261).
- Safe-area and occlusion insets committed into window metrics (ADR 0232).

Rationale: on mobile the OS can invalidate input sessions and insets state across transitions.

## Consequences

- Mobile bring-up can be judged on stable, observable outcomes (no crash/black screen loops).
- Ecosystem components do not need platform branches for surface-lost handling.
- The runner has a clear place to evolve platform-specific lifecycle quirks without leaking them into
  `crates/fret-ui`.

## Implementation status (current)

As of 2026-02-12:

Implemented (desktop + web evidence anchors):

- Surface error recovery (desktop): `crates/fret-launch/src/runner/desktop/app_handler.rs`
- Surface error recovery (web): `crates/fret-launch/src/runner/web/render_loop.rs`

In progress / target gaps for mobile bring-up:

- Mobile-specific `Suspended` / `Resumed` behavior is runner-owned but not yet validated on Android/iOS.
- Post-resume re-synchronization of IME + insets should be verified with diagnostics scenarios once
  mobile runners exist (ADR 0261 + ADR 0232).

## Evidence anchors (current implementation)

- `Suspended` / `Resumed` handlers (mobile targets): `crates/fret-launch/src/runner/desktop/app_handler.rs`
- Surface error recovery (desktop): `crates/fret-launch/src/runner/desktop/app_handler.rs`
- Surface error recovery (web): `crates/fret-launch/src/runner/web/render_loop.rs`
