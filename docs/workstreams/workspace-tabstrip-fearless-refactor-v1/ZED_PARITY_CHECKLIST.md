# Workspace TabStrip (Fearless Refactor v1) — Zed Parity Checklist

This document turns Zed’s “editor-grade pane tab bar” behavior into an actionable checklist for
Fret’s workspace tab strip. The goal is to align **outcomes / invariants**, not code structure.

Status labels:

- `Yes`: implemented + gated
- `Partial`: implemented with known gaps
- `No`: not implemented
- `N/A`: not applicable for v1

## Scope

In scope (v1):

- Pinned tabs model + bulk-close protection
- Single preview slot (Zed-style) + commit/replace rules
- Drag and drop semantics for reordering and cross-pane moves
- Overflow / end-drop surfaces and click arbitration (close vs activate)

Out of scope (v1):

- Platform title bar / OS-native window tabbing (separate Zed surface)
- Full keyboard navigation parity (tracked separately under a11y/APG work)

## Zed sources of truth

Primary editor tab bar:

- `repo-ref/zed/crates/workspace/src/pane.rs`
  - Preview: `replace_preview_item_id`, `close_current_preview_item`, `unpreview_item_if_preview`,
    `handle_item_edit`, `open_item`
  - Pinned: `pinned_tab_count`, `is_tab_pinned`, `handle_pinned_tab_bar_drop`
  - Bulk close: `close_other_items`, `close_items_to_the_side_by_id`
  - DnD: `handle_tab_drop`, `render_tab_bar` (single-row vs two-row), pinned drop target

Related (not a pane tab bar, but useful vocabulary):

- `repo-ref/zed/crates/platform_title_bar/src/system_window_tabs.rs` (window-level tab dragging)

## Mapping: Zed state ↔ Fret state

- Zed pinned model: `pinned_tab_count` (prefix of tab list is pinned)
  - Fret workspace: `WorkspaceTabs::pinned_tab_count` + `is_tab_pinned` (same prefix model)
- Zed preview model: `preview_item_id: Option<EntityId>`
  - Fret workspace: `WorkspaceTabs::preview_tab_id: Option<Arc<str>>`

## Checklist

| Capability | Zed behavior (summary) | Fret status | Evidence / gates | Notes / follow-ups | Layer |
|---|---|---:|---|---|---|
| Explicit end-drop target | “Last empty space” drop inserts at end; stable drop surface | Yes | Diag scripts in `tools/diag-scripts/workspace/shell-demo/*reorder*end*` | Keep stable `drop_end` anchor | Mechanism |
| Overflow close vs activate | Clicking overflow row close must not activate that tab | Yes | `workspace-shell-demo-tab-overflow-close-does-not-activate.json` | Shared click arbitration lives in headless | Mechanism + policy |
| “Close others” keeps pinned | Bulk-close must not close pinned tabs by default | Yes | `ecosystem/fret-workspace/src/tabs.rs` + `workspace-shell-demo-tab-close-*-keeps-pinned-smoke.json` | Zed has `close_pinned` override option; Fret currently models only “protect pinned” | Policy |
| Preview replaces previous preview | Opening a new preview replaces existing preview if safe | Yes | `ecosystem/fret-workspace/src/tabs.rs:open_preview_and_activate` + preview diag scripts | Ensure replacement is blocked by dirty/pinned (matches Zed intent) | Policy |
| Preview commits on edit / dirty | Editing a preview tab should unpreview / commit | Yes | `ecosystem/fret-workspace/src/tabs.rs:set_dirty` commits preview | Align long-term with “preserve preview” hook if needed | Policy |
| Cross-pane move | Dragging a tab to another pane moves it and preserves ordering | Yes | Workspace cross-pane move diag gate | Keep canonical-order insert semantics | Policy |
| Pinned boundary surface | Clear pinned/unpinned split surface exists | Partial | `ecosystem/fret-workspace/src/tab_strip/mod.rs` pinned boundary + smoke gates | Zed supports optional two-row pinned; Fret currently uses a single strip + boundary | Mechanism |
| Cross-boundary DnD (pinned↔unpinned) | Zed: pinned-ness can be preserved when dropping at beginning / pinned row | No (by design today) | Fret: `ecosystem/fret-workspace/src/tab_strip/mod.rs:814` drops are rejected when pinned state differs | Decide whether Fret should adopt Zed’s “drop at start keeps pinned” rule, or keep “explicit pin only” | Policy |
| Drop-to-pin affordance | Zed: dedicated pinned row drop target pins on drop | No | N/A | Could be a future policy affordance (might require two-row UI) | Policy |
| Close active pinned tab | Zed: close_active_item can refuse and activate an unpinned tab (unless `close_pinned`) | Partial | Fret: bulk-close protection exists; active-close policy not fully modeled | Decide whether “close pinned” should be an explicit override command/policy | Policy |
| “Close others” unpreviews active | Zed: `close_other_items` unpreviews active item before closing | Partial | Fret: preview is committed/cleared on dirty; bulk-close behavior needs an explicit parity check | Ensure bulk-close does not accidentally preserve preview state | Policy |
| Dirty close confirmation | Zed prompts (save/discard/cancel) before closing dirty items | Partial | Evidence: `ecosystem/fret-workspace/src/close_policy.rs` + `ecosystem/fret-workspace/src/tabs.rs` (`apply_command_with_close_policy`) + `workspace-shell-demo-tab-close-dirty-is-blocked-smoke.json` | Fret provides a policy hook; apps must implement prompt + follow-up commands | Policy |

## Priority next steps (recommended)

P0 (unblocks cleanup / reduces regressions):

- Audit which workspace paths still use legacy ad-hoc hit-tests and consolidate under the kernel
  (WTS-cleanup-040), using this checklist as acceptance criteria.

P1 (editor semantics convergence):

- Decide the cross-boundary DnD rule:
  - Option A: keep “no implicit pin/unpin” (current Fret behavior, requires explicit pin action)
  - Option B: adopt Zed’s “drop at beginning keeps pinned” semantics (requires clear affordance)
- If Option B: add a minimal “drop-to-pin” affordance (two-row or explicit pinned drop zone).

P2 (command surface parity):

- Consider adding `close_pinned` overrides to bulk-close commands (Zed has `CloseOtherItems { close_pinned }`).
