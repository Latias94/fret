# ADR 0034: Timers, Animation, and Redraw Scheduling (Event-Driven with Continuous Mode)

Status: Accepted

## Context

Editor-grade UI must feel responsive:

- docking drag/drop,
- hover feedback and cursor changes,
- scrolling (potentially with inertia),
- animated transitions (optional),
- caret blinking and IME composition UI,
- multi-window tear-off behavior.

Without a stable frame scheduling contract, projects tend to accumulate:

- ad-hoc `request_redraw` calls scattered across widgets,
- busy loops or unnecessary redraws at idle (battery/CPU cost),
- inconsistent multi-window timing,
- difficulty meeting high-refresh displays smoothly.

References:

- App effects queue and runner draining:
  - `docs/adr/0001-app-effects.md`
- Canonical frame lifecycle and submission ordering:
  - `docs/adr/0015-frame-lifecycle-and-submission-order.md`
- Zed/GPUI performance and frame scheduling lessons:
  - https://zed.dev/blog/120fps
- Zed/GPUI scheduling primitives (non-normative code anchors):
  - `Window::refresh` (mark dirty) and the draw/present split:
    `repo-ref/zed/crates/gpui/src/window.rs` (`Window::refresh`, `Window::draw`, `Window::present`)

## Decision

### 1) Primary model: event-driven rendering with explicit continuous mode

Fret’s default scheduling is **event-driven**:

- no input/model changes → no redraw,
- effects and timers drive redraw requests.

Additionally, Fret supports **continuous redraw mode** per window (or per app), explicitly enabled for:

- real-time engine viewports,
- “play mode” / simulation,
- profiling sessions.

Continuous mode is a policy decision made by the host/editor app.

### 2) Timers are scheduled via app effects (not platform calls from widgets)

Introduce timer requests as data effects:

- `Effect::SetTimer { window: Option<AppWindowId>, token, after, repeat }`
- `Effect::CancelTimer { token }`

The platform runner owns the actual timer mechanism and injects timer events back into the app loop.

### 2.1) Timer routing is element-owned by default

When an element-owned hook schedules a timer, the runtime records the timer token’s **target element**.
When the runner later injects `Event::Timer { token }`, the UI runtime first attempts to dispatch it
to the recorded target element (if it is still mounted). If no target is found, the runtime falls
back to dispatching the timer event to visible layers that have opted into timer delivery.

This allows interaction policies (e.g. overlay/menu hover delays) to remain deterministic under
view caching and multi-layer input routing.

### 3) Animation frames are requested, not assumed

Introduce an explicit request for "next frame as soon as possible":

- `Effect::RequestAnimationFrame(AppWindowId)`

Any subsystem that needs continuous frames (drag operations, animations, caret blink, IME UI) requests
animation frames while active, and stops requesting when inactive.

### 3.1) Public API shape (P0; GPUI-aligned)

To avoid scattered ad-hoc `request_redraw` calls and "ever-growing mode switches", we lock a small,
composable scheduling API that matches GPUI/Zed's mental model:

- **One-shot frame request** (GPUI `Window::refresh()`):
  - `request_frame(window)` marks the window dirty and schedules it to draw *once* on the next
    available frame.
  - This request is coalesced per window per tick.
- **Animation frame request** (explicit "wake up next frame interval"):
  - `request_animation_frame(window)` requests a frame at the runner’s `frame_interval` cadence.
  - This is the correct primitive for short-lived animations (spinners, hover intent, transitions),
    inertial scrolling, caret blinking, and IME UI.
- **Continuous frames (RAII lease)**:
  - `begin_continuous_frames(window) -> ContinuousFrames` returns a small RAII lease object.
  - While *any* lease is alive, the runtime guarantees the window keeps requesting animation frames
    (effectively "continuous mode", but refcounted and localized).
  - When the last lease is dropped, the window returns to event-driven scheduling.

Design intent:

- Continuous rendering is expressed as **"hold a lease while you need it"**, not as a set of
  component-specific boolean toggles.
- The host/app may still provide an **explicit global override** (profiling/play-mode) that forces
  continuous rendering regardless of leases.

### 4) Coalescing and bounded fixed-point draining

The runner may coalesce redraw and timer wake-ups:

- multiple redraw requests for the same window collapse into one,
- multiple timers expiring in the same turn are handled in a bounded loop.

This integrates with the fixed-point effect draining model (ADR 0001) to prevent unbounded “effect storms”.

### 5) Time source is monotonic; frame ids are observable

Scheduling uses a monotonic clock.
Introduce stable, observable frame identifiers for tracing and resource pools:

- `TickId` increments per platform runner turn (event-loop “tick”), even if no rendering occurs.
- `FrameId` increments only when the runner actually performs a render submission/present for at least one window.

This supports:

- in-flight GPU resources (triple buffering style),
- profiling and debugging (ADR 0036).

Defaults (P0):

- Timers are globally scheduled but may carry an optional `AppWindowId` for wake-up affinity.
- A platform “display link” (vsync-driven callback) is an **internal runner implementation detail**; the cross-platform
  contract is `RequestAnimationFrame` + timer effects.
- Inertia/physics-based animations are app-/widget-owned policy; the framework provides only clock + scheduling primitives.
- On wasm, `RequestAnimationFrame` maps to browser `requestAnimationFrame`, timers map to `setTimeout`, and the framework
  accepts browser throttling as a documented constraint.
- `TickId` is the correct unit for “ignore platform echo events this turn” style logic (winit window move/resize echoes).
- `FrameId` is the correct unit for renderer pooling and cache eviction (“unused for N rendered frames”).

## Consequences

- Idle becomes truly idle (lower CPU/GPU), while still enabling smooth high-refresh interaction when needed.
- Multi-window redraw remains deterministic and centralized in the runner.
- Timers/animations become composable and testable as data effects.

## Open Questions (To Decide Before Implementation)

### Locked P0 Choices

1) **Continuous mode scope**:
   - Continuous mode is **per-window** by default (fits “some windows animate, others idle”).
   - The app may also enable an **app-global override** for profiling/play-mode, implemented as “treat all windows as continuous”.

2) **Expose `TickId` and `FrameId`**:
   - `TickId` is exposed to app/UI code as a read-only value for deterministic “same-turn echo suppression” and debugging.
   - `FrameId` is exposed to renderer/observability as the canonical “rendered frame counter” for pooling/eviction.

Additional locked behavior:

- A redraw request is always coalesced per window per tick.
- The effects drain loop is bounded; the runner must enforce a fixed upper bound (default `max_effect_drains_per_tick = 8`).

## Implementation Notes

MVP0 is implemented and provides the basic primitives described above.

Code anchors:

- Effects: `crates/fret-app/src/app.rs` (`Effect::{RequestAnimationFrame, SetTimer, CancelTimer}`)
- Runner coalescing/draining: `crates/fret-launch/src/runner/mod.rs`
