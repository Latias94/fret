# ADR 0068: Focus Traversal and Focus Scopes (Tab/Shift+Tab, Modal-Aware)

Status: Accepted

## Implementation Status (as of 2025-12-29)

The runtime traversal mechanism described here is **largely implemented** in `crates/fret-ui`:

- Commands: `focus.next` / `focus.previous` are handled by `UiTree` (`crates/fret-ui/src/tree/mod.rs`).
- Modal-aware candidate gating: traversal only considers nodes in the **active input layers**
  (modal-barrier aware), and applies a conservative **visibility intersection** filter.
- Cross-root ordering: active roots are iterated in **reverse paint order** (top → bottom), then a
  deterministic pre-order traversal is used within each root.
- FocusScope trap wiring: declarative `FocusScopeProps { trap_focus: true }` traps Tab traversal
  inside its subtree, and prevents focus requests that would move focus outside while the trap is
  active.

Current implementation entry points:

- `UiTree::dispatch_focus_traversal(...)` and `UiTree::focus_traverse_in_roots(...)`
  (`crates/fret-ui/src/tree/mod.rs`).

Known gaps / follow-ups:

1) **Focus scopes are not yet surfaced as a reusable component policy**
   - The runtime exposes a reusable mechanism (`focus_traverse_in_roots`) intended to be used by
     component-owned focus scopes.
   - Partial: `fret-ui-kit` now provides:
     - `primitives::focus_scope::{focus_trap, focus_trap_with_id}` for trap/loop composition, and
     - reusable “initial focus” / “restore focus” helpers used by the overlay system
       (`ecosystem/fret-ui-kit/src/window_overlays/render.rs`).
   - Overlay auto-focus policy is now expressible via Radix-style open/close auto focus hooks
     (opt-in callbacks that may `preventDefault`) in the overlay controller.
   - Remaining work: extend the primitive to cover non-overlay subtrees (restore semantics when a
     scope unmounts without being a distinct overlay layer) and cover additional FocusScope
     outcomes (e.g. focus guards) as needed.

2) **Scroll-into-view is implemented, but still needs contract polishing**
   - The runtime now attempts to scroll a newly focused node into view by walking ancestors and
     calling `Widget::scroll_descendant_into_view` on the first scroll-capable widget.
     - Runtime entry point: `UiTree::scroll_node_into_view` (`crates/fret-ui/src/tree/commands.rs`)
     - Widget contract: `can_scroll_descendant_into_view` / `scroll_descendant_into_view`
       (`crates/fret-ui/src/widget.rs`)
     - Default declarative implementation (Scroll / VirtualList): `crates/fret-ui/src/declarative/host_widget.rs`
     - Regression test: `crates/fret-ui/src/tree/tests/scroll_into_view.rs`
   - Remaining work: clarify axis behavior, nested scroll priority rules, and interactions with
     overlays/modality so components can rely on a crisp “focus implies visibility” baseline.

## Context

Fret targets editor-grade UX with:

- multi-root overlays (ADR 0011),
- a command routing layer (ADR 0020),
- modal barriers that make underlay UI inert (ADR 0066),
- shadcn/Radix-aligned component behavior (APG/Radix outcomes).

Many component behaviors implicitly assume a consistent focus traversal baseline:

- `Tab` / `Shift+Tab` in dialogs and sheets,
- keyboard-only navigation affordances (focus-visible, ADR 0061),
- predictable interaction under modal barriers (no underlay focus leakage).

If we do not lock a runtime-level traversal contract early, component-layer overlay policies will
either:

- duplicate traversal logic per component, or
- encode ad-hoc focus hacks into the runtime (contract drift).

## Decision

### 1) Focus traversal is a runtime mechanism (not a component policy)

`crates/fret-ui` provides a stable focus traversal mechanism that components can rely on:

- Traversal is expressed as commands: `focus.next` and `focus.previous`.
- The runtime decides the traversal scope root based on the active modal barrier.
- Components may implement higher-level focus policies (trap/restore/initial focus) by:
  - choosing when to dispatch traversal commands,
  - installing modal barriers for modal surfaces,
  - intercepting traversal commands in specific subtrees (future FocusScope policy, component-owned).

This ADR does not introduce a shadcn/Radix-style `FocusScope` component in the runtime. A headless
`FocusScope` policy belongs in `fret-ui-kit` (see ADR 0067).

### 2) Modal barrier defines the traversal scope root (P0)

When a modal barrier is active (ADR 0066):

- Traversal is restricted to the active input layer set (barrier root and any roots above it).
  - This supports portal patterns (e.g. a combobox popover inside a dialog) where the nested popup lives in a
    separate overlay root above the modal barrier.
  - Focusable candidates in underlay roots are ignored.

When no barrier is active:

- Traversal considers all active input roots (base root + visible overlay roots).

This matches the core modal invariant: underlay UI is inert to pointer/keyboard/focus while modal.

### 3) Candidate set and ordering (P0, conservative)

For MVP, focus traversal uses a conservative candidate set:

- Only nodes in the active input layers are considered (overlay-aware).
- A node is a candidate if its widget reports `is_focusable() == true`.
- Candidates must have non-zero bounds.
- Candidates must intersect the traversal scope bounds (visibility-biased).

Ordering:

- Candidates are collected in a deterministic order:
  - traverse active input roots in reverse paint order (top -> bottom),
  - within each root, collect focusables in a deterministic pre-order traversal.
- `focus.next` moves to the next candidate (wraps).
- `focus.previous` moves to the previous candidate (wraps).

Rationale:

- Fret does not yet have a stable `scroll-into-view` contract; focusing offscreen nodes would often
  be invisible and confusing.
- This keeps traversal deterministic and predictable while we scale core components.

### 4) Reserved extension: Focus scopes + scroll-into-view (P1)

We expect two follow-ups once the runtime substrate grows:

1) **Focus scopes (policy, component-owned)**:
   - Trap focus within a subtree for modal-like surfaces that are not full barriers, or for nested focus groups.
   - Align with Radix `FocusScope` outcomes.

2) **Scroll-into-view (mechanism)**:
   - A stable way for a scroll container to ensure a newly focused descendant becomes visible.
   - Once available, traversal can include offscreen candidates without breaking UX.

## References

- Focus + command routing: `docs/adr/0020-focus-and-command-routing.md`
- Multi-root overlays: `docs/adr/0011-overlays-and-multi-root.md`
- Runtime contract surface + modal barrier semantics: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Overlay policy architecture (FocusScope belongs to components): `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Focus-visible heuristic + rings: `docs/adr/0061-focus-rings-and-focus-visible.md`
- Behavior reference stack (APG/Radix): `docs/reference-stack-ui-behavior.md`
- Zed/GPUI (non-normative):
  - tab-stop based traversal with stable ordering groups:
    `repo-ref/zed/crates/gpui/src/tab_stop.rs` (`TabStopMap`)
  - window-level traversal entry points:
    `repo-ref/zed/crates/gpui/src/window.rs` (`focus_next`, `focus_prev`)
