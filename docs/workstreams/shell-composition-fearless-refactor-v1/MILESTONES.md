# Shell Composition Fearless Refactor v1 (Milestones)

## Status summary

- `M0` Baseline and owner inventory: **Completed**
- `M1` Contract freeze: **Completed**
- `M2` Surface naming cleanup: **Planned**
- `M3` Page-shell promotion decision: **Planned**
- `M4` Workspace-shell consolidation: **Planned**
- `M5` Teaching-surface cleanup: **Planned**
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

**Status:** Planned

**Planned slice**

- Rename misleading generic shell surfaces such as `workspace_menu`.
- Keep neutral menu-bridge helpers opt-in and off the default prelude.
- Delete stale names in the same lane once first-party callers migrate.

**Target evidence**

- `ecosystem/fret/src/workspace_menu.rs` or its replacement
- `ecosystem/fret/src/lib.rs`
- source-policy tests that prevent old names from returning

## M3 — Page-shell promotion decision

**Status:** Planned

**Planned slice**

- Audit first-party page-shell helpers and consumers.
- Decide whether a reusable page-shell surface is actually justified.
- If justified, move it to one owning ecosystem lane without compatibility aliases.
- If not justified, keep page shell app-owned and document the promotion rule.

**Target evidence**

- `apps/fret-cookbook/src/scaffold.rs`
- `apps/fret-ui-gallery/src/ui/doc_layout.rs`
- first-party demo call sites such as `apps/fret-examples/src/todo_demo.rs`

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

**Status:** Planned

**Planned slice**

- Update docs so the shell split is visible to contributors and app authors.
- Ensure first-party examples teach startup window policy separately from interior shell
  composition.
- Prevent docs from teaching app-local shell helpers as if they were already stable framework
  contracts.

**Target evidence**

- `docs/README.md`
- `docs/crate-usage-guide.md`
- `docs/examples/README.md` if needed

## M6 — Gates

**Status:** Planned

**Planned slice**

- Add source-policy protection for shell owner boundaries.
- Add a first-party usage gate if a shared page shell is promoted.
- Add a guard against old shell names reappearing after renames.

**Target evidence**

- source-policy tests in owning crates
- docs/source audits for renamed surfaces
