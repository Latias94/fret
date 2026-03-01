# Editor TabStrip Unification Fearless Refactor v1 (Parity Matrix)

This document captures a **behavioral parity map** between:
- Zed (`repo-ref/zed`) as an editor-grade reference implementation.
- Dockview (`repo-ref/dockview`) as a docking/tab overflow reference (especially dropdown semantics).
- Fret implementations:
  - Workspace tab strip: `ecosystem/fret-workspace/src/tab_strip/`
  - Docking tab bar: `ecosystem/fret-docking/src/dock/`

The goal is to converge on shared mechanism vocabulary (via `fret-ui-headless`) while keeping
**interaction policy** in ecosystem layers (`fret-ui-kit`, `fret-workspace`, `fret-docking`).

## Terminology (normalized)

- **Tab strip viewport**: the visible rect used to compute overflow membership.
- **Overflow control**: the affordance that opens a list of tabs when the strip overflows.
- **Overflow dropdown/menu**: the UI surface listing hidden tabs.
- **Header space / end-drop surface**: reserved strip space after the last tab that still accepts
  drops, resolving to a canonical end insert-index.
- **Close affordance**: the “x” button; close vs activate must be explicitly arbitrated.

## Reference anchors

- Zed pane tab bar (pinned/unpinned, scroll handle, two-row option):
  - `repo-ref/zed/crates/workspace/src/pane.rs`
  - `repo-ref/zed/crates/ui/src/components/tab_bar.rs`
- Dockview overflow dropdown with close buttons:
  - `repo-ref/dockview/packages/dockview-core/src/__tests__/dockview/components/titlebar/tabsContainer.spec.ts`
- Fret workspace tab strip:
  - `ecosystem/fret-workspace/src/tab_strip/mod.rs`
  - `ecosystem/fret-workspace/src/tab_strip/surface.rs`
- Fret docking tab bar:
  - `ecosystem/fret-docking/src/dock/space.rs`
  - `ecosystem/fret-docking/src/dock/tab_overflow.rs`

## Parity matrix (outcomes, not implementation)

Legend:
- ✅ implemented
- ⚠️ partially / differs
- ❌ missing
- N/A not applicable / different model

| Topic | Zed | Dockview | Fret (workspace) | Fret (docking) | Owner layer | Notes / gates |
|---|---:|---:|---:|---:|---|---|
| Tab strip overflow mechanism | ✅ scrollable strip | ✅ (with dropdown control) | ✅ (scroll + overflow control) | ✅ (scroll + overflow control) | `fret-ui-headless` (mechanism) | Ensure both use the same “viewport + margin” overflow membership helper. |
| Overflow item membership | N/A (no dropdown) | ✅ overflowed-only | ✅ overflowed-only or overflowed+active (policy) | ✅ overflowed+active (policy) | Adapter policy | Keep policy per adapter; record default in `OPEN_QUESTIONS.md`. |
| Overflow dropdown close button visible | N/A | ✅ (explicit test) | ✅ | ✅ | Adapter policy | Workspace gate: `ecosystem/fret-workspace/tests/tab_strip_overflow_menu_lists_overflowed_tabs.rs`. |
| Overflow close does **not** activate | N/A | ✅ (explicit test) | ✅ | ✅ | Adapter policy | Docking gate: `dock::tests::dock_space::overflow_menu_close_does_not_activate_tab`. Workspace gate: `ecosystem/fret-workspace/tests/tab_strip_overflow_menu_lists_overflowed_tabs.rs`. |
| Selecting overflow item activates + ensures visible | N/A | ✅ | ⚠️ likely | ✅ | Adapter policy + shared helper | Docking already ensures visible on selection; workspace should match. |
| Close vs activate hit-test arbitration | ✅ (tab-specific) | ✅ | ✅ (overflow menu) | ✅ (overflow menu) | Adapter policy | Uses `fret-ui-shadcn` dropdown item trailing action hook: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`. Priority ordering still needs to be documented (see Q5). |
| Overflow control excluded from drop surfaces | ✅ (no dedicated control) | ✅ | ✅ | ✅ | `fret-ui-headless` | Both already treat overflow control as non-drop surface. |
| Header space treated as end-drop surface | ✅ | ✅ | ✅ | ✅ | `fret-ui-headless` | Canonical end-drop insert index must be deterministic across both. |
| Tab halves decide insert-index on drop | ✅ | ✅ | ✅ | ✅ | `fret-ui-headless` | Existing docking tests; workspace needs diag or unit gate. |
| Auto-scroll near edges during drag | ✅ | ✅ | ✅ | ✅ | Adapter policy + helper | Keep per-impl, but share helper vocabulary for “edge delta”. |
| Pinned/unpinned tabs | ✅ | ❌ | ❌ | ❌ | Workspace policy | Zed supports pinned tabs and optional separate rows; Fret should treat this as workspace policy (not docking). |
| Multi-row tab strip | ✅ (configurable) | ❌ | ❌ | ❌ | Workspace policy | Depends on pinned/unpinned decision. |
| Tab bar nav history buttons | ✅ | ❌ | ❌ | ❌ | Workspace policy | Not required for docking; editor-grade workspace might want it later. |
| Keyboard/focus semantics (editor expectations) | ✅ | ✅ | ⚠️ partial | ⚠️ partial | `fret-ui-kit` policy | Track via APG-aligned rules and `workspace.pane.focus_tab_strip` command closure. |
| Input arbitration priority with other affordances | ✅ | ✅ | ⚠️ | ⚠️ | Adapter policy (documented) | Docking fixed a real overlap bug: overflow control must win over float-zone. |

## Immediate parity targets (v1)

1) Keep docking/workspace aligned on:
   - overflow membership helper + margin
   - overflow dropdown activation ensures visible
   - canonical end-drop surface semantics
2) Introduce a shared **TabStripController** module in `fret-ui-kit` (policy toolbox), so that:
   - workspace and docking can share close/activate arbitration rules
   - tests/diag scripts can reference the same intent vocabulary
