# Crate Audit (L1) — `fret-ui`

Status: L1 complete (targeted deep dive + one new regression gate)

Supersedes: `docs/workstreams/crate-audits/fret-ui.l0.md` (keep for the initial snapshot)

## Purpose

Mechanism-only UI runtime substrate: tree synthesis, layout, dispatch, semantics, overlay layers, hit testing,
and cache/invalidation behavior. This crate must remain policy-free (ADR 0066).

## Audit focus (L1)

- Overlay dismissal + routing mechanism correctness (Escape, outside-press observer pass).
- Focus-scoped routing invariants (focus scopes + traversal availability).

## What changed (evidence-backed)

- Added a regression gate ensuring Escape dismisses only the topmost dismissible overlay.
  - Evidence:
    - `crates/fret-ui/src/tree/tests/escape_dismiss.rs` (`escape_dismisses_only_the_topmost_overlay`)

## Hazards (top)

- Escape dismissal drift (wrong overlay dismissed; dismissing multiple overlays).
  - Existing gates:
    - `crates/fret-ui/src/tree/tests/escape_dismiss.rs`
    - `crates/fret-ui/src/tree/tests/dock_drag.rs` (Escape cancels dock drags without dismissing overlays)
- Outside-press observer pass drift (click-through, transforms, touch delay/cancel, drag suppression).
  - Existing gates:
    - `crates/fret-ui/src/tree/tests/outside_press.rs`
- Focus scope/traversal drift (trap focus, availability).
  - Existing gates:
    - `crates/fret-ui/src/tree/tests/focus_scope.rs`
    - `crates/fret-ui/src/tree/tests/focus_traversal_availability.rs`

## Recommended next steps

1. Turn one overlay-focused `fretboard diag` script into an “always-run” suite candidate (dialog escape + focus restore),
   and map it into the workstream guardrails.
2. Continue splitting `crates/fret-ui/src/tree/mod.rs` by responsibility only after gates exist for the hazard list.

