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
| NavigationMenu chevron | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/navigation-menu.tsx` (chevron `transition duration-300 ... rotate-180`) | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs:1551` | Smooth chevron rotate on open/close (duration ~300ms) | Missing transition (instant rotate) | shadcn recipe (uses kit transition) | Fixed-delta diag screenshot + unit invariant | Gap | Should mirror Accordion transition driver pattern. |
| Spinner | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/spinner.tsx` (`animate-spin`) | `ecosystem/fret-ui-shadcn/src/spinner.rs:98` | Continuous rotation with stable timebase; reduced motion stops | Frame-driven (`frame_id * speed`) | shadcn recipe (use kit motion dt) | Unit test for reduced motion + deterministic diag | Gap | Current API is “radians per frame”; likely refactor to duration-based. |
| Skeleton | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/skeleton.tsx` (`animate-pulse`) | `ecosystem/fret-ui-shadcn/src/skeleton.rs:76` | Pulse alpha over time; reduced motion settles | Frame-driven (`frame_id` + `sin`) | shadcn recipe (use kit motion dt) | Unit test for settling + diag snapshot | Gap | Upstream pulse is duration-based; keep tokenized duration. |
| InputOtp caret | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input-otp.tsx:62` (`animate-caret-blink ... duration-1000`) | `ecosystem/fret-ui-shadcn/src/input_otp.rs:412` | Caret blinks on a 1000ms cycle; reduced motion disables blink | Frame-driven modulo (`frame_id / 30`) | shadcn recipe (use kit motion dt) | Unit test for cycle timing under fixed delta | Gap | Needs a duration-driven blink model (no implicit 60Hz). |
| Avatar fallback delay | Radix `delayMs` outcome (see Radix avatar primitive) | `ecosystem/fret-ui-kit/src/primitives/avatar.rs:24` + `ecosystem/fret-ui-shadcn/src/avatar.rs:734` | Delay is time-based (ms), not frame-count based | Frame-count API (`delay_frames`) | kit primitive | Unit test with fixed-delta semantics or new ms-based helper | Gap | Decide whether to preserve a frame-based helper for determinism, but expose ms-based API at shadcn layer. |

