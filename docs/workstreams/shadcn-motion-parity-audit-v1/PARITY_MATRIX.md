# Shadcn Motion Parity Audit v1 — Parity Matrix

Last updated: 2026-03-03.

Legend (Status):

- `Aligned`: matches upstream outcome and has a gate.
- `Aligned (no gate)`: looks correct but needs a regression gate.
- `Gap`: known mismatch with a concrete fix plan.
- `Not audited`: not reviewed yet.

| Component | Upstream source (repo-ref/ui) | Fret impl | Expected motion outcome | Gap type | Owner layer | Gate plan | Status | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Accordion chevron | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/accordion.tsx` (transition on chevron) | `ecosystem/fret-ui-shadcn/src/accordion.rs` | Smooth chevron rotation on toggle; respects reduced motion | — | shadcn recipe | Fixed-delta diag + unit tests | Aligned | Landed previously (use as reference pattern). |
| NavigationMenu chevron | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/navigation-menu.tsx` (chevron `transition duration-300 ... rotate-180`) | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs:1551` | Smooth chevron rotate on open/close (duration ~300ms) | — | shadcn recipe (uses kit transition) | (recommended) add fixed-delta diag + bounds gate | Aligned (no gate) | Uses the kit transition driver to mirror Accordion's chevron motion model. |
| Spinner | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/spinner.tsx` (`animate-spin`) | `ecosystem/fret-ui-shadcn/src/spinner.rs:98` | Continuous rotation with stable timebase; reduced motion stops | — | shadcn recipe (kit motion loop) | Add deterministic diag + reduced motion unit test | Aligned (no gate) | Uses `fret-ui-kit` duration-driven loop progress (no `frame_id * speed`). |
| Skeleton | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/skeleton.tsx` (`animate-pulse`) | `ecosystem/fret-ui-shadcn/src/skeleton.rs:76` | Pulse alpha over time; reduced motion settles | — | shadcn recipe (kit motion loop) | Add deterministic diag snapshot | Aligned (no gate) | Duration-driven pulse (stable across fixed-delta and high refresh rates). |
| InputOtp caret | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input-otp.tsx:62` (`animate-caret-blink ... duration-1000`) | `ecosystem/fret-ui-shadcn/src/input_otp.rs:412` | Caret blinks on a 1000ms cycle; reduced motion disables blink | — | shadcn recipe (kit motion loop) | Add unit test for cycle timing under fixed delta | Aligned (no gate) | Duration-driven blink (no implicit 60Hz coupling). |
| Avatar fallback delay | Radix `delayMs` outcome (see Radix avatar primitive) | `ecosystem/fret-ui-kit/src/primitives/avatar.rs:24` + `ecosystem/fret-ui-shadcn/src/avatar.rs:734` | Delay is time-based (ms), not frame-count based | Frame-count API (`delay_frames`) | kit primitive | Unit test with fixed-delta semantics or new ms-based helper | Gap | Decide whether to preserve a frame-based helper for determinism, but expose ms-based API at shadcn layer. |
