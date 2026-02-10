# Fret Interaction Kernel (v1) — Milestones

## M0 — API + contracts (1–2 days)

Deliverables:

- `ecosystem/fret-interaction` crate exists and builds in the workspace.
- Public types are documented with explicit coordinate conventions and ownership boundaries.
- A single math source of truth is locked:
  - viewport / pan-zoom mapping is canonical in `ecosystem/fret-canvas` (no duplicate mapping math in
    `fret-interaction`).
- Unit tests exist for kernel primitives (state machines + threshold/DPI helpers).

Exit criteria:

- `cargo test -p fret-interaction` passes.
- The workstream doc + TODO doc reflect the chosen boundaries.

## M1 — `imui` floating windows (3–5 days)

Deliverables:

- `imui` floating window drag/resize/activation uses `fret-interaction`.
- Fractional DPI behavior stays correct:
  - title bar does not spill into body,
  - wrapped body text does not overlap following items.

Exit criteria:

- `cargo nextest run -p fret-ui-kit` passes.
- `fretboard diag run` gates are added/updated for the floating windows demo.

## M2 — `fret-node` viewport helpers (2–4 days)

Deliverables:

- `fret-node` continues to use the canonical `fret-canvas` viewport transform helpers without
  changing external behavior.
- Existing viewport conformance tests remain meaningful and pass.

Exit criteria:

- `cargo nextest run -p fret-node` passes.
- No new drift is introduced in XyFlow parity surfaces.

## M3 — docking / multi-window parity touchpoints (time-boxed)

Deliverables:

- Identify and unify the minimum required primitives:
  - drag capture choreography (transparent moving window),
  - hit-test/hover arbitration hooks.

Exit criteria:

- A diag repro exists in `tools/diag-scripts/` that guards the parity behavior under a deterministic script.
