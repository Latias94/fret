# Shell Composition Fearless Refactor v1 — TODO

This file is the execution checklist for `DESIGN.md`.

## M0 — Baseline and owner inventory

- [x] Confirm the current shell families:
  - [x] window bootstrap in `fret-launch` / `fret-bootstrap` / `fret`
  - [x] page-shell helpers in app-owned surfaces such as `apps/fret-cookbook`
  - [x] workspace-shell building blocks in `ecosystem/fret-workspace`
- [x] Confirm current docs already point away from shell-in-runtime drift:
  - [x] `docs/workspace-shell.md`
  - [x] `docs/crate-usage-guide.md`
  - [x] `ecosystem/fret/src/lib.rs` source-policy tests
- [x] Audit external reference shapes:
  - [x] Zed uses an explicit workspace owner
  - [x] GPUI component provides shell-capable building blocks, not a universal shell

## M1 — Contract freeze

- [x] Add this workstream folder with:
  - [x] `DESIGN.md`
  - [x] `TODO.md`
  - [x] `MILESTONES.md`
- [x] Freeze the no-universal-`AppShell` decision.
- [x] Freeze the three-way owner split:
  - [x] window bootstrap
  - [x] page shell
  - [x] workspace shell
- [x] Freeze the breaking-policy rule for this lane:
  - [x] no compatibility aliases by default
  - [x] delete stale names once first-party callers migrate

## M2 — Surface naming cleanup

- [ ] Audit all shell-like names on the `fret` facade and direct ecosystem callers.
- [ ] Rename misleading generic shell names:
  - [ ] `workspace_menu` should migrate to a neutral menu-bridge name
- [ ] Update docs and first-party callers in the same slice.
- [ ] Add a source-policy test that prevents old shell names from returning on the root facade.

## M3 — Page-shell promotion decision

- [ ] Inventory first-party page-shell helpers and consumers:
  - [ ] `apps/fret-cookbook/src/scaffold.rs`
  - [ ] `apps/fret-ui-gallery/src/ui/doc_layout.rs`
  - [ ] user-facing demos such as `todo_demo`
- [ ] Decide whether at least three aligned consumers exist.
- [ ] If the answer is **no**:
  - [ ] keep helpers app-owned,
  - [ ] document the promotion rule explicitly,
  - [ ] avoid premature framework surface growth.
- [ ] If the answer is **yes**:
  - [ ] choose one owning ecosystem lane,
  - [ ] move the helper there without leaving a compatibility alias,
  - [ ] keep it off the default `fret` prelude/root shortcuts.

## M4 — Workspace-shell consolidation

- [ ] Audit whether any editor/workspace shell composition still leaks through `fret`.
- [ ] Keep editor/workspace shell assembly on `fret-workspace` or other shell-aware owning crates.
- [ ] Keep docking-specific shell choreography on docking/workspace owners, not runtime/UI
  mechanism crates.
- [ ] Verify `docs/workspace-shell.md` and this workstream tell the same story.

## M5 — Teaching-surface cleanup

- [x] Update `docs/README.md` to point to this lane.
- [ ] Update app-facing docs if a shell surface moves or is renamed:
  - [ ] `docs/crate-usage-guide.md`
  - [ ] `docs/examples/README.md` if needed
- [ ] Ensure first-party examples teach the final shell split:
  - [ ] startup window policy at the builder layer,
  - [ ] interior page shell in app-facing composition,
  - [ ] workspace shell on `fret-workspace`.

## M6 — Gates

- [ ] Add or update at least one source-policy gate for shell owner boundaries.
- [ ] If a shared page shell is promoted, add a small first-party usage gate proving the chosen
  owner crate is actually used by multiple consumers.
- [ ] If a menu-bridge rename lands, add a docs/source audit so old names do not quietly return.

## Notes

- This lane prefers deleting wrong surfaces over preserving compatibility.
- This lane should not introduce a mega-crate just to “group shells”.
- This lane should not widen `crates/fret-ui` to solve ownership confusion.
