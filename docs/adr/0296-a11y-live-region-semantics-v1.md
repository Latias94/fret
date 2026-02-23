# ADR 0296: A11y Live Region Semantics (v1)

Status: Accepted

## Context

Many UI surfaces need to announce dynamic updates to assistive technologies without forcing authors to encode the
announcement into arbitrary label/value strings. Common examples include:

- toast/notification viewports,
- background status updates (“Saved”, “Connected”, “Build finished”),
- inline validation summaries.

ARIA models this via *live regions* (e.g. `aria-live` and `aria-atomic`). AccessKit provides a portable surface for live
region semantics.

## Goals

1. Add a portable, mechanism-level contract for live region semantics (`polite` / `assertive` / `off`).
2. Map the contract into AccessKit consistently.
3. Provide at least one ecosystem adoption and a regression gate.

## Non-goals (v1)

- A global “announce queue” API (policy-layer concern).
- Authoring a DOM-like `aria-relevant` surface or portal semantics (policy-layer concern).
- Automatically inferring live regions from widget roles or text content (policy-layer concern).

## Decision

### D1 — Extend `SemanticsFlags` with live region fields

Add two new portable fields:

- `SemanticsFlags.live: Option<SemanticsLive>`
  - `None` means “no live region semantics requested”.
  - `Some(Off|Polite|Assertive)` requests a specific live region mode.
- `SemanticsFlags.live_atomic: bool` (default `false`)
  - When `true`, updates to this live region should be presented atomically (ARIA `aria-atomic`-like outcome).

`SemanticsLive` is a small non-exhaustive enum:

- `Off`
- `Polite`
- `Assertive`

### D2 — AccessKit mapping

When `SemanticsFlags.live` is set, map into AccessKit:

- `SemanticsLive::{Off,Polite,Assertive}` → `Node::set_live(Live::{Off,Polite,Assertive})`
- `live_atomic == true` → `Node::set_live_atomic()`

If `live` is `None`, `live_atomic` is ignored by the AccessKit bridge.

### D3 — Ecosystem adoption (toast viewport)

The toast viewport overlay root should publish a polite live region, matching common notification announcement patterns:

- Toast viewport root → `SemanticsFlags.live = Some(Polite)`

Atomic announcements and richer policy (e.g. per-toast roles) remain policy-layer decisions and can evolve separately.

## Evidence (implementation)

- Contract: `crates/fret-core/src/semantics.rs` (`SemanticsLive`, `SemanticsFlags.live/live_atomic`)
- UI writers:
  - `crates/fret-ui/src/widget.rs` (`SemanticsCx::{set_live,set_live_atomic}`)
  - `crates/fret-ui/src/element.rs` (`SemanticsProps` + `SemanticsDecoration` surfaces)
  - `crates/fret-ui/src/declarative/host_widget/semantics.rs` (writer plumbing)
- AccessKit mapping + test: `crates/fret-a11y-accesskit/src/{mapping.rs,tests.rs}`
- Ecosystem adoption: `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (toast viewport semantics decoration)
- Regression gate: `ecosystem/fret-ui-shadcn/tests/snapshots/live_region_semantics.json`
- Diagnostics snapshot field + fingerprint:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/semantics.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`

## Alternatives considered

1. **Model announcements as a dedicated `announce(text)` action in `fret-core`.**
   - Pros: “most direct” authoring surface.
   - Cons: policy-heavy (queueing, dedupe, timing, focus interaction); hard to keep portable and deterministic.
2. **Keep live region semantics ecosystem-only (string-only behavior).**
   - Pros: avoids contract expansion.
   - Cons: platform bridges and diagnostics cannot represent announcements consistently; risks long-term drift.

