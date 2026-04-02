# Shell Composition Fearless Refactor v1 — Design

Status: Draft

This workstream exists to freeze the ownership model for Fret's shell surfaces before more app
examples, workspace chrome, and window configuration helpers drift into each other.

It is intentionally willing to delete, move, or rename pre-release authoring surfaces when they
blur the correct layer boundary. This is not a compatibility-preservation lane.

## Problem statement

Fret already has several distinct shell-like surfaces, but they are not yet described as one
coherent model:

- **Window bootstrap** exists on `fret-launch`, `fret-bootstrap`, and `fret` builder surfaces
  (`window`, size constraints, startup position, resize increments, resizable flags).
- **Page shell** helpers exist in app-owned places such as
  `apps/fret-cookbook/src/scaffold.rs` and `apps/fret-ui-gallery/src/ui/doc_layout.rs`.
- **Workspace shell** building blocks already exist in `ecosystem/fret-workspace`.
- some historical naming blurred ownership, especially the old `fret::workspace_menu` lane for a
  generic in-window menubar bridge.

The result is predictable:

- user-facing demos mix startup window policy, page framing, and app content in one place,
- page-shell helpers are duplicated or stranded in app crates without a promotion rule,
- editor/workspace shell composition risks creeping back into the `fret` app-facing facade,
- misleading names make it harder to understand where future shell code should live.

The repo already documents the intended direction:

- `docs/workspace-shell.md` says the missing layer is a cohesive workspace shell, not a wider
  `fret-ui` runtime contract,
- `docs/crate-usage-guide.md` says editor/workspace shell composition should stay on owning crates
  such as `fret-workspace`,
- `ecosystem/fret/src/lib.rs` contains a source-policy test that keeps `workspace_shell` off the
  root `fret` facade.

This workstream turns those scattered conclusions into one explicit shell model.

## Goals

1. Freeze a canonical shell split for first-party and third-party authors:
   - window bootstrap,
   - page shell,
   - workspace shell.
2. Keep `crates/fret-ui` mechanism-only and keep shell policy out of runtime contracts.
3. Keep the default `fret` lane app-facing; do not let editor/workspace shell APIs creep back into
   the facade.
4. Prefer deletion over compatibility shims for pre-release shell surface cleanup.
5. Define when an app-local shell helper is allowed to graduate into a reusable ecosystem surface.
6. Identify misleading names and owner mismatches early enough to fix them once.

## Non-goals

- Introducing a universal `AppShell` trait, base struct, or mega-crate.
- Pixel-parity work on every first-party demo.
- Immediate promotion of every app-local shell helper into a reusable framework surface.
- Reworking runtime contracts that do not need to change for shell cleanup.

## References

- Fret shell direction: `docs/workspace-shell.md`
- App-facing facade guidance: `docs/crate-usage-guide.md`
- Current page-shell helper: `apps/fret-cookbook/src/scaffold.rs`
- Current workspace-shell building blocks: `ecosystem/fret-workspace/src/{lib.rs,frame.rs,layout.rs}`
- Current app facade source policy: `ecosystem/fret/src/lib.rs`
- Zed high-level owner model:
  - `repo-ref/zed/crates/workspace/src/workspace.rs`
  - `repo-ref/zed/crates/title_bar/src/title_bar.rs`
  - `repo-ref/zed/crates/panel/src/panel.rs`
- GPUI component building blocks:
  - `repo-ref/gpui-component/crates/ui/src/{lib.rs,root.rs,title_bar.rs}`
  - `repo-ref/gpui-component/crates/ui/src/dock/mod.rs`
  - `repo-ref/gpui-component/examples/{system_monitor,window_title}/src/main.rs`

## What upstream references teach

### Zed

Zed does not center its architecture on a generic `AppShell`.

Instead:

- a high-level `workspace` crate owns pane groups, docks, status bar, modal/toast layers, and a
  titlebar slot,
- separate crates such as `title_bar`, `panel`, and `sidebar` extend that workspace owner,
- the `zed` application crate mainly initializes modules and composes them.

That is a strong signal that editor/workspace shell should be an explicit high-level owner, not a
feature tunnel inside a generic UI substrate.

### GPUI component

`gpui-component` looks different:

- it provides reusable building blocks (`Root`, `TitleBar`, `DockArea`, `Sidebar`, dialog/sheet
  roots),
- examples assemble those blocks directly into app-specific shells.

That is a strong signal that a component/design-system layer should provide shell-capable parts,
while the final app shell still belongs to an app-aware or domain-aware owner.

## Core decisions

### 1) There is no universal `AppShell`

Fret should not introduce a single catch-all `AppShell` abstraction.

The correct model is a three-way split:

1. **Window bootstrap**
2. **Page shell**
3. **Workspace shell**

Each has a different owner, change velocity, and teaching surface.

### 2) Window bootstrap is startup/runtime policy, not interior shell composition

Window bootstrap owns:

- title,
- initial size,
- min/max size,
- initial position,
- resize increments,
- resizable flag,
- future window-style / utility-window policy.

Owner split:

- mechanism: `crates/fret-launch`
- app-author builders: `ecosystem/fret-bootstrap` and `ecosystem/fret`

Window bootstrap must not own:

- page centering,
- content padding,
- card framing,
- workspace chrome.

Those are interior shell concerns, not window-creation concerns.

### 3) Workspace shell stays in owning shell-aware crates

Workspace shell owns editor-grade chrome and policy:

- frame composition,
- pane/tab chrome,
- docking integration,
- editor-style menubar / toolbar / status bar choreography,
- focus restore rules specific to pane content.

Primary owner:

- `ecosystem/fret-workspace`

Adjacent owners:

- `ecosystem/fret-docking` for docking-specific arbitration,
- app/demo crates only for proof surfaces, not for framework owner decisions.

Workspace shell does not belong in:

- `crates/fret-ui`,
- the default `fret` facade,
- generic page-shell helpers.

### 4) Page shell is real, but it should not be promoted too early

Page shell owns the inside of an app window for non-editor surfaces:

- background and outer padding,
- centered or max-width content frames,
- scroll-root choice,
- narrow vs regular-width presentation decisions,
- dialog-like or card-like outer composition.

Current evidence exists, but it is still fragmented:

- cookbook-centered card pages,
- UI gallery doc pages,
- demo-specific centered shells.

Decision:

- page shell is a valid first-class concept,
- but it should remain app-owned until multiple first-party consumers prove the same shape,
- promotion should happen only after an explicit source audit, not by convenience copying.

Promotion rule for v1:

- require at least **three aligned first-party consumers** with materially similar layout
  semantics before moving a page-shell helper into a reusable ecosystem surface,
- when promoted, do **not** put it on the default `fret` root or prelude,
- prefer a dedicated shell-aware ecosystem lane over hiding it inside `crates/fret-ui`.

### 5) Misleading shell names should be fixed, not documented around

The historical `fret::workspace_menu` name was a naming smell:

- it is a generic in-window menubar bridge built from runtime menu data and UI-kit primitives,
- it is not an editor-workspace shell by itself.

Decision:

- the old `workspace_menu` name should not return,
- the surface should live under a neutral explicit lane (`fret::in_window_menubar`) instead of
  accreting more callers under a misleading module name,
- no compatibility alias is required in this pre-release lane once first-party surfaces migrate.

### 6) Compatibility is not a goal for shell cleanup

This lane explicitly allows:

- deleting aliases,
- moving modules,
- renaming surface modules,
- updating first-party examples and docs in the same slice.

This lane explicitly avoids:

- deprecated bridge aliases,
- facade shims kept only for inertia,
- root-level re-exports that hide the owning crate.

## Proposed target architecture

### A) Window bootstrap lane

Canonical user story:

- startup window policy is authored through `FretApp`, `UiAppBuilder`, and `BootstrapBuilder`,
- backend/platform translation stays in `fret-launch`,
- demos set window constraints there and nowhere else.

Examples:

- `.window(...)`
- `.window_min_size(...)`
- `.window_position_logical(...)`
- `.window_resize_increments(...)`
- `.with_default_window_*` for auxiliary windows

### B) Page shell lane

Canonical user story:

- ordinary app demos and cookbook lessons compose content inside a page shell helper,
- page shell helpers remain opt-in and app-facing,
- page shell is not taught as part of the low-level runtime vocabulary.

Initial v1 posture:

- keep page-shell helpers app-owned unless promotion criteria are met,
- do not teach cookbook-local helpers as if they are already framework contracts,
- audit first-party users first, then decide whether a reusable promotion target is justified.

Current audit result:

- `M3` closes with **no shared page-shell promotion** in v1,
- cookbook lesson shells, UI Gallery docs shells, and demo-specific responsive shells are not the
  same surface,
- see `docs/workstreams/shell-composition-fearless-refactor-v1/PAGE_SHELL_AUDIT_2026-04-02.md`.

### C) Workspace shell lane

Canonical user story:

- editor/workspace composition uses `fret-workspace`,
- docking integration layers build around that owner,
- chrome and focus policy stay shell-aware and ecosystem-owned.

This aligns with both:

- Fret's own documented direction,
- Zed's explicit workspace owner model.

### D) Menu bridge lane

Canonical user story:

- runtime `MenuBar` / command-gating integration is exposed under a neutral app-facing name,
- it stays opt-in,
- it remains off the default prelude,
- it is not misclassified as workspace shell.

## Execution policy

When a shell surface moves:

1. move/rename the owning module,
2. update first-party callers,
3. update docs that teach the surface,
4. delete the old alias in the same lane unless a hard external consumer exists.

When a shell helper is proposed for promotion:

1. inventory first-party callers,
2. prove aligned semantics,
3. decide owner crate,
4. add at least one source-policy or diagnostics gate that prevents owner drift.

## Risks

- Promoting page shell too early will create a second shell rewrite later.
- Leaving misleading names in place will keep reintroducing owner confusion.
- Moving shell policy into `crates/fret-ui` will harden the wrong contracts.
- Treating demos as one-off code instead of teaching surfaces will keep the public story blurry.

## Evidence anchors (current baseline)

- `docs/workspace-shell.md`
- `docs/crate-usage-guide.md`
- `docs/workstreams/shell-composition-fearless-refactor-v1/PAGE_SHELL_AUDIT_2026-04-02.md`
- `apps/fret-cookbook/src/scaffold.rs`
- `ecosystem/fret-workspace/src/lib.rs`
- `ecosystem/fret-workspace/src/frame.rs`
- `ecosystem/fret-workspace/src/layout.rs`
- `ecosystem/fret/src/in_window_menubar.rs`
- `ecosystem/fret/src/lib.rs`
- `repo-ref/zed/crates/workspace/src/workspace.rs`
- `repo-ref/gpui-component/crates/ui/src/{lib.rs,root.rs,title_bar.rs}`

## Definition of done

This lane is in a good v1 state when:

- the repo has one explicit shell ownership model,
- window bootstrap, page shell, and workspace shell are no longer conflated in docs or first-party
  examples,
- misleading shell names are either fixed or tracked with a concrete migration step,
- the default `fret` lane remains app-facing and does not absorb workspace-shell surface again,
- the next code refactor can delete stale shell APIs directly instead of preserving aliases.
