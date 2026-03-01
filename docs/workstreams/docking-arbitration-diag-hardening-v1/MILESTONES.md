# Docking arbitration diag hardening (v1) — Milestones

## M1 — Deterministic cross-window drag-back

Goal: the “tear off a tab then drag it back to the main window” scenario runs green reliably in
`--launch` mode.

Deliverables:

- A stable script (schema v2) that:
  - tears off a tab into a new OS window,
  - repositions windows into a known geometry,
  - drags the torn-off tab back into the main window,
  - asserts a stable dock graph outcome.
- A bounded evidence bundle captured near the end of the scenario.

## M2 — Suite-level stability and isolation

Goal: the full `docking-arbitration` suite runs without cross-script contamination.

Deliverables:

- Launch-mode environment injection is per-script (no suite-level env leakage).
- Quarantined “known-flaky” cases are either fixed or explicitly gated (with a reason + link to evidence).
- Registry/index kept in sync; scripts are discoverable via `script_id` and suites.

## M3 — Contract + tooling closure

Goal: the diagnostics + runner contract for multi-window drags is explicit, testable, and
maintainable.

Deliverables:

- A short contract note documenting:
  - cross-window hover detection expectations,
  - drop routing semantics for scripted drags,
  - required invariants (no stuck drags, consistent window targeting).
- At least one runner-level regression test or diagnostics gate that fails fast on drift.

