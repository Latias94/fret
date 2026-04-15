# IMUI Immediate LocalState Bridge Owner Audit — 2026-04-15

Status: landed follow-on audit
Last updated: 2026-04-15

Related:

- `TODO.md`
- `MILESTONES.md`
- `ecosystem/fret/src/view.rs`
- `apps/fret-examples/src/imui_hello_demo.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

## Why this note exists

After the grouped query-maintenance, cookbook theme-owner, render-root model-read, stress-root, and
GenUI message-lane slices landed, the remaining `app.models()` grep tail in first-party examples
was no longer one backlog item.

The IMUI-related remainder mixed two different owners:

- immediate-mode UI closures that already own `LocalState<T>` plus `ui.cx_mut()` as an
  `ElementContext` bridge,
- pure `&mut App` / driver-side helpers that intentionally still own raw `ModelStore`.

This note records which side of that split should move and which should stay explicit.

## Assumptions

1. `imui_hello_demo` and `imui_interaction_showcase_demo` are immediate-mode bridge surfaces, not
   driver-owned raw model examples.
   Evidence: both files allocate state with `cx.state().local_init(...)` and mount IMUI through
   `fret_imui::imui_in(...)` or `fret_imui::imui(...)`, where `ui.cx_mut()` already exposes the
   helper-heavy `ElementContext` bridge.
   Confidence: Confident.
   Consequence if wrong: migrating reads away from `app.models()` would hide an intentional raw
   store teaching surface.

2. No new IMUI-specific framework API is required for this slice.
   Evidence: `LocalState<T>` already exposes `layout_value_in(...)` and `paint_value_in(...)` in
   `ecosystem/fret/src/view.rs` precisely for helper-heavy `ElementContext` surfaces.
   Confidence: Confident.
   Consequence if wrong: this slice would add redundant surface instead of using the already-landed
   bridge owner.

3. `push_showcase_event(...)` and the remaining `imui_editor_proof_demo` raw model reads are not
   the same owner as the UI closures above.
   Evidence: `push_showcase_event(...)` only owns `&mut KernelApp`, and the remaining
   `imui_editor_proof_demo` reads sit on app/driver or retained orchestration paths rather than
   inside ordinary `LocalState`-owned IMUI closures.
   Confidence: Likely.
   Consequence if wrong: a later cleanup might either over-generalize this audit or accidentally
   flatten legitimate driver/model orchestration onto UI-local helper APIs.

## Owner resolution

### 1. IMUI closure-local `LocalState` reads

Owner: `LocalState<T>` bridge helpers on `ElementContext`.

Decision:

- `imui_hello_demo` should read the toggled checkbox value with
  `enabled_state.paint_value_in(ui.cx_mut())`.
- `imui_interaction_showcase_demo` should read bookmark/tool/tab/review/toggle/exposure state with
  `layout_value_in(ui.cx_mut())`.

Why:

- the state is already allocated as `LocalState<T>`,
- the closure already owns `ui.cx_mut()` as the correct bridge surface,
- and raw `ui.cx_mut().app.models()` only bypasses an existing typed owner.

### 2. App/driver helpers outside IMUI closures

Owner: raw `app.models()` / `app.models_mut()`.

Decision:

- keep `push_showcase_event(...)` on raw `ModelStore`,
- keep the remaining `imui_editor_proof_demo` app/driver reads out of this slice.

Why:

- those call sites do not own `ElementContext`,
- they are not ordinary closure-local IMUI reads,
- and moving them onto closure-oriented helpers would blur the UI-vs-driver owner split.

## Landed result

This audit lands:

- `imui_hello_demo` now uses `LocalState::paint_value_in(...)` inside the IMUI closure instead of
  raw `ui.cx_mut().app.models()`.
- `imui_interaction_showcase_demo` now uses `LocalState::layout_value_in(...)` for closure-local
  bookmark/tool/autosave/exposure/review/tab/context reads instead of raw
  `ui.cx_mut().app.models()`.
- `apps/fret-examples/src/lib.rs` now includes a dedicated source-policy gate that locks this
  owner split while still allowing `push_showcase_event(...)` to keep its explicit raw app-owned
  store access.

## Decision from this audit

Treat IMUI closure-local reads the same way as other helper-heavy `ElementContext` surfaces:

- UI-local `LocalState<T>` reads belong on `layout_value_in(...)` / `paint_value_in(...)`,
- pure app/driver helpers keep raw `app.models()` access,
- and this lane should not grow new IMUI-specific sugar unless real evidence shows that the
  existing `LocalState` bridge still cannot express multiple product surfaces cleanly.
