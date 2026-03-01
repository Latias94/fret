# Action-First Authoring + View Runtime (Fearless Refactor v1) — Open Questions

Last updated: 2026-03-01

This file lists decisions intentionally deferred while we draft the v1 contracts.

---

## Q1 — Should `ActionId` be distinct from `CommandId`?

Options:

- A) `ActionId` is a thin wrapper around `CommandId` (no schema churn; quickest adoption).
- B) `ActionId` is a distinct type and keymap schema gets an “action binding” variant (clearer, more explicit).

Tradeoff:

- A is cheaper now; B is cleaner long-term for DSL/spec frontends.

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

---

## Q5 — Diagnostics and picking mode interaction with view cache reuse

We already treat “inspection/picking” as a reason to disable some caching/reuse.

Question:

- which caches must be disabled to keep action availability/dispatch path and selector resolution deterministic?

This must stay aligned with:

- ADR 0159 (selectors + scripted interaction),
- ADR 0213 (cache roots).

