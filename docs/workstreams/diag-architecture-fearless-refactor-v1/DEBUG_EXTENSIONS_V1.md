# UI diagnostics debug extensions v1

Status: Draft (workstream design note)

This note defines the **runtime snapshot extension seam** tracked by M2 in this workstream:
`UiDiagnosticsSnapshotV1.debug.extensions`.

It is intended to let ecosystem crates contribute small, debug-oriented signals without widening
the core typed snapshot schema on every iteration.

For the long-lived contract, see ADR 0310:

- `docs/adr/0310-ui-diagnostics-debug-extensions-v1.md`

## Scope

In scope for v1:

- A per-snapshot `debug.extensions` map.
- Init-time registration of best-effort writers (Option A).
- Bounded capture budgets with explicit clipping markers.
- Stable key naming rules so tooling can index/view extensions.

Out of scope for v1:

- A polished GUI viewer (raw JSON browsing is fine initially).
- Large payloads (use sidecars for those).
- Script steps that “require” extensions (extensions are best-effort evidence by default).

## Contract (v1)

### Location

`extensions` lives under the existing debug snapshot:

- `UiDiagnosticsSnapshotV1.debug.extensions`

### JSON shape

- `extensions`: optional map (key → JSON value)

Tooling MUST ignore unknown keys. Runners MUST write stable keys when present.

### Key naming rules

Keys MUST:

- be lowercase,
- be dot-separated,
- end with `.vN` (e.g. `dock.graph.v1`, `diag.runtime.v1`).

Guidance:

- prefer `area.topic.vN` (not “demo-specific” names),
- if the payload is tied to a specific crate, prefix with that crate conceptually (e.g. `dock.*`).

### Budgets and clipping

Extensions are bounded best-effort debug payloads:

- per-extension budget (bytes),
- total extensions budget per snapshot (bytes).

If a writer returns a payload that exceeds the per-extension budget, the runtime replaces it with a
small clipping marker object (and accounts the original size against the total budget).

If the total budget would be exceeded, the runtime stops capturing further extensions.

## How ecosystem crates register extensions (Option A)

Preferred model: register at init time via `fret-bootstrap`.

Example (pseudo-code):

- call `fret_bootstrap::ui_diagnostics::register_debug_extension_best_effort(app, "my.feature.v1", writer)`
- or use `BootstrapBuilder::register_diag_debug_extension(...)` when using the golden path.

Writers MUST:

- return `None` when the signal is not available,
- return small JSON values,
- avoid allocating per frame unless the value is actually present and meaningful.

## Recommended usage patterns

- Use `debug.extensions` for small “explain why” payloads (routing decisions, small summaries).
- Use bundle-scoped sidecars for large, high-volume dumps (layout trees, large traces).
- Keep extension payloads stable and versioned; if you need to evolve fields, bump the key suffix
  from `.v1` to `.v2`.

## Evidence anchors

- Registry + budgets: `ecosystem/fret-bootstrap/src/ui_diagnostics/extensions.rs`
- Snapshot field: `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_types.rs`
- Capture hook: `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
- Tooling viewer (CLI): `crates/fret-diag/src/commands/extensions.rs` (`fretboard diag extensions ...`)
