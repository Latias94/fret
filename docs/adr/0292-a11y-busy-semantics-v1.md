# ADR 0292: A11y Busy Semantics (v1)

Status: Accepted

## Context

Many UI surfaces can enter a transient “loading / updating” state (async fetches, recomputation, paging) where assistive
technologies should be able to understand that a region is **busy**, without relying on ad-hoc text like “Loading…”.

ARIA models this via `aria-busy` on a container/region. AccessKit exposes a portable `busy` flag that maps into platform
accessibility APIs.

## Goals

1. Add a portable, mechanism-level representation of “busy/loading”.
2. Keep the surface additive and low-policy (components decide when a region is busy).
3. Map the flag into AccessKit consistently.
4. Provide at least one ecosystem adoption + regression gate (shadcn command list first).

## Non-goals (v1)

- Modeling progress values (use `SemanticsRole::ProgressBar` + `SemanticsNodeExtra.numeric` for determinate progress).
- Defining “busy” propagation rules (e.g. whether busy implies disabling actions): policy-layer concern.
- Live region announcements for loading state (separate contract).

## Decision

### D1 — Extend `SemanticsFlags` with `busy`

Add a new portable flag:

- `SemanticsFlags.busy: bool` (default `false`)

This is intended to be set on a region/container that is currently busy, typically covering a subtree.

### D2 — AccessKit mapping

When `busy == true`, map into AccessKit:

- `busy == true` → `Node::set_busy()`

### D3 — Ecosystem adoption (shadcn command palette/list)

The shadcn command list/palette should mark the listbox as busy when a loading row is present:

- `CommandLoading` row exists → listbox `SemanticsFlags.busy = true`

## Evidence (implementation)

- Contract: `crates/fret-core/src/semantics.rs` (`SemanticsFlags.busy`)
- UI writers:
  - `crates/fret-ui/src/widget.rs` (`SemanticsCx::set_busy`)
  - `crates/fret-ui/src/declarative/host_widget/semantics.rs` (applies `SemanticsDecoration.busy` / `SemanticsProps.busy`)
- AccessKit mapping + test: `crates/fret-a11y-accesskit/src/{mapping.rs,tests.rs}`
- Ecosystem adoption + gate:
  - `ecosystem/fret-ui-shadcn/src/command.rs` (listbox marks busy when loading rows exist)
  - shadcn snapshot gate: `ecosystem/fret-ui-shadcn/tests/snapshots/command_list_busy_semantics.json`
- Diagnostics snapshot field + fingerprint:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/semantics.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`

## Alternatives considered

1. **Only render a “Loading…” text row without any structured semantics.**
   - Pros: no contract change.
   - Cons: screen readers and automation cannot reliably treat the region as busy; localization/string parsing pitfalls.
2. **Model busy as a role.**
   - Pros: potentially simpler for some widgets.
   - Cons: “busy” is an orthogonal state that composes with many roles; it belongs in flags.

