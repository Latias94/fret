# Workspace TabStrip (editor-grade) — TODO

Scope: `ecosystem/fret-workspace` tab strip + pane/workspace shell integration.

Non-goals:
- Do **not** move editor policy into `crates/fret-ui` (mechanism/contract layer).
- Do **not** build a general-purpose component library here; keep this workspace/editor oriented.

## Audit + gap list

- [ ] Inventory current implementation and seams:
  - `ecosystem/fret-workspace/src/tab_strip/mod.rs`
  - `ecosystem/fret-workspace/src/tab_drag.rs`
  - `ecosystem/fret-workspace/src/tabs.rs`
- [ ] Identify missing editor-grade behaviors vs reference implementations:
  - Zed pane tab bar (editor-grade UX): `repo-ref/zed/crates/workspace/src/pane.rs`
  - Dockview tab DnD invariants/tests: `repo-ref/dockview/packages/dockview-core/src/__tests__/dockview/components/*`

## Refactor (fearless but modular)

- [ ] Split `ecosystem/fret-workspace/src/tab_strip/mod.rs` into cohesive modules:
  - `view` (rendering + tokens)
  - `interaction` (pointer/keyboard -> intents)
  - `geometry` (rect caching, scroll viewport, overflow)
  - keep `kernel` as the headless “compute” layer
- [ ] Define an explicit intent surface for the tab strip:
  - activate / close / reorder / pin-unpin / open context menu / start drag
  - keep command dispatch as an adapter, not the core interaction output
- [ ] Make drag+drop state ownership explicit:
  - local reorder vs cross-pane drop vs “external drag” (should be ignored / delegated)
- [ ] Add a small “UI contract” checklist for tab strip selectors:
  - stable `test_id` for root, tab chrome, close button, overflow button, pinned boundary

## Behavior work (editor-grade)

- [ ] Pinned tabs:
  - [ ] Support “separate row” and “single row + pinned boundary” modes (configurable)
  - [ ] Ensure reorder cannot cross pinned boundary unless explicitly pin/unpin
- [ ] Overflow and discoverability:
  - [ ] Overflow button lists hidden tabs (stable `test_id` per entry)
  - [ ] Scroll-to-active on activation, with hysteresis to avoid jitter
- [ ] Keyboard:
  - [ ] Roving focus + semantics role mapping consistent with APG expectations
  - [ ] Tab cycle policy integrates with `WorkspaceTabs::TabCycleMode` (InOrder vs MRU)
- [ ] Pointer:
  - [ ] Middle-click close (optional policy)
  - [ ] Close button hit target does not start a drag
  - [ ] Double-click policy (new tab / maximize / no-op) lives in the shell layer

## Diagnostics + regression gates

- [ ] Add/extend promoted diag scripts under `tools/diag-scripts/workspace/**`:
  - drag reorder within strip (invariants-first)
  - drag tab to split-zone (drop preview) in workspace shell demo (start with screenshot, then invariants)
  - pinned boundary behavior (pin/unpin + reorder)
- [ ] Promote at least one workspace shell script into a smoke suite:
  - suite: `diag-hardening-smoke-workspace`
  - initial script: `workspace-shell-demo-tab-drag-to-split-right-drop-preview-screenshot`
- [ ] For each new behavior, add at least one gate:
  - invariants (preferred) via diagnostics snapshots
  - screenshots only when invariants are insufficient
