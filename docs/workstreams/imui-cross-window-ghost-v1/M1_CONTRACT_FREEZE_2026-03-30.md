# M1 Contract Freeze — 2026-03-30

Status: accepted v1 decision

Related:

- `docs/workstreams/imui-cross-window-ghost-v1/DESIGN.md`
- `docs/workstreams/imui-cross-window-ghost-v1/TODO.md`
- `docs/workstreams/imui-cross-window-ghost-v1/MILESTONES.md`
- `docs/workstreams/imui-drag-preview-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`
- `docs/workstreams/docking-hovered-window-contract-v1/docking-hovered-window-contract-v1.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- `ecosystem/fret-ui-kit/src/recipes/imui_drag_preview.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `crates/fret-runtime/src/drag.rs`
- `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`

## Purpose

This note closes the Phase B question set for `imui-cross-window-ghost-v1`.

The goal is to freeze the smallest correct contract before implementation drift starts:

- which layer owns generic cross-window ghost policy,
- which window paints the ghost,
- how the ghost hands off between windows without duplicates,
- whether preview meaning transfers through a descriptor,
- and how the contract degrades when multi-window hover capability is absent or weak.

## Frozen decisions

### 1) The generic public surface stays recipe-owned, but shell choreography does not

Decision:

- keep `ecosystem/fret-ui-kit::imui` limited to drag observation seams,
- allow `ecosystem/fret-ui-kit::recipes` to own one generic cross-window ghost transfer helper,
- keep docking/workspace moving-window choreography in shell-aware layers,
- and do not widen runtime crates unless the first proof shows a real mechanism gap.

This means the next generic surface is still above `imui`, not inside it.

It also means the existing same-window helper must **not** silently grow shell behavior.
If the generic cross-window recipe needs a different public shape, introduce a distinct recipe
surface or rename the current one instead of hiding the contract change behind compatibility
options.

### 2) Exactly one window paints the generic ghost: the drag session `current_window`

Decision:

- the generic cross-window ghost is painted only by the window whose `AppWindowId` equals
  `DragSession.current_window`,
- `source_window` remains the semantic owner of payload meaning and preview intent,
- and `moving_window` / `window_under_moving_window` remain shell diagnostics/arbitration hints,
  not the generic ghost owner rule.

Why:

- source-only painting cannot produce a correct cross-window outcome because the existing overlay
  root is clipped to the source window,
- and `DragSourceResponse::position()` is only meaningful for the current drag window, not for an
  arbitrary non-current window.

So the correct generic rule is:

- semantic ownership stays with the source,
- paint ownership follows `current_window`.

### 3) Handoff is immediate, and duplicate ghosts are forbidden

Decision:

- the previous owner hides its ghost as soon as `current_window` changes away from it,
- the new owner may paint on the next frame using the transferred descriptor,
- and the generic layer must never intentionally show two ghosts for the same drag session.

Operational rule:

- while `current_window == source_window`, the source-side same-window ghost may render,
- once `current_window != source_window`, the source-side generic ghost must stop rendering,
- and only the current hovered/target window may render the transferred ghost.

There is no “keep the source ghost until transfer confirmation” grace rule in v1.
That rule would create duplicate windows-of-truth and is explicitly rejected.

### 4) Cross-window rendering requires a transferred preview descriptor, not source-only painting

Decision:

- the hovered/current window renders from a recipe-owned transferred preview descriptor,
- the source publishes that descriptor while it is authoring the drag source,
- and the descriptor carries only the minimal render intent needed for the ghost.

The descriptor may include:

- preview content/render closure or equivalent recipe-owned render intent,
- stable ghost options such as offset/chrome/test id,
- and session identity needed to avoid duplicates and stale reuse.

The descriptor must not include:

- shell choreography decisions,
- target lookup logic,
- or a global payload skin registry detached from source authoring.

This keeps the current architectural win intact:

- the source still decides what the ghost means,
- but the hovered/current window becomes the paint host when the drag leaves the source window.

### 5) Capability fallback is explicit and layer-local

Decision:

- `Reliable` multi-window hover routing gets the full generic cross-window transfer contract,
- `BestEffort` may use the same recipe transfer rule, but only by following the runner's
  best-effort `current_window` selection,
- `None` or `multi_window = false` disables generic transfer and falls back to same-window-only
  source ghost behavior.

Important rule:

- the recipe layer must not invent its own cross-window hover heuristics.

It may only consume the already-routed drag session state:

- `source_window`,
- `current_window`,
- `cross_window_hover`,
- and the current drag position.

When the runner cannot name an eligible current Fret window, the generic recipe paints no
cross-window ghost.
Shell layers may still choose to show a utility/follow-window preview, but that is a separate shell
contract and must not be hidden inside the generic recipe.

## Rejected alternatives

### Source window always paints

Rejected because:

- the source window cannot paint outside its own overlay root,
- clipping makes the ghost disappear or appear in the wrong place,
- and the existing position seam is not source-window-local once routing switches windows.

### Shell-only ownership for every cross-window ghost

Rejected because:

- there is still a real generic contract for ordinary multi-window immediate drags,
- forcing every app/shell to reinvent preview transfer would duplicate policy immediately,
- and the shell-specific problem is moving-window choreography, not the entire transfer model.

### Runtime-owned preview registry

Rejected for v1 because:

- preview meaning belongs to source authoring, not to runtime payload transport,
- and there is no proof yet that runtime needs a preview-specific mechanism beyond the existing drag
  session routing state.

## Immediate consequences

From this point forward:

1. treat `current_window` as the generic cross-window ghost paint owner,
2. treat `source_window` as semantic owner only,
3. keep the transfer descriptor in the recipe layer unless M2 proof shows a hard mechanism gap,
4. keep moving-window / transparent-payload / peek-behind behavior in shell-aware layers,
5. use `apps/fret-examples/src/imui_editor_proof_demo.rs` as the first proof surface for the next
   implementation slice.

## What Phase C now needs to prove

The next proof/gate package should verify:

- ghost ownership visibly hands off from source window to aux window,
- the previous window hides immediately enough that duplicates are impossible,
- the transferred ghost still reflects source-authored meaning,
- and single-window / degraded capability mode stays on the same-window contract without extra
  recipe heuristics.
