---
title: "ADR 0310: UI Diagnostics Debug Extensions v1"
---

# ADR 0310: UI Diagnostics Debug Extensions v1

Status: Proposed

Related:

- ADR 0159 (bundles + scripts baseline): `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- ADR 0189 (capabilities + extensibility): `docs/adr/0189-ui-diagnostics-extensibility-and-capabilities-v1.md`
- ADR 0036 (inspector + observability hooks): `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`

## Context

Fret’s diagnostics bundles are intentionally artifact-first: we want portable evidence that can be
packed, shared, linted, and diffed. At the same time, many useful debug signals are:

- highly app- or ecosystem-specific (docking, canvas/node-graph, markdown editor),
- not stable enough to widen the core typed bundle schema for,
- small enough to be embedded directly in the per-frame snapshot without needing a dedicated sidecar.

Today, new signals often land by widening `UiTreeDebugSnapshotV1` directly. This creates churn and
makes it harder for ecosystem crates to contribute diagnostics without touching central wiring.

We need an explicit, bounded extension seam.

## Decision

### 1) Add a first-class `debug.extensions` slot to the diagnostics snapshot

`UiDiagnosticsSnapshotV1.debug` MUST include an optional `extensions` field:

- type: `map<string, json>`
- semantics: best-effort, additive, ignored by tooling when unknown

This is an **ecosystem extension seam** for small debug-oriented payloads.

### 2) Keys are versioned and namespace-like

Extension keys MUST:

- be lowercase,
- be dot-separated,
- end with a version suffix `.vN` (e.g. `dock.graph.v1`).

Rationale:

- keys become stable “contracts” for tooling and scripts,
- `.vN` allows independent evolution without breaking existing consumers.

### 3) Payloads are bounded (budget + clipping)

Extensions MUST remain small. The runtime MUST enforce:

- per-extension byte budget (best-effort),
- total extensions byte budget per snapshot (best-effort),
- clipping markers when a payload is too large.

Large payloads MUST use bundle-scoped sidecars instead (e.g. `layout.taffy.v1.json`).

### 4) Registration model: Option A (init-time closure registry)

The runtime MUST support registering extension writers at init time:

- writer signature: `(app, window) -> Option<json>`
- writers are best-effort and MUST NOT fail the run when missing.

This keeps the mechanism in `fret-bootstrap` and lets ecosystem crates contribute without
monolith edits.

### 5) Capability advertisement

Runners that support `debug.extensions` SHOULD advertise a capability string:

- `diag.debug_extensions_v1`

Tooling MAY use this for “fail-fast” expectations (but missing extensions is not a hard failure by
default).

## Consequences

Pros:

- Ecosystem crates can add diagnostics cheaply.
- Tooling can evolve viewers incrementally (CLI first, GUI later).
- Keeps the core typed snapshot schema stable while allowing additive growth.

Cons / tradeoffs:

- JSON payloads are less type-safe than dedicated structs.
- Budgets/clipping require ongoing tuning as we learn real-world sizes.

## Implementation notes (non-normative)

Reference implementation lives in:

- Runtime registry + capture budgets: `ecosystem/fret-bootstrap/src/ui_diagnostics/extensions.rs`
- Snapshot field: `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_types.rs` (`debug.extensions`)
- Capture hook: `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs` (assign `debug.extensions`)

