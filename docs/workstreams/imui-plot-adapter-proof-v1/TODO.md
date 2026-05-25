# IMUI Plot Adapter Proof v1 - TODO

Status: Closed
Last updated: 2026-05-25

## IPA-010 - Owner Lane And Boundary Decision

- [x] Create the active workstream docs.
- [x] Record that plot adapter work belongs in `fret-plot`, not `fret-imui` or
      `fret-ui-kit::imui`.
- [x] Keep the deleted retained plot facade out of scope.

## IPA-020 - Optional Declarative Adapter

- [x] Add `fret-plot/imui` as an opt-in feature.
- [x] Add `ecosystem/fret-plot/src/imui.rs` with `UiWriter` helpers over declarative plot panels.
- [x] Keep the default `fret-plot` feature set unchanged.
- [x] Add a source-policy test proving the adapter is opt-in and declarative-only.

## IPA-030 - Proof Gates

- [x] Run default and `imui` feature compile gates.
- [x] Run the focused source-policy test.
- [x] Run IMUI source-policy and workstream catalog gates.
- [x] Record fresh results in `EVIDENCE_AND_GATES.md`.

## IPA-040 - Deferred Product Adoption

- [x] Defer cookbook or canonical-workbench plot adapter usage until repeated authoring
      friction appears in product routes.
- [x] Keep root `fret::imui` plot sugar deferred until at least two product surfaces prove the same
      shorthand is needed.
