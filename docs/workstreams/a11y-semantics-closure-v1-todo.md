# A11y semantics closure (v1) — TODO

Last updated: 2026-02-23

## P0: Pressed semantics (toggle buttons)

- [x] Confirm AccessKit surface:
  - property shape (`pressed` / `toggled` / role differentiation) and whether “mixed” is supported.
  - action surface expectations (Invoke only vs explicit Toggle).
- [x] Add portable contract in `crates/fret-core` (additive, validated where appropriate).
- [x] Add `fret-ui` writers:
  - declarative `Pressable` / shadcn `Toggle` / `ToggleGroup` publish the pressed semantics.
- [x] Add `fret-a11y-accesskit` mapping + unit tests.
- [x] Add shadcn snapshot gate(s) asserting pressed semantics for:
  - a single toggle button,
  - a toggle group item.
- [x] Ensure diagnostics snapshots/fingerprint include the new field(s) if they affect determinism.

## P0: Required + invalid semantics (forms)

- [x] Decide contract shape:
  - `required: bool` vs `Option<bool>` (unknown vs false),
  - `invalid: bool` vs richer invalid reason (v1 should stay mechanism-only).
- [x] Map into AccessKit (if supported) and document fallbacks.
- [x] Adopt in shadcn primitives:
  - input / textarea / select / checkbox (at least one).
- [x] Gate via shadcn semantics snapshots and/or a diag script.

## P0: Busy semantics (loading)

- [ ] Decide contract shape (node flag vs extra field) and how it composes with progress numeric semantics.
- [ ] Map into AccessKit where possible.
- [ ] Adopt in at least one shadcn component (e.g. command loading state or spinner wrapper).
- [ ] Gate via snapshot/diag script.

## Hygiene

- [x] Add/refresh ADR(s) for any new hard-to-change surfaces (ADR 0290).
- [x] Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with evidence anchors once a surface is closed.
