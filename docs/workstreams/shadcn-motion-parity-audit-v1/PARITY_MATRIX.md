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
| NavigationMenu chevron | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/navigation-menu.tsx` (chevron `transition duration-300 ... rotate-180`) | `ecosystem/fret-ui-shadcn/src/navigation_menu.rs:1551` | Smooth chevron rotate on open/close (duration ~300ms) | — | shadcn recipe (uses kit transition) | Unit test for transition settling (optional: fixed-delta diag screenshots) | Aligned | Gate: `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` test `trigger_chevron_motion_advances_and_settles_like_a_300ms_transition`. |
| Spinner | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/spinner.tsx` (`animate-spin`) | `ecosystem/fret-ui-shadcn/src/spinner.rs:98` | Continuous rotation with stable timebase; reduced motion stops | — | shadcn recipe (kit motion loop) | Unit test for timebase + reduced motion (optional: diag screenshot) | Aligned | Gate: `ecosystem/fret-ui-shadcn/src/spinner.rs` test `spinner_rotation_advances_with_fixed_delta_and_respects_reduced_motion`. |
| Skeleton | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/skeleton.tsx` (`animate-pulse`) | `ecosystem/fret-ui-shadcn/src/skeleton.rs:76` | Pulse alpha over time; reduced motion settles | — | shadcn recipe (kit motion loop) | Unit test for alpha modulation (optional: diag screenshot) | Aligned | Gate: `ecosystem/fret-ui-shadcn/src/skeleton.rs` test `skeleton_pulse_changes_background_alpha_across_frames`. |
| InputOtp caret | `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input-otp.tsx:62` (`animate-caret-blink ... duration-1000`) | `ecosystem/fret-ui-shadcn/src/input_otp.rs:412` | Caret blinks on a 1000ms cycle; reduced motion disables blink | — | shadcn recipe (kit motion loop) | Unit test for blink toggle under fixed delta (needs caret selector) | Aligned (no gate) | Duration-driven blink (no implicit 60Hz coupling), but lacks a stable caret gate today. |
| Avatar fallback delay | Radix `delayMs` outcome (see Radix avatar primitive) | `ecosystem/fret-ui-kit/src/primitives/avatar.rs` + `ecosystem/fret-ui-shadcn/src/avatar.rs:661` | Delay is time-based (ms), not frame-count based | — | kit primitive + shadcn recipe | Unit tests (kit primitive + shadcn integration) | Aligned | `delay_ms` is duration-driven; frame-based inputs are treated as 60Hz reference ticks. |
