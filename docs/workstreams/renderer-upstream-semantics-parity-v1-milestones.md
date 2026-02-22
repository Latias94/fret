# Renderer Upstream Semantics Parity v1 — Milestones

## M0 — Workstream ready

Completion criteria:

- A repeatable parity-note template exists.
- One seam is selected with evidence anchors for both upstream and Fret.

## M1 — One seam closed (guarded)

Completion criteria:

- At least one parity note is written (scissor spaces recommended).
- If a gap is found, at least one guardrail is added (validator/test/conformance).
- The guardrail is tied to a local evidence anchor.

## M2 — Two more seams analyzed

Completion criteria:

- Clip/mask composition parity note exists.
- Intermediate reuse/lifetime parity note exists.
- Each note has a “gap vs deliberate difference” call.

## M3 — Refactor-ready summary

Completion criteria:

- A short summary lists:
  - seams that are safe to refactor internally (guarded),
  - seams that require ADR changes (contract work),
  - and seams that are intentionally divergent (documented rationale).

