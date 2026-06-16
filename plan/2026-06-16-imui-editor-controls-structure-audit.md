---
title: IMUI editor-controls structure audit
date: 2026-06-16
type: working-note
---

# IMUI Editor-Controls Structure Audit

## Purpose

Record the current structural read on the IMUI/editor-controls lane before any fearless refactor.
This is not an implementation plan. It is a checkpoint for deciding which seam deserves the next
deepening pass.

## What the evidence says

- The current editor-controls pain is mostly composition depth and layout-policy coupling.
- The strongest source of visible height change is `PropertyRowLayoutVariant::Auto` in
  `ecosystem/fret-ui-editor/src/composites/property_row/layout.rs`, because the row can flip
  between horizontal and vertical composition based on last-frame bounds.
- `DragValue` and `NumericInput` both build a dual-branch shell spine through
  `ecosystem/fret-ui-editor/src/controls/session_shell.rs`, which is correct but heavy for dense
  inspector surfaces.
- `TextAssistField` and `ColorEdit` keep popup/overlay policy inside the editor ecosystem, which is
  the right layer, but those surfaces add more wrapper depth around already dense rows.
- The hot path is therefore not a runtime rewrite candidate first. It is a component-structure and
  lane-policy candidate.

## 2026-06-16 Implementation Note

- The default `PropertyGrid` / `PropertyGridVirtualized` row option path now uses
  `PropertyRowLayoutVariant::Row` instead of implicit `Auto`.
- `PropertyRowOptions::with_grid_defaults(...)` preserves explicit caller variants and only fills
  missing shared metrics.
- Coverage now includes:
  - a default-row context test for `PropertyGrid`,
  - a merge test for `PropertyRowOptions`,
  - and the existing row-separation geometry test.
- The visible-height jump remains a valid concern for explicit `Auto` callers, but it is now an
  opt-in policy rather than the grid default.

## Upstream shape comparison

- `repo-ref/base-ui` keeps its composite list machinery shallow: `CompositeList` is a registry and
  ordering layer, `ComboboxList` is a thin list container, `ComboboxCollection` is a fragment
  mapper, and `ComboboxItem` owns the item-specific behavior.
- `repo-ref/imgui` shows the opposite extreme: immediate-mode code stays flat and lets the demo
  surface own the composition locally.
- Fret's editor controls sit between those two. The current issue is that the inspector lane has
  accumulated too many policy shells around each row, not that the runtime substrate is missing a
  primitive.

## Fearless refactor candidates

1. Stabilize the property row shell and make the row/column split explicit for dense inspector
   surfaces. Keep `Auto` as an opt-in policy, not the default shape everywhere.
2. Simplify the numeric edit spine so `DragValue` / `NumericInput` share a flatter stable shell and
   stop changing the row's effective height when the mode changes.
3. Keep overlay policy in the editor ecosystem, but factor repeated popup surface chrome into a
   shared helper instead of repeating the same wrapper pattern in each control.

## Not the next target

- Do not start with `fret-ui` runtime rewrites.
- Do not move overlay/focus policy out of `fret-ui-editor` just because these controls are dense.
- Do not treat `repo-ref/base-ui` as evidence that the editor lane should become equally shallow;
  the problem here is the shape of the current inspector lane, not the existence of wrappers by
  itself.

## Current recommendation

Start with `PropertyRow` / `PropertyGrid` first. That seam has the clearest link to the observed
height jump and the biggest leverage over the rest of the dense editor surface.
