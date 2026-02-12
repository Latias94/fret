---
name: fret-animation-and-scheduling
description: Scheduling and animation in Fret (runner-owned frame loop). Use when implementing animations/transitions, timers, hover-intent delays, or when deciding between `request_frame`, `request_animation_frame`, and continuous-frames leases (`ContinuousFrames`).
---

# Fret animation and scheduling

Fret’s runner owns the event loop and frame clock. Declarative UI drives time via **effects**
(`Effect::RequestAnimationFrame`, `Effect::SetTimer`) and **continuous frames leases**.

## When to use

Use this skill when:

- Implementing animations/transitions (presence, expand/collapse, spinners).
- Adding timed behavior (debounce, hover intent delays, auto-close).
- Deciding between one-shot redraw, RAF, and continuous frames leases.

## Inputs to collect (ask the user)

Ask these up front so scheduling stays deterministic and doesn’t leak:

- Behavior class: one-shot redraw, timed delay, continuous animation, or transition/presence?
- Lifecycle: when should the animation start/stop (what is the controlling state)?
- Ownership: which element should own the continuous frames lease (and how is it stored/dropped)?
- Timing constraints: is a fixed frame delta needed for tests/repros?
- Regression gate: do we need a diag script to lock down timing-sensitive behavior?

Defaults if unclear:

- Use `presence`/`transition` helpers and tie continuous frames to element-local state so unmount drops the lease.

## Smallest starting point (one command)

- `cargo run -p fretboard -- dev native --bin effects_demo`

## Quick start

### Redraw primitives (what to reach for)

**Redraw primitives:**

- One-shot redraw: `ElementContext::request_frame()` → `Effect::Redraw(window)`
- One-shot RAF: `ElementContext::request_animation_frame()` → `Effect::RequestAnimationFrame(window)`
- Continuous frames (preferred for declarative animations): `ElementContext::begin_continuous_frames()`
  - Holding the returned `ContinuousFrames` lease keeps RAF requests flowing.

**Ecosystem helpers (recommended):**

- Continuous frames lease management: `fret_ui_kit::declarative::scheduling::set_continuous_frames`
- Transition drivers: `fret_ui_kit::declarative::transition::*`
- Presence helpers (fade/scale-fade): `fret_ui_kit::declarative::presence::*`

### 1) Fade presence (open/close with opacity)

```rust
use fret_ui_kit::prelude::*;
use fret_ui_kit::declarative::presence;

pub fn fade_example<H: UiHost>(cx: &mut ElementContext<'_, H>, open: bool) -> AnyElement {
    let out = presence::fade_presence(cx, open, 6);
    if !out.present {
        return ui::container(cx, |_| Vec::new()).into_element(cx);
    }

    // Apply `out.opacity` via an Opacity element or a higher-level recipe.
    // The helper already requests frames while animating.
    ui::text(cx, format!("opacity={:.2}", out.opacity)).into_element(cx)
}
```

### 2) Tie continuous frames to element state (avoid leaking RAF requests)

```rust
use fret_ui_kit::prelude::*;
use fret_ui_kit::declarative::scheduling::set_continuous_frames;

pub fn spinner_like<H: UiHost>(cx: &mut ElementContext<'_, H>, spinning: bool) -> AnyElement {
    set_continuous_frames(cx, spinning);
    ui::text(cx, if spinning { "spinning" } else { "idle" }).into_element(cx)
}
```

## Workflow (recommended checklist)

1. Prefer ecosystem helpers (`presence`, `transition`) for common animations.
2. Tie any continuous-frames lease to element lifetime:
   - Store the lease in element-local state so unmount drops it (no leaked RAF requests).
3. Prefer tokened, runner-owned timers (`Effect::SetTimer` / `CancelTimer`) for UI-visible time.
4. For delays and long-lived callbacks, prefer weak models to avoid keeping state alive.

## Definition of done (what to leave behind)

- The chosen primitive matches the intent (redraw vs RAF vs continuous frames vs timer).
- Any continuous frames lease is tied to element lifetime (no leaked scheduling after unmount).
- Timed behavior uses runner-owned timers (no ad-hoc threads/timers for UI-visible time).
- If behavior is timing-sensitive, there is at least one stable repro gate (unit test or `tools/diag-scripts/*.json`).

## Timers (delays, hover intent, auto-close)

Prefer runner-owned timer effects over ad-hoc thread timers:

- Schedule via `Effect::SetTimer` / `Effect::CancelTimer` (tokened by `TimerToken`)
- Handle via timer hooks on the relevant element:
  - `ElementContext::timer_on_timer_for(...)`
  - `ElementContext::timer_add_on_timer_for(...)`

Use `WeakModel<T>` in long-lived callbacks when the timer should not keep state alive.

## Common pitfalls

- Leaking continuous frames by keeping a `ContinuousFrames` lease alive after unmount.
- Using ad-hoc thread timers for UI-visible behavior (breaks determinism and diagnostics).
- Re-issuing the same “scroll/animation request” every frame instead of treating it as a one-shot intent.

## Best practices

- Prefer `presence::*` + `transition::*` helpers over custom “tick counters” in leaf components.
- Always tie leases to element lifetime (`with_state`) so unmount stops scheduling.
- While animating opacity/transform, call `cx.notify_for_animation_frame()` (helpers do this) so
  paint-cache roots rerun paint deterministically.

## Evidence anchors (where to look)

- Scheduling contract: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- Execution/portability surface: `docs/adr/0184-execution-and-concurrency-surface-v1.md`
- Runtime APIs:
  - `crates/fret-ui/src/elements/cx.rs` (`request_frame`, `request_animation_frame`, `begin_continuous_frames`)
  - `crates/fret-runtime/src/effect.rs` (`Effect::Redraw`, `Effect::RequestAnimationFrame`, timers)
- Ecosystem helpers:
  - `ecosystem/fret-ui-kit/src/declarative/scheduling.rs`
  - `ecosystem/fret-ui-kit/src/declarative/transition.rs`
  - `ecosystem/fret-ui-kit/src/declarative/presence.rs`

## Related skills

- `fret-action-hooks` (timers + long-lived callbacks wired as component-owned policy)
- `fret-overlays-and-focus` (hover intent, delayed tooltips/menus)
- `fret-diag-workflow` (scripted repros for timing bugs)
