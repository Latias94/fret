# MVU Status (Archived)

Last updated: 2026-03-05

This file is kept for historical context only.

## Summary

- The repository golden path is **View runtime + typed actions** (ADRs 0308/0307), with payload
  actions v2 (ADR 0312) for pointer/programmatic parameterization.
- Legacy MVU authoring surfaces have been **hard-deleted in-tree** as part of milestone M9 in the
  action-first authoring fearless refactor v1 workstream.

## Migration notes

If you have an external codebase that still uses MVU patterns, keep the migration guidance in:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`

This repo intentionally does not document MVU as an available authoring path anymore.
