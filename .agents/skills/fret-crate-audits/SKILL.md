---
name: fret-crate-audits
description: "Crate-by-crate code-quality audits for fearless refactors: produce a lightweight audit note (purpose/exports/deps/hazards), run a small gate set, and turn findings into landable steps. Use when you want to review a crate for Rust best practices, contract surface hygiene, and refactor hazards."
---

# Fret crate audits (code-quality pass)

This skill is for the **second pass** of the Bottom-Up Fearless Refactor program: reading crates and
reviewing them for Rust best practices, boundary hygiene, and refactor hazards.

## Inputs

- Target crate name (package name), e.g. `fret-runtime`
- Optional focus (pick 1–2):
  - public surface (exports / accidental re-exports)
  - dependency posture (backend coupling risks)
  - hot paths (dispatch/layout/paint)
  - serialization formats / stability
  - error handling and diagnostics

## Quick start (L0 audit)

1. Generate a quick audit report (facts + evidence anchors):
   - `pwsh -NoProfile -File tools/audit_crate.ps1 -Crate fret-runtime`
2. Create (or update) a crate audit note using the template:
   - `docs/workstreams/bottom-up-fearless-refactor-v1-crate-audit-template.md`
3. Update the audits index:
   - `docs/workstreams/bottom-up-fearless-refactor-v1-crate-audits.md`

## Minimum gates (recommended)

- Format (touched crates only): `cargo fmt`
- Tests (touched crate): `cargo nextest run -p <crate>`
- Layering: `pwsh -NoProfile -File tools/check_layering.ps1`

If you touch hot paths, add at least one additional regression gate (unit/integration/diag).

## Output expectations

Prefer producing:

- 3–10 hazards + at least one missing gate to add
- 3–8 concrete, landable refactor steps
- 1–3 evidence anchors per major claim (paths + key functions/tests)

