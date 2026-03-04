# MVU Long-term Status (Action-First Authoring v1+)

Last updated: 2026-03-04

This document locks the “when to use MVU” policy for the action-first authoring workstream.

## Decision

**MVU is legacy-only (compat), not a supported alternative golden path.**

Normative implications:

- New code (templates, cookbook, recommended app authoring) must use:
  - View runtime + hooks (ADR 0308),
  - typed actions (ADR 0307),
  - and payload actions v2 where per-item parameterization is required (ADR 0312).
- MVU remains available for:
  - existing in-tree demos that have not migrated yet,
  - small, explicitly-labeled experiments,
  - edge cases where the v1/v2 action-first surfaces are not sufficient.
- MVU must stay quarantined behind explicit imports:
  - use `fret::legacy::prelude::*` (do not rely on `fret::prelude::*`).
  - enable the `fret` feature `legacy-mvu` (MVU is opt-in).

Rationale:

- The repo must teach **one boring golden path** for new users.
- Action-first closes the cross-surface story (keyboard/menus/palette/pointer) and is diagnosable.
- View runtime + cache closure aligns with GPUI/Zed and keeps performance/correctness auditable.
- Payload actions v2 removes the main remaining practical reason to use MVU for common per-item cases.

## What MVU is for (allowed uses)

MVU can still be used when it is the lowest-risk choice **and** it is explicitly labeled as legacy:

1. Maintaining an existing MVU-based demo where migration adds no new evidence/gates.
2. Prototyping a short-lived experiment (one-file loop) where rewrite churn is expected.
3. A missing capability in the action-first surface:
   - payload actions v2 is insufficient (e.g. you need keymap-bound payload, durable/replayable
     payload semantics, or multi-step payload prompting),
   - or you need a router-like multiplexing pattern that does not yet have an action-first equivalent.

Even in these cases:

- Prefer stable ActionId/CommandId strings for any cross-surface behavior.
- Avoid stringly “prefix.{id}” parsing patterns unless there is no alternative; keep them local and
  document why.

## What MVU must NOT be used for

1. Cookbook examples and `fretboard new` templates (golden path).
2. New ecosystem policy/components (shadcn/material3/kit).
3. Anything that needs cross-surface command/availability gating correctness: use actions.

## Guidance: View vs payload actions vs MVU

Recommended default (new code):

- **View runtime + typed actions** for the authoring loop.

When you need per-item parameterization:

- Prefer **payload actions v2** (ADR 0312) for pointer/programmatic dispatch.
  - Keep keymap/palette/menus as unit actions (no schema changes in v2).
  - Treat missing payload as safe “not handled”.

When payload actions v2 is not sufficient:

- MVU is acceptable, but only as legacy/compat and with clear labeling.

## Deprecation window plan (M8)

This decision unblocks the MVU deprecation window milestone:

1. Docs/templates already teach the golden path (View + actions).
2. Next: add compile-time deprecations and/or feature-gate MVU surfaces (AFA-clean-064/065).
3. Only later: consider removal or tighter quarantine once all in-tree usages are migrated or
   explicitly legacy-labeled (see `LEGACY_MVU_INVENTORY.md`).

This repo does **docs-first deprecation**: do not add warnings before docs/templates stop teaching
the surface.

Status (as of 2026-03-04):

- MVU surfaces are compile-time deprecated.
- MVU is feature-gated behind `legacy-mvu` to keep downstream opt-in explicit.
