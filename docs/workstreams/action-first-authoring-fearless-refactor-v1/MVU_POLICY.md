# MVU Status (Removed In-Tree; Historical Migration Context Only)

Last updated: 2026-03-06

This file records the final MVU policy after the action-first authoring refactor. It remains
short and policy-focused so future cleanup work has a stable historical note.

## Summary

- The repository golden path is **View runtime + typed actions** (ADRs 0308/0307), with payload
  actions v2 (ADR 0312) for pointer/programmatic parameterization.
- In-tree MVU modules, demo routing, and scaffolding were removed as part of M9.
- Remaining MVU discussion in this repo is historical/external migration context only.
- Guardrails: `tools/gate_no_mvu_in_tree.py` and `tools/gate_no_mvu_in_cookbook.py` prevent reintroduction.

## Migration notes

If you have an external codebase that still uses MVU patterns, keep the migration guidance in:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`

This repo intentionally does not document MVU as an available authoring path anymore.

Keep this file as a short archival policy note so future contributors can see why MVU references
may still appear in historical docs while the code surface remains removed.
