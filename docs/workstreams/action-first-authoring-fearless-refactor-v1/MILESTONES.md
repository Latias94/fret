# Action-First Authoring + View Runtime (Fearless Refactor v1) — Milestones

Last updated: 2026-03-03

Related:

- Design: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`

---

## Current status snapshot (as of 2026-03-03)

- **M0**: Met (ADRs 0307/0308 are accepted; ADR index jump table is updated).
- **M1**: Met (ActionId identity + typed unit actions + converged metadata via command registry; dispatch diagnostics include handler scope and driver-handled classification).
- **M2**: Met (View runtime v1 + hooks + view-cache keepalive + gates exist; cookbook adoption landed).
- **M3**: Met (Declarative + imui + GenUI converge on the same ActionId dispatch and stable selectors; cross-frontend diag gate exists).
- **M4**: Met (ui-gallery includes an action-first view runtime snippet; templates + docs converge on View+actions).
- **M5**: Met (workspace shell demo tab strip uses action-first pointer dispatch hooks; scripted diag gate asserts pointer dispatch trace exists).
- **M6**: Met (legacy MVU authoring is quarantined; golden path is action-first + view runtime).
  - Status (as of 2026-03-03): MVU remains available as compat under `fret::legacy::prelude::*` while cookbook/templates stay MVU-free.

---

## M0 — Decision gates locked (ADRs accepted)

Exit criteria:

- ADR 0307 and ADR 0308 exist as the canonical contract references (Status: Accepted).
- Keymap strategy (ActionId vs CommandId) is explicit and stable for v1.
- `docs/adr/README.md` jump table points to the new ADRs.

---

## M1 — Action system v1 landed (additive)

Exit criteria:

- Action IDs are stable and debuggable.
- UI can bind to actions without string parsing glue.
- Keymap can trigger actions and diagnostics can explain availability/dispatch outcomes.
- At least one palette/menu trigger path uses the same action dispatch pipeline (no divergence).

Notes:

- v1 may keep `ActionId` == `CommandId` to avoid schema churn; the key is the authoring surface and routing semantics.

---

## M2 — View runtime v1 landed (minimal, ecosystem-level)

Exit criteria:

- A minimal `View` + `ViewCx` exists with:
  - action handler table registration,
  - `notify()` dirty marking,
  - `use_state`, `use_selector`, `use_query` integration surfaces.
- At least one demo renders via the view runtime without MVU.
- At least one gate explains view rebuild reasons (notify vs observed deps vs inspection mode).

---

## M3 — Multi-frontend convergence (imui + GenUI alignment)

Exit criteria:

- imui can dispatch `ActionId` directly (no string commands required).
- GenUI action bindings align with action conventions and can trigger the same action handler surfaces.
- Diagnostics selectors still work across all frontends (stable `test_id` surfaces).

---

## M4 — Adoption (cookbook + gallery)

Exit criteria:

- At least 2 cookbook examples migrated and used as the “before/after” teaching baseline.
- At least 1 ui-gallery page/snippet migrated.
- Docs show a single golden path for new users.
- `fretboard` templates do not teach a conflicting default paradigm (updated or explicitly deferred).

---

## M5 — Editor-grade proof (docking/workspace integration)

Exit criteria:

- At least one editor-grade surface (workspace shell / docking) uses the new action-first routing where appropriate.
- Regression gates exist (tests + diag script) for action routing + availability under overlays/focus changes.

---

## M6 — Cleanup (delete legacy, keep it boring)

Exit criteria:

- Redundant/legacy APIs are removed or clearly quarantined as “legacy”.
- Templates default to action-first + view runtime patterns.
- No in-tree demo requires stringly command routing glue.
- `cargo nextest run` gates remain green.
- “Risk matrix” items (R1–R6) have explicit mitigations/gates or are explicitly deferred.
