# Action-First Authoring + View Runtime (Fearless Refactor v1) — Cleanup Plan

Last updated: 2026-03-06

This document exists to prevent a common failure mode of fearless refactors:

> “We landed a new surface, but the old surface stayed, docs drifted, templates diverged, and
> the repo now teaches three different ways to do the same thing.”

This workstream is only “done” if we leave a **clean architecture** behind.

---

## Status (as of 2026-03-06)

The repo is converging on the golden path (**View runtime + typed actions**) but cleanup/deletion is
not complete yet:

- Templates: `fretboard new` scaffolds prefer View + typed actions.
- Cookbook/examples: migrations are ongoing; keep teaching surfaces consistent and regression-gated.
- MVU: still exists in-tree as a compatibility surface, but it is not the recommended authoring path.
  - Planned: MVU deprecation window (M8), then hard delete (M9) once adoption gates are met.

This plan exists to keep the refactor boring at the end: no drift, no split teaching surfaces, and a
clear deprecation → deletion sequence.

---

## 1) Cleanup principles

1) **No deletions until adoption is proven**
   - Do not delete an old surface until all in-tree demos + official ecosystem crates are migrated
     (or explicitly labeled as legacy on purpose).

2) **Deprecate in docs first, then in code**
   - Step 1: update docs/templates to stop teaching the legacy surface.
   - Step 2: add deprecation warnings (where feasible).
   - Step 3: delete/quarantine only after a full milestone cycle.

3) **Quarantine is valid**
   - If deletion is too risky, move legacy surfaces into clearly named modules:
     - `legacy::*`, `compat::*`, or feature-gated `compat-*` modules.

4) **Keep diagnostics stable**
   - When an API migration affects `test_id` conventions or action IDs, update diag scripts
     and keep the inspector output predictable.

---

## 2) Target “golden path” after cleanup

For new app authors (cookbook/templates):

- View runtime + hooks (selectors/queries) for the default loop.
- Action-first dispatch for pointer/key/palette integration.
- imui as an optional authoring frontend for tool panels/debug surfaces.
- GenUI as an optional spec frontend (guardrailed), aligned on action IDs.

---

## 3) Legacy surfaces to demote (candidates)

This list is intentionally phrased as “demote” rather than “delete”:

### 3.1 Stringly command routing patterns

These should remain disallowed in golden-path code:

- `"prefix.{id}"` parsing in handlers
- ad-hoc routing tables built from string patterns

Repo already has checks for these patterns; keep them and update them to the new golden path.

### 3.2 MVU typed command routers (`MessageRouter`, `KeyedMessageRouter`) — planned removal

These legacy routers existed as a first-party ecosystem surface during the v1 transition.

Status (as of 2026-03-06):

- Compatibility-only today.
- Planned removal at M9 once all in-tree teaching surfaces and ecosystem defaults have migrated.

Replacement guidance:

- Use typed actions (ADR 0307) and payload actions v2 (ADR 0312) for per-item pointer/programmatic
  dispatch.

### 3.3 Duplicate authoring entry points

Avoid two overlapping entry points that teach different patterns:

- “command-first” and “action-first” should converge; keep one recommended story.
- If two paths remain, they must have explicit “when to use” guidance.

---

## 4) Template + docs convergence tasks (non-optional)

- Update `fretboard` templates to generate action-first + view-runtime examples.
- Update `docs/README.md` “State management” section to list the new golden path primitives.
- Keep `docs/workstreams/authoring-paradigm-gpui-style-v1.md` aligned:
  - it may reference MVU as legacy or as an alternative; but it must not contradict the golden path.

---

## 5) Post-migration deletion checklist (when safe)

Only after M6 exit criteria:

- Remove/quarantine legacy modules not used by any in-tree code.
- Remove obsolete docs and keep a single “Start here” path.
- Update CI grep gates / check scripts to enforce the new golden path.
