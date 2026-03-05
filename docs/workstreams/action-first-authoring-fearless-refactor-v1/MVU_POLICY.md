# MVU Status (Compatibility Surface; Planned Deprecation/Removal)

Last updated: 2026-03-06

This file documents the current MVU stance during the action-first authoring refactor. It should
remain short and policy-focused.

## Summary

- The repository golden path is **View runtime + typed actions** (ADRs 0308/0307), with payload
  actions v2 (ADR 0312) for pointer/programmatic parameterization.
- MVU authoring still exists in-tree today as a compatibility surface, but it is not the recommended
  authoring path for new code.
- Planned sequence (subject to exit gates):
  - **M8**: MVU deprecation window (warn + migrate).
  - **M9**: hard delete MVU in-tree (remove modules, templates/docs cleanup, add a regression gate).

## Migration notes

If you have an external codebase that still uses MVU patterns, keep the migration guidance in:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`

This repo intentionally does not document MVU as an available authoring path anymore.

Once MVU is removed in-tree (M9), this file should be archived to retain the historical policy
context without keeping MVU discoverable as a supported surface.
