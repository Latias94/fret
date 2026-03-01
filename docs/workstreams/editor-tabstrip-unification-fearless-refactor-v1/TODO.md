# Editor TabStrip Unification Fearless Refactor v1 (TODO)

Goal: converge `workspace` tabstrip + `docking` tab bar behaviors onto shared mechanism vocabulary
(`fret-ui-headless`) while keeping policy in ecosystem layers.

This workstream is intentionally scoped to “editor-grade tab UX”:
- overflow behavior (membership, dropdown list, close affordances)
- drag & drop targets (tab halves, end-drop surfaces, overflow button exclusion)
- scroll/visibility guarantees (active tab stays visible)
- keyboard/focus semantics (editor expectations)

## Scope

- [ ] Write a cross-reference parity matrix:
  - `repo-ref/zed` (editor pane tab bar, pinned/unpinned, scroll handle)
  - `repo-ref/dockview` (overflow list membership + dropdown behaviors)
  - Fret: `ecosystem/fret-workspace` + `ecosystem/fret-docking`
-   Output: `docs/workstreams/editor-tabstrip-unification-fearless-refactor-v1/PARITY_MATRIX.md`
- [ ] Normalize terminology in code and docs:
  - “tabs viewport”, “header space”, “overflow control”, “end-drop surface”
- [ ] Decide overflow dropdown policy:
  - list overflowed-only vs overflowed+active (current docking policy)
  - include close buttons in overflow list (dockview has tests for this)
- [ ] Add diag script coverage:
  - overflow dropdown open/close
  - select tab from dropdown keeps active visible
  - drag end-drop on overflow header space resolves canonical insert_index
- [ ] Add minimal unit tests where headless helpers are used by adapters.

## Non-goals

- Component styling/token design beyond necessary hit rects and affordances.
- Replacing Fret’s retained tree with declarative rebuild (tracked elsewhere).
- Full APG/a11y closure (tracked in a11y workstreams); only tab-relevant keyboard/focus here.

## References

- Existing workstreams:
  - `docs/workstreams/workspace-tabstrip-editor-grade-v1/`
  - `docs/workstreams/docking-tabbar-fearless-refactor-v1/`
- Headless mechanism helpers:
  - `ecosystem/fret-ui-headless/src/tab_strip_surface.rs`
  - `ecosystem/fret-ui-headless/src/tab_strip_overflow.rs`
