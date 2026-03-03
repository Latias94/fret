# Diagnostics Architecture (Fearless Refactor v1) — Crate Plan

Last updated: 2026-03-02

This document defines the crate boundary plan for the diagnostics stack:

- which crates are involved,
- what the “do not cross” boundaries are,
- and what “extensibility” should mean for ecosystem crates.

This is a workstream note. Hard boundaries must be backed by ADRs.

---

## 1) Boundary stance (do not violate)

Non-negotiable:

- `crates/fret-diag-protocol` remains portable (native + wasm32) and contains only versioned serde types.
- `crates/fret-diag` remains tooling-focused (host-side), not a runtime dependency for apps.
- `crates/fret-ui` remains mechanism/contract-only; policy belongs in ecosystem crates.

---

## 2) Crates (today) and their roles

### 2.1 `crates/fret-diag-protocol`

Owns:

- JSON schema types (selectors, predicates, scripts, results, transport envelopes).

Must:

- compile for wasm32,
- be forward-compatible (unknown fields ignored by default, schema versions explicit).

### 2.2 `crates/fret-diag-ws`

Owns:

- WebSocket transport server/client implementation.

Must:

- keep protocol payloads as `fret-diag-protocol` values,
- avoid embedding policy (do not reimplement lint/triage logic here).

### 2.3 `crates/fret-diag`

Owns:

- tooling engine used by CLI (`fretboard`) and DevTools GUI,
- artifact materialization, packing, lint/triage, compare, perf gates.

Must:

- be transport-agnostic (FS vs WS should be a detail),
- preserve artifact invariants and stable reason codes.

### 2.4 `ecosystem/fret-bootstrap` (runtime capture/export)

Owns:

- in-app diagnostics service (bundle export, scripts execution, inspector/picker),
- “golden path” wiring for demos/apps through `UiAppDriver`.

Must:

- keep per-frame overhead bounded,
- keep snapshot schemas stable and versioned (or placed behind extension keys).

---

## 3) Proposed module plan (refactor within crates first)

Before splitting crates, we should make `crates/fret-diag` modular and stable by internal boundaries:

- `transport/*` (logical operations over FS/WS)
- `artifacts/*` (manifest, chunking, integrity, materialization)
- `bundle/*` (schema2 preference, meta extraction, streaming reads)
- `script/*` (push/run/suite/repeat orchestration)
- `checks/*` (lint/triage/hotspots/perf gates)
- `pack/*` (zip + ai-packet)
- `compare/*` (diff + reports)
- `registry/*` (suite/check/script registries)

Exit criteria:

- adding a new suite/check should not require touching unrelated modules,
- the public API surface inside the crate stays small and intentional.

Notes (as of 2026-03-03):

- Initial `registry/*` scaffolding exists at:
  - `crates/fret-diag/src/registry/suites.rs`
  - `crates/fret-diag/src/registry/checks/mod.rs`
  - builtin post-run check implementations currently live in `crates/fret-diag/src/registry/checks/builtin_post_run.rs`

---

## 4) Ecosystem extensibility (contract shape)

### 4.1 Extensions slot (recommended)

Introduce a bounded optional slot in runtime debug snapshots:

- `debug.extensions: map<string, json>`

Rules:

- extension keys must be namespaced and versioned (e.g. `fret.docking.interaction.v1`),
- each payload includes its own `schema_version`,
- payloads are bounded and report clipping.

### 4.2 Capabilities gating (required)

If a script or tool depends on an extension:

- the dependency must be declared as a required capability (script meta),
- missing capability must fail fast with a stable `reason_code` (no timeouts).

---

## 5) New crates (not recommended for v1)

We should avoid splitting new crates until internal module boundaries hold.

If a split becomes necessary, candidates are:

- `fret-diag-engine` (transport + artifact materialization core),
- `fret-diag-checks` (lint/hotspots/perf gates),
- `fret-diag-pack` (zip + ai packet).

Split trigger conditions:

- compile times or dependency trees become a bottleneck,
- ownership boundaries become unclear inside one crate,
- or we need to embed the engine into another process in a “library first” way.
