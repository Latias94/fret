# Fretboard Public App-Author Surface v1 — Final Status

Status: Closed
Date: 2026-04-09

This lane is complete.

It answered the product-taxonomy question for the Fret CLI surface:

- public `fretboard` v1 teaches only project-agnostic app-author workflows:
  - `new`
  - `assets`
  - `config`
- repo-only `fretboard-dev` retains mono-repo maintainer workflows:
  - `dev`
  - `diag`
  - `hotpatch`
  - `list`
  - `theme`
- future public `dev` remains a project-facing follow-on, not a demo-registry surface
- future public diagnostics remain part of the `fretboard` product, but only as a reduced
  app-author core
- hotpatch stays repo-only for v1 and may only re-enter later as `fretboard dev native --hotpatch`
- `theme import-vscode` stays off public `fretboard`; if it becomes public later, it should move as
  a dedicated package around `fret-vscode-theme`

## Why the lane can close

The done condition for this lane was:

- the public `fretboard` product story is explicit,
- repo-only `fretboard-dev` ownership is explicit,
- and future work can proceed as narrow implementation follow-ons instead of reopening the product
  taxonomy.

That condition is now satisfied.

## Follow-on implementation work items

Future work should open narrow implementation lanes rather than extending this folder indefinitely.

Recommended follow-ons:

1. Public `dev` implementation follow-on
   - move the project-facing `dev native` / `dev web` contract into publishable `fretboard`
   - keep repo-only demo/cookbook/gallery shortcuts on `fretboard-dev`

2. Public diagnostics implementation follow-on
   - publish the dependency closure needed for the reduced diagnostics core
   - wire `fretboard diag` around the frozen public-core verbs without copying the full repo tree

3. Dedicated theme import packaging follow-on
   - if external demand exists, publish a focused CLI/package around `fret-vscode-theme`
   - do not widen the main `fretboard` product for this sidecar utility

4. ADR/docs alignment maintenance
   - keep root docs and ADRs aligned with the frozen public/private/package split
   - treat old wording that implies public `fretboard dev/diag/hotpatch/theme` as historical unless
     re-landed by a follow-on implementation lane

## Authoritative docs

- `docs/workstreams/fretboard-public-app-author-surface-v1/README.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/DESIGN.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/DIAG_TARGET_INTERFACE_STATE.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/HOTPATCH_TARGET_INTERFACE_STATE.md`
- `docs/workstreams/fretboard-public-app-author-surface-v1/THEME_TARGET_INTERFACE_STATE.md`

## Evidence anchors

- `docs/adr/0106-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- `docs/README.md`
- `docs/crate-usage-guide.md`
- `crates/fretboard/src/cli/contracts.rs`
- `apps/fretboard/src/cli/contracts.rs`
