# Workspace Shell TabStrip (Fearless Refactor v1) — Design

## Context

Fret has two distinct “tab” families:

- **In-page Tabs** (Radix/shadcn semantics): `ecosystem/fret-ui-shadcn` (`Tabs`, `TabsList`, `TabsTrigger`).
- **Editor/workspace TabStrip** (window/pane chrome): `ecosystem/fret-workspace::WorkspaceTabStrip`.

This workstream is about the second family: an editor-grade TabStrip that can serve:

- workspace shells (top bar tab strip),
- docking panes (tab bar inside a dock layout),
- future multi-window / multi-viewport shells.

The repository is contract-driven and layered:

- `crates/*` (especially `crates/fret-ui`) must remain **mechanism/contract-only**.
- policy-heavy interaction (dismiss, focus rules, sizing defaults, editor chrome) must live in
  `ecosystem/*`.

## Why “fearless refactor” here

Tab strips tend to accumulate one-off behavior (overflow, scrolling, drag/drop, pin/preview, MRU,
focus rules). If we do not lock down a clear kernel and regression gates early, we will keep
rewriting the same behavior in multiple places (workspace vs docking vs future shells).

## References (sources of truth)

We use different references for different kinds of parity:

- **Zed (`repo-ref/zed`)**: editor-grade semantics and workflow outcomes (pinned tabs, preview tabs,
  drag-to-split, multiple tab rows, drop targets).
- **dockview (`repo-ref/dockview`)**: robust overflow detection + “overflow list” UX patterns in a
  docking UI (header overflow renderer, dropdown pipeline).
- **gpui-component (`repo-ref/gpui-component`)**: a smaller GPUI-native dock/tab panel model that
  shows how TabBar + Dock can be wired together.

We are not porting DOM/CSS behavior 1:1. We are aligning **outcomes** in Fret’s renderer/runtime.

## Goals

1. **Single interaction kernel** for editor-style tab bars:
   - selection + keyboard navigation (roving tabindex),
   - scrolling + “active tab into view” behavior,
   - overflow dropdown/list (optional),
   - drag reorder + “insert before/after” indicators,
   - cross-pane drag/drop intents (workspace & docking),
   - pinned tabs + (optional) separate pinned row,
   - preview tab semantics (optional, Zed/VS Code style).
2. **One implementation, multiple adapters**:
   - `WorkspaceTabStrip` (workspace shell),
   - docking tab bars (inside `ecosystem/fret-docking`),
   - future custom shells (apps in `apps/*`).
3. **No layering violations**:
   - no new policy knobs in `crates/fret-ui`,
   - new behavior lives in `ecosystem/*`,
   - mechanism fixes in `crates/fret-ui` only when clearly generic (e.g. roving collection rules).
4. **Regression gates**:
   - deterministic tests for the core state machine,
   - `fretboard diag` scripts for drag/drop and overflow UX where appropriate,
   - stable `test_id` surfaces.

## Non-goals (v1)

- Building a full Zed/VS Code workspace (history stack, breadcrumbs, etc.) beyond what is needed
  for the tab strip.
- Styling parity pixel-perfect across themes; this workstream targets interaction correctness and
  stable seams first.
- Committing to a public, stable API for third-party crates yet (we can iterate inside `ecosystem/`
  first).

## Constraints / Design rules

- **Mechanism vs policy**: `crates/fret-ui` provides primitives (`ScrollHandle`, `PointerRegion`,
  `RovingFlex`, semantics roles). The TabStrip behavior policy is ecosystem-owned.
- **Determinism**: tab identity must be stable across reorders; avoid “index identity”.
- **Testability**: every “hard to change” outcome must have a gate (test or diag script).
- **Automation**: interactive targets must have stable `test_id` surfaces.

## Decisions (v1)

Decisions for v1 are recorded in `OPEN_QUESTIONS.md` (accepted).

## Proposed architecture

### 1) TabStrip kernel (ecosystem, headless-ish)

Introduce a reusable kernel module (location TBD, see “Ownership”):

- `TabStripModel` (state):
  - ordered list of tabs (ids),
  - active tab id,
  - pinned boundary (pinned_count or per-tab pin flag),
  - preview tab id (optional),
  - scroll state (external handle),
  - drag state (source tab, hovered target, insertion side),
  - overflow state (computed: is_overflowing, overflowed_tabs list).
- `TabStripOps` (pure-ish helpers):
  - compute hit rects → insertion target,
  - compute scroll-to-reveal region,
  - compute overflow membership,
  - apply commands (move left/right, close, pin/unpin, activate next/prev, MRU toggle).

The kernel should be structured so that:

- geometry inputs are data (rects, viewport size, pointer position),
- outputs are intents (activate tab, move tab, close tab, scroll offset delta, show overflow menu),
- integration with `UiHost` is in adapter code, not in the kernel.

### 2) UI adapters (ecosystem)

#### Workspace adapter

`ecosystem/fret-workspace::WorkspaceTabStrip` becomes a thin adapter that:

- renders the tab row(s),
- binds `ScrollHandle`,
- wires pointer + keyboard events into kernel intents,
- dispatches workspace commands (`CommandId`) on intents,
- provides stable `test_id` targets for automation.

#### Docking adapter

Docking has its own geometry model today (e.g. tab bar geometry utilities). We should avoid
duplicating interaction policy:

- docking provides the *layout constraints + pane model*,
- the TabStrip kernel provides *tab strip interaction semantics*.

### 3) Overflow dropdown/list

We want an overflow mechanism that can match both Zed-like and dockview-like UX:

- “More” dropdown listing overflowed tabs (dockview-style),
- optional search/typeahead inside the overflow list (future),
- close buttons in overflow list (dockview has tests around this).

Implementation note:

- dockview’s `OverflowObserver` is DOM-specific, but the concept maps well:
  compute overflow based on measured tab rects vs viewport width and scroll offset.

### 4) Drag/drop + split integration

We need two distinct drags:

1) **Reorder within a strip** (before/after insertion).
2) **Move across panes**:
   - drop into another strip (becomes active tab, insert at target),
   - drop into “header space” (empty strip region),
   - drop to split (edge drop targets; Zed-like).

The kernel should emit a `DropIntent` that docking/workspace can interpret:

- `Reorder { target, side }`
- `MoveToPane { pane_id, index }`
- `SplitPane { pane_id, direction }` (workspace/docking owns whether it is allowed)

## Ownership (where code should live)

Recommended v1 ownership:

- shared “tab strip kernel”: `ecosystem/fret-workspace` (initially) or a new crate
  `ecosystem/fret-editor-chrome` if docking also needs it immediately.
- docking-specific adapters remain in `ecosystem/fret-docking`.

We should not put editor chrome policy in `ecosystem/fret-ui-kit` unless we expect it to be a
generic UI-kit primitive. TabStrip policy is editor-heavy and likely belongs in `fret-workspace`.

## Evidence anchors (current baseline)

Recent baseline improvements (already landed):

- `ecosystem/fret-workspace/src/tab_strip/mod.rs` — roving keyboard navigation + wheel-to-horizontal
  scroll policy + focus stability.
- `crates/fret-ui/src/declarative/host_widget/event/roving_flex.rs` — `PointerRegion` treated as a
  transparent wrapper for roving item collection.
- `crates/fret-ui/src/declarative/tests/interactions/roving_flex.rs` — regression test for roving
  under pointer regions.
- `ecosystem/fret-workspace/tests/tab_strip_pointer_down_does_not_steal_focus.rs` — focus
  stability gate for tab strip pointer down.
- `ecosystem/fret-dnd/src/scroll.rs` — shared axis auto-scroll helpers (`compute_autoscroll_x/y`)
  reused by workspace + docking tab bars (avoids one-off edge-scroll formulas).

Reference anchors:

- Zed pane tab bar: `repo-ref/zed/crates/workspace/src/pane.rs` (`render_tab_bar`, pinned rows,
  drop targets, preview tab semantics, drag-to-split).
- dockview header tabs: `repo-ref/dockview/packages/dockview-core/src/dockview/components/titlebar/tabs.ts`
  and `tabsContainer.ts` (overflow list pipeline, headerOverflow renderer).

## Definition of done (v1)

We consider v1 “done” when:

- Workspace TabStrip supports: roving nav, scroll-to-reveal, overflow list, reorder, cross-pane move
  (intra-workspace), and pinned boundary (at least single-row pinned section).
- Docking tab bars reuse the same interaction kernel (no duplicate policy code).
- At least one regression gate exists for each of:
  - roving navigation,
  - overflow list membership / open-close,
  - reorder insertion side,
  - cross-pane drop intent,
  - pinned boundary behavior,
  - preview tab (if enabled).
