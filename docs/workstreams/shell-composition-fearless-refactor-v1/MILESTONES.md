# Shell Composition Fearless Refactor v1 (Milestones)

## Status summary

- `M0` Baseline and owner inventory: **Completed**
- `M1` Contract freeze: **Completed**
- `M2` Surface naming cleanup: **Completed**
- `M3` Page-shell promotion decision: **Completed**
- `M4` Workspace-shell consolidation: **Planned**
- `M5` Teaching-surface cleanup: **Completed**
- `M6` Gates: **Planned**

## M0 — Baseline and owner inventory

**Status:** Completed

**What closed**

- Established that Fret already has three distinct shell families:
  - window bootstrap,
  - page shell,
  - workspace shell.
- Confirmed that current docs already reject pushing shell policy into `crates/fret-ui`.
- Confirmed that Zed and GPUI component do not justify introducing one universal `AppShell`.

**Evidence**

- `docs/workspace-shell.md`
- `docs/crate-usage-guide.md`
- `apps/fret-cookbook/src/scaffold.rs`
- `ecosystem/fret-workspace/src/{lib.rs,frame.rs,layout.rs}`
- `repo-ref/zed/crates/workspace/src/workspace.rs`
- `repo-ref/gpui-component/crates/ui/src/{lib.rs,root.rs,title_bar.rs}`

## M1 — Contract freeze

**Status:** Completed

**What closed**

- Added a dedicated fearless-refactor lane for shell composition.
- Froze the three-way owner split:
  - window bootstrap,
  - page shell,
  - workspace shell.
- Froze the rule that this lane does not preserve compatibility aliases by default.
- Froze the decision that Fret should not introduce a universal `AppShell`.

**Evidence**

- `docs/workstreams/shell-composition-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/shell-composition-fearless-refactor-v1/TODO.md`
- `docs/workstreams/shell-composition-fearless-refactor-v1/MILESTONES.md`

## M2 — Surface naming cleanup

**Status:** Completed

**What closed**

- Renamed the misleading `workspace_menu` lane to `in_window_menubar`.
- Updated first-party callers and source-audit tests to consume the new module path.
- Added root-surface source-policy protection so the old module name does not quietly return.

**Evidence**

- `ecosystem/fret/src/in_window_menubar.rs`
- `ecosystem/fret/src/lib.rs`
- `apps/fret-ui-gallery/src/driver/menubar.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`

## M3 — Page-shell promotion decision

**Status:** Completed

**What closed**

- Audited the cookbook lesson shell, the UI Gallery docs scaffold, and the `todo_demo`
  responsive shell.
- Decided those first-party consumers are **not** aligned enough to justify a shared reusable
  page-shell surface.
- Kept the current page shells app-owned instead of inventing a false shared abstraction.
- Froze the promotion gate: require at least three aligned first-party consumers with materially
  similar layout semantics before promoting a reusable owner surface.

**Evidence**

- `docs/workstreams/shell-composition-fearless-refactor-v1/PAGE_SHELL_AUDIT_2026-04-02.md`
- `apps/fret-cookbook/src/scaffold.rs`
- `apps/fret-cookbook/src/lib.rs`
- `apps/fret-ui-gallery/src/ui/doc_layout.rs`
- `apps/fret-examples/src/todo_demo.rs`

## M4 — Workspace-shell consolidation

**Status:** Planned

**Planned slice**

- Reconfirm that editor/workspace shell composition stays on `fret-workspace` and other explicit
  shell-aware owners.
- Remove any remaining facade drift that would pull workspace shell back into `fret`.
- Keep docking/workspace choreography out of mechanism crates.

**Target evidence**

- `ecosystem/fret-workspace/src/{lib.rs,frame.rs,layout.rs}`
- `ecosystem/fret/src/lib.rs`
- `docs/workspace-shell.md`

## M5 — Teaching-surface cleanup

**Status:** Completed

**What closed**

- Update docs so the shell split is visible to contributors and app authors.
- Ensure first-party examples teach startup window policy separately from interior shell
  composition.
- Prevent docs from teaching app-local shell helpers as if they were already stable framework
  contracts.
- Sweep historical docs/workstreams that still point to deleted workspace-shell facade paths as if
  they were current owner surfaces.

**Evidence**

- `docs/README.md`
- `docs/crate-usage-guide.md`
- `docs/examples/README.md`

## M6 — Gates

**Status:** Planned

**Planned slice**

- Add source-policy protection for shell owner boundaries.
- Add a first-party usage gate if a shared page shell is promoted.
- Add a guard against old shell names reappearing after renames.

**Target evidence**

- source-policy tests in owning crates
- docs/source audits for renamed surfaces
