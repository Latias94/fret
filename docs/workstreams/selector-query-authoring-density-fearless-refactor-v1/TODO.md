# Selector / Query Authoring Density (Fearless Refactor v1) — TODO

Status: active
Last updated: 2026-03-20

Companion docs:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `SELECTOR_BORROWED_INPUT_AUDIT_2026-03-20.md`

## Current checklist

- [x] Confirm the lane is justified by non-Todo evidence.
- [x] Freeze router as adjacent-only.
- [x] Split the problem into:
  - query semantic projection density,
  - selector borrowed-input density.
- [x] Land the first query semantic helper batch.
- [x] Adopt that batch on first-party proof surfaces.
- [x] Add tests/gates for the new posture.
- [x] Audit selector borrowed-input pressure before proposing a selector API change.

Execution note on 2026-03-20:

- the initial query batch is now landed on:
  - `ecosystem/fret-query/src/lib.rs`
  - `apps/fret-examples/src/query_demo.rs`
  - `apps/fret-examples/src/query_async_tokio_demo.rs`
  - `apps/fret-cookbook/examples/query_basics.rs`
  - `apps/fret-examples/src/async_playground_demo.rs`
  - `apps/fret-examples/src/lib.rs`
- that batch intentionally stops at semantic projections such as status text/predicates and
  refreshing detection
- selector borrowed-input density is now audited on a no-new-API verdict; reopen only with
  stronger non-Todo proof

## Standing rules

- [ ] No helper lands from Todo-only pressure.
- [ ] No batch is complete until code + docs + gates agree.
- [ ] No shadcn-specific policy leaks into `fret-query`.
- [ ] No router/history/link expansion enters this lane without fresh evidence.
