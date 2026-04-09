# Iconify Import-Pack Generator v1 — TODO

Status: Active
Last updated: 2026-04-09

## Lane opening

- [x] IIPG-001 Open this as a dedicated follow-on lane instead of reopening
  `icon-system-extension-v1`.
- [x] IIPG-002 Record the shipped predecessor boundary and the assumptions for this narrower lane.

## M0 — Scope and evidence freeze

- [x] IIPG-010 Audit the current icon codegen/tooling surface:
  - `tools/gen_icons.py`
  - `tools/icon_codegen.py`
  - `tools/sync_icons.py`
  - `tools/verify_icons.py`
  - `tools/check_icons_generation.py`
  - Recorded in `BASELINE_AUDIT_2026-04-09.md`.
- [x] IIPG-011 Record the exact current generated-pack shape already implied by Lucide/Radix:
  - assets layout
  - generated ids/constants
  - `PACK_METADATA`
  - `PACK` / `VENDOR_PACK`
  - `app::install(...)`
  - Recorded in `BASELINE_AUDIT_2026-04-09.md`.
- [x] IIPG-012 Freeze the non-goals so this lane does not turn into another runtime icon surface
  redesign.
  - Locked in `DESIGN.md` and validated in `BASELINE_AUDIT_2026-04-09.md`.

## M1 — Generator contract freeze

- [x] IIPG-020 Decide the v1 input boundary:
  - Iconify-style collection snapshots,
  - local SVG files/directories,
  - alias/provenance config,
  - and whether remote fetch is ever allowed inside the generator itself.
  - Recorded in `M1_CONTRACT_FREEZE_2026-04-09.md`.
- [x] IIPG-021 Decide the v1 output boundary for a generated pack crate.
  - Recorded in `M1_CONTRACT_FREEZE_2026-04-09.md`.
- [x] IIPG-022 Decide where the stable reusable surface lives:
  - repo-local `tools/`,
  - a publishable CLI crate,
  - or a narrower library + CLI split.
  - Recorded in `M1_CONTRACT_FREEZE_2026-04-09.md`.
- [x] IIPG-023 Decide how generated packs classify provenance:
  - `Generated`,
  - `Vendored`,
  - or another additive import model distinction if needed.
  - Recorded in `M1_CONTRACT_FREEZE_2026-04-09.md`.

## M2 — Proof surface

- [ ] IIPG-030 Land one smallest proof surface that exercises the chosen generator boundary.
- [ ] IIPG-031 Prove that the generated output fits the current app/bootstrap install contract
  without manual cleanup steps.
- [ ] IIPG-032 Keep the out-of-tree / third-party developer story explicit rather than relying on
  monorepo-only assumptions.

## M3 — Gates and docs

- [ ] IIPG-040 Add one deterministic regression gate for generated-pack output.
- [ ] IIPG-041 Update user-facing docs once the generator contract is real.
- [ ] IIPG-042 Leave one follow-on-safe evidence set for future pack-specific expansion.

## Boundaries to protect

- Do not add a runtime Iconify client to the framework core.
- Do not reopen `SvgIcon` vs `SvgImage`.
- Do not move pack-specific policy into `crates/fret-ui`.
- Do not treat vendor curation/alias taste as part of the core generator contract unless the proof
  surface forces that conclusion.
