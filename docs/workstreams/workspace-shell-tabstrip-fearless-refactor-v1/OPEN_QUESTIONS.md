# Workspace Shell TabStrip (Fearless Refactor v1) — Open Questions (with recommendations)

This file lists decisions we should make explicitly to avoid churn during implementation.

## Q1: Pinned model — `pinned_tab_count` boundary vs per-tab pinned flag

**Recommendation:** start with **`pinned_tab_count`** (Zed-like boundary).

Why:

- Matches editor UX expectations (pinned is “left segment”).
- Simplifies reorder + drop targets (one boundary, easy invariants).
- Makes a “separate pinned row” optional without changing the underlying meaning.

Upgrade path:

- We can later migrate to per-tab pin flags if we need non-contiguous pinned groups.

## Q2: Overflow dropdown list — “overflowed only” vs “all tabs”

**Recommendation:** **overflowed-only** list (dockview-like) for v1.

Why:

- Fast path for the common pain: selecting a tab you cannot see.
- Keeps the list short; typeahead/search can be added later if needed.

Optional later:

- “All tabs” list with grouping and typeahead (VS Code-like).

## Q3: Click focus policy — should clicking a tab move keyboard focus?

**Recommendation:** clicking a tab **should not steal content focus** by default.

Why:

- Editor shells rely on stable focus for typing; tab activation is a view change, not a focus change.
- Aligns with “chrome interactions are focus-neutral” principle.

Escape hatches:

- Provide a command “Focus tab strip” for keyboard users.
- When the content is empty/unfocused, allow tab strip to become focus owner.

## Q4: Keyboard nav semantics — roving focus vs “selection without focus”

**Recommendation:** **roving focus + activation on roving change** (APG-aligned).

Details:

- Arrow keys move the roving active item.
- Activation mode:
  - v1 default: automatic (activates as roving moves) for editor tab strips.
  - optional: manual (Enter/Space) if a use-case appears.

## Q5: Preview tab semantics — do we implement it in v1?

**Recommendation:** **Yes (optional but recommended)**, modeled after Zed/VS Code.

Minimum contract:

- At most one preview tab per pane.
- Opening a previewable item reuses/replaces the existing preview tab slot.
- “Commit” turns preview → normal (e.g. after edit or explicit pin/keep).

Opt-out:

- Preview can be disabled globally; behavior becomes “always open normal tab”.

## Q6: Drag-to-split — where does the split policy live?

**Recommendation:** split is **policy-owned** by workspace/docking; the kernel emits **intent only**.

Why:

- Avoids bleeding shell policy into shared logic.
- Different shells may disallow split (or vary thresholds/hysteresis).

## Q7: Edge auto-scroll during drag reorder

**Recommendation:** implement in v1.

Why:

- Without it, reorder with many tabs is frustrating even if overflow list exists.
- Mechanically straightforward if we already have tab rects + viewport + scroll handle.

## Q8: Drop targets — do we add explicit “end of strip” targets?

**Recommendation:** Yes, add explicit end targets.

Why:

- Zed and gpui-component both rely on “empty space” / “drop target” nodes for robust “after last tab”.
- It reduces reliance on fuzzy hit-testing near the last tab’s right edge.

## Q9: Cross-pane drag — should we allow “drop into header space” (not on a tab)?

**Recommendation:** Yes.

Why:

- dockview uses “header_space” as a first-class drop surface.
- Enables “move to end” without requiring a specific tab as target.

