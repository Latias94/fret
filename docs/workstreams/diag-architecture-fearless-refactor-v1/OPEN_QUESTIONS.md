# Diagnostics Architecture (Fearless Refactor v1) — Open Questions

Last updated: 2026-03-02

This file lists decisions intentionally deferred while the architecture stabilizes.

---

## Q1 — Where should the long-term “extensions registry” live?

Options:

- A) `ecosystem/fret-bootstrap` (fast, lowest churn; extensions are “golden path runtime” only).
- B) `crates/fret-runtime` store (cleaner layering; ecosystem crates publish snapshots without exporter coupling).

Recommendation (v1):

- Start with A to validate the contract and the bounding/clipping story.
- Graduate to B only if coupling/churn becomes a real cost (measured by PR churn and dependency pressure).

---

## Q2 — Should extensions be typed or JSON-only?

Options:

- A) `debug.extensions: map<string, json>` with per-extension `schema_version` inside the value (most flexible).
- B) Typed extensions in the core schema (best ergonomics, but schema churn).

Recommendation (v1):

- Choose A. Reserve B for a small set of “core invariants” only if they become universally needed.

---

## Q3 — Layout sidecars: what is the minimal portable artifact?

Candidate payload shapes:

- “Taffy subtree dump JSON” (native-only, best-effort).
- A smaller “layout explainability summary” keyed by `test_id` bounds deltas (portable).

Open question:

- What is the smallest payload that actually reduces time-to-fix without adding perf cliffs?

---

## Q4 — How do we correlate semantics nodes to layout nodes?

Today:

- scripts primarily target semantics (`test_id`),
- layout engine nodes are `NodeId`-based and can be ephemeral across runs.

Options:

- A) add a debug-only mapping table (semantics node id → NodeId) when available,
- B) treat correlation as “best effort” via element paths/labels and keep sidecars optional.

---

## Q5 — Should “layout correctness gates” live purely in the script protocol?

Options:

- A) stay script-protocol-first (predicates about bounds, stability, containment).
- B) add dedicated tooling-only gates (post-process bundle, compute layout invariants offline).

Recommendation (v1):

- Start with A for determinism and reviewability.
- Add B only for heavy analysis that is impractical in-app.

---

## Q6 — Transport parity: what differences are acceptable?

We want FS and WS to behave the same.

However, some differences may remain:

- filesystem transport can rely on local paths directly,
- web runner requires host-side artifact materialization and chunking.

Open question:

- what is the minimal “parity contract” the project treats as non-negotiable?

---

## Q7 — Security stance for DevTools WS (local-only defaults)

Open questions:

- do we require an origin allowlist for browser clients by default?
- should we rotate tokens per session/run automatically?
- do we need a “safe mode” that disables potentially sensitive payloads (e.g. text) by default?

