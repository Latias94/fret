# Action-First Authoring + View Runtime (Fearless Refactor v1) — Open Questions

Last updated: 2026-03-04

This file tracks decisions intentionally deferred **past the v1 landing**.

For v1 locked decisions, see:

- ADRs: `docs/adr/0307-action-registry-and-typed-action-dispatch-v1.md`,
  `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`
- Workstream decision snapshot: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md` (“Decision Snapshot”)

---

## Q1 — Should `ActionId` be distinct from `CommandId`?

Options:

- A) `ActionId` is a thin wrapper around `CommandId` (no schema churn; quickest adoption).
- B) `ActionId` is a distinct type and keymap schema gets an “action binding” variant (clearer, more explicit).

Tradeoff:

- A is cheaper now; B is cleaner long-term for DSL/spec frontends.

Decision (v1):

- Choose A for v1: treat `ActionId` as an alias/wrapper over `CommandId` (no keymap schema churn).
- Revisit B only after adoption, if payload actions or a DSL/frontend requires an explicit schema distinction.

---

## Q2 — Structured action payloads (v2)?

GPUI supports structured actions (serialized payloads) for keymap bindings.

For v1:

- prefer **unit actions only** (no payload) to keep dispatch simple.

For v2:

- define a minimal payload encoding (likely JSON) with strict validation and deterministic hashing rules.

---

## Q3 — Where should view runtime live?

Options:

- `ecosystem/fret` (golden path; simplest for apps),
- new `ecosystem/fret-view` crate re-exported by `fret` (cleaner dependency and testing boundaries).

---

## Q4 — How should MVU coexist with the view runtime?

Options:

- keep MVU indefinitely as an alternative paradigm (documented “use when”),
- treat MVU as legacy and plan a staged deprecation.

Recommendation:

- keep MVU initially; decide deprecation only after adoption evidence and a full cleanup milestone.

Decision (v1):

- Land the view runtime in `ecosystem/fret` (golden path). Defer a split crate until after M2/M3 adoption.
- MVU is quarantined as a compat surface under `fret::legacy::prelude::*` and is gated out of the cookbook/templates.

---

## Q5 — Diagnostics and picking mode interaction with view cache reuse

We already treat “inspection/picking” as a reason to disable some caching/reuse.

Question:

- which caches must be disabled to keep action availability/dispatch path and selector resolution deterministic?

This must stay aligned with:

- ADR 0159 (selectors + scripted interaction),
- ADR 0213 (cache roots).

Status (as of 2026-03-04):

- The v1 view-cache + action-first gates are green (`pwsh tools/diag_gate_action_first_authoring_v1.ps1`).
- Inspection/picking may still affect reuse decisions; gaps should be documented as explicit reuse reasons in cache-root diagnostics.
