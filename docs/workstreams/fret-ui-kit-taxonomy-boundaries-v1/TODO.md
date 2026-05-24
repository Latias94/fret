# Fret UI Kit Taxonomy Boundaries v1 - TODO

Status: Active
Last updated: 2026-05-25

## UKT-M0 - Source Audit

- [ ] UKT-010 [owner=planner] [deps=none] [scope=ecosystem/fret-ui-kit/src]
  Goal: Map modules to style/headless/primitives/declarative/imui/recipes owner categories.
  Validation: Audit note added to this workstream.
  Evidence: `docs/workstreams/fret-ui-kit-taxonomy-boundaries-v1/HANDOFF.md`
  Handoff: Pick exactly one confused owner for the first code move.

## UKT-M1 - First Owner Move

- [ ] UKT-020 [owner=unassigned] [deps=UKT-010] [scope=ecosystem/fret-ui-kit/src]
  Goal: Move or rename one owner to match the taxonomy without behavior or public API change.
  Validation: `cargo check -p fret-ui-kit --features imui`
  Evidence: moved module and focused tests.
  Handoff: Keep public widening out of this slice.
