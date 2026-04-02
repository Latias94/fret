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

- [x] Audit all shell-like names on the `fret` facade and direct ecosystem callers.
- [x] Rename misleading generic shell names:
  - [x] `workspace_menu` migrated to the neutral `in_window_menubar` lane
- [x] Update docs and first-party callers in the same slice.
- [x] Add a source-policy test that prevents old shell names from returning on the root facade.

## M3 — Page-shell promotion decision

- [x] Inventory first-party page-shell helpers and consumers:
  - [x] `apps/fret-cookbook/src/scaffold.rs`
  - [x] `apps/fret-ui-gallery/src/ui/doc_layout.rs`
  - [x] user-facing demos such as `todo_demo`
- [x] Decide whether at least three aligned consumers exist.
- [x] Close the current answer as **no** for v1.
- [x] Keep helpers app-owned:
  - [x] cookbook lesson shell stays on `apps/fret-cookbook`
  - [x] UI Gallery docs scaffold stays on `apps/fret-ui-gallery`
  - [x] `todo_demo` responsive shell stays demo-owned
- [x] Document the promotion rule explicitly.
- [x] Avoid premature framework surface growth by deferring shared promotion.
- [x] Record the audit artifact:
  - [x] `docs/workstreams/shell-composition-fearless-refactor-v1/PAGE_SHELL_AUDIT_2026-04-02.md`

## M4 — Workspace-shell consolidation

- [ ] Audit whether any editor/workspace shell composition still leaks through `fret`.
- [ ] Keep editor/workspace shell assembly on `fret-workspace` or other shell-aware owning crates.
- [ ] Keep docking-specific shell choreography on docking/workspace owners, not runtime/UI
  mechanism crates.
- [ ] Verify `docs/workspace-shell.md` and this workstream tell the same story.

## M5 — Teaching-surface cleanup

- [x] Update `docs/README.md` to point to this lane.
- [x] Update app-facing docs if a shell surface moves or is renamed:
  - [x] `docs/crate-usage-guide.md`
  - [x] `docs/examples/README.md` if needed
- [x] Sweep historical docs/workstreams that still teach deleted workspace-shell facade paths such
  as `fret::workspace_shell::*` or `ecosystem/fret/src/workspace_shell.rs` as active surfaces.
- [x] Ensure first-party examples teach the final shell split:
  - [x] startup window policy at the builder layer,
  - [x] interior page shell in app-facing composition,
  - [x] workspace shell on `fret-workspace`.

## M6 — Gates

- [ ] Add or update at least one source-policy gate for shell owner boundaries.
- [ ] If a shared page shell is promoted, add a small first-party usage gate proving the chosen
  owner crate is actually used by multiple consumers.
- [ ] If a menu-bridge rename lands, add a docs/source audit so old names do not quietly return.

## Notes

- This lane prefers deleting wrong surfaces over preserving compatibility.
- This lane should not introduce a mega-crate just to “group shells”.
- This lane should not widen `crates/fret-ui` to solve ownership confusion.
