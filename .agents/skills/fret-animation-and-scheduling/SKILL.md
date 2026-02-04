---
name: fret-animation-and-scheduling
description: Scheduling and animation in Fret (runner-owned frame loop). Use when implementing animations/transitions, timers, hover-intent delays, or when deciding between `request_frame`, `request_animation_frame`, and continuous-frames leases (`ContinuousFrames`).
---

# Fret animation and scheduling

Fret’s runner owns the event loop and frame clock. Declarative UI drives time via **effects**
(`Effect::RequestAnimationFrame`, `Effect::SetTimer`) and **continuous frames leases**.

## Overview

**Redraw primitives:**

- One-shot redraw: `ElementContext::request_frame()` → `Effect::Redraw(window)`
- One-shot RAF: `ElementContext::request_animation_frame()` → `Effect::RequestAnimationFrame(window)`
- Continuous frames (preferred for declarative animations): `ElementContext::begin_continuous_frames()`
  - Holding the returned `ContinuousFrames` lease keeps RAF requests flowing.

**Ecosystem helpers (recommended):**

- Continuous frames lease management: `fret_ui_kit::declarative::scheduling::set_continuous_frames`
- Transition drivers: `fret_ui_kit::declarative::transition::*`
- Presence helpers (fade/scale-fade): `fret_ui_kit::declarative::presence::*`

## Quick start

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

## Timers (delays, hover intent, auto-close)

Prefer runner-owned timer effects over ad-hoc thread timers:

- Schedule via `Effect::SetTimer` / `Effect::CancelTimer` (tokened by `TimerToken`)
- Handle via timer hooks on the relevant element:
  - `ElementContext::timer_on_timer_for(...)`
  - `ElementContext::timer_add_on_timer_for(...)`

Use `WeakModel<T>` in long-lived callbacks when the timer should not keep state alive.

## Best practices

- Prefer `presence::*` + `transition::*` helpers over custom “tick counters” in leaf components.
- Always tie leases to element lifetime (`with_state`) so unmount stops scheduling.
- While animating opacity/transform, call `cx.notify_for_animation_frame()` (helpers do this) so
  paint-cache roots rerun paint deterministically.

## References

- Scheduling contract: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- Execution/portability surface: `docs/adr/0190-execution-and-concurrency-surface-v1.md`
- Runtime APIs:
  - `crates/fret-ui/src/elements/cx.rs` (`request_frame`, `request_animation_frame`, `begin_continuous_frames`)
  - `crates/fret-runtime/src/effect.rs` (`Effect::Redraw`, `Effect::RequestAnimationFrame`, timers)
- Ecosystem helpers:
  - `ecosystem/fret-ui-kit/src/declarative/scheduling.rs`
  - `ecosystem/fret-ui-kit/src/declarative/transition.rs`
  - `ecosystem/fret-ui-kit/src/declarative/presence.rs`
