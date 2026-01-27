# Input Dispatch v2 — Backport Plan (input-dispatch-v2 worktree)

Status: Draft (worktree-local; intended to reduce churn when backporting to `main`)

This document tracks how to backport the Input Dispatch v2 and overlay arbitration work from
`input-dispatch-v2` worktrees into `main` without losing bisectability or creating large conflict
surfaces.

Guiding principles:

- Prefer small, themed cherry-picks over a single mega-merge.
- Keep tests landing with the behavior they lock.
- Land runtime/mechanism contracts before ecosystem policies.
- Avoid touching unrelated areas (e.g. `tools/fret-bundle-viewer/`) unless a change is required by
  the contract work.

## How to generate the candidate list

From the worktree branch:

- `git log --cherry --right-only --no-merges --oneline main...HEAD`

Use that output as the “source of truth” for what is still missing on `main` (patch-id aware).

## Recommended backport bundles (top → bottom)

### Bundle A — Command gating snapshot stack (runner-facing parity)

Goal: make menus / command palette / shortcut help share the same “frozen gating” semantics.

- Runtime: `WindowCommandGatingService` stack (`push_snapshot`/`pop_snapshot`) and base-vs-stack tests.
  - Evidence: `crates/fret-runtime/src/window_command_gating.rs`
- Bootstrap: command palette uses `push_snapshot` and stores a handle until close.
  - Evidence: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- Runner: native menus prefer `WindowCommandGatingService::snapshot(window)` when present.
  - Evidence: `crates/fret-launch/src/runner/desktop/{windows_menu.rs,macos_menu.rs}`

Validation:

- `cargo nextest run -p fret-runtime --lib`
- `cargo check -p fret-bootstrap`
- `cargo check -p fret-launch`

### Bundle B — Mechanism ergonomics (clarify base snapshot API)

Goal: reduce future refactor pressure by making the “base snapshot” intent explicit.

- Add `base_snapshot` / `set_base_snapshot` / `clear_base_snapshot` helpers (aliases remain).
  - Evidence: `crates/fret-runtime/src/window_command_gating.rs`

Validation:

- `cargo nextest run -p fret-runtime --lib`

### Bundle C — Overlay close-transition invariants (Radix hand-feel)

Goal: lock “present vs interactive” behavior and observer/timer invariants during close transitions.

- UI-kit policy: click-through close transitions; observers/timers disabled while closing.
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/{state.rs,render.rs}`
- Conformance: shadcn tests cover tooltip/hover-card/menu close transitions.
  - Evidence: `ecosystem/fret-ui-shadcn/src/{tooltip.rs,hover_card.rs,dropdown_menu.rs,context_menu.rs,menubar.rs}`

Validation:

- `cargo nextest run -p fret-ui-kit --lib`
- `cargo nextest run -p fret-ui-shadcn --lib`

### Bundle D — Menu modality + entry focus + auto-focus hooks

Goal: pointer-open vs keyboard-open focus behavior matches Radix, and is customizable via hooks.

- Menu root policy: modality-gated initial focus + `MenuInitialFocusTargets`.
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/menu/root.rs`
- Auto-focus hooks: `onOpenAutoFocus` / `onCloseAutoFocus` support preventDefault.
  - Evidence: `crates/fret-ui/src/action.rs`, `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- Shadcn surfaces wire hooks and have conformance tests.
  - Evidence: `ecosystem/fret-ui-shadcn/src/{dropdown_menu.rs,menubar.rs,context_menu.rs,dialog.rs,popover.rs}`

Validation:

- `cargo nextest run -p fret-ui-shadcn --lib`

### Bundle E — Select/Combobox modality focus + click-through preventDefault conformance

Goal: keep “menu-like” overlays consistent across surfaces (select/combobox/context menu/dropdown menu),
including the subtle “prevent default dismissal but still allow click-through under `modal=false`” cases.

- UI-kit: Select initial focus targets (pointer-open vs keyboard-open).
  - Evidence: `ecosystem/fret-ui-kit/src/primitives/select.rs`
- Shadcn: Select wires initial focus targets + conformance tests.
  - Evidence: `ecosystem/fret-ui-shadcn/src/select.rs`
- Shadcn: Combobox open auto-focus focuses the search input (pointer-open and keyboard-open).
  - Evidence: `ecosystem/fret-ui-shadcn/src/combobox.rs`
- Shadcn: Click-through (`modal=false`) outside-press can be prevented without blocking underlay activation.
  - Evidence:
    - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
    - `ecosystem/fret-ui-shadcn/src/context_menu.rs`
    - Tracker: `docs/workstreams/overlay-input-arbitration-v2-todo-input-dispatch-v2.md` (OIA2-test-039..041)

Validation:

- `cargo nextest run -p fret-ui-shadcn --lib`
- (Optional) `cargo nextest run -p fret-ui-kit --lib`

## Notes / open items

- Keep overlay arbitration docs split from Input Dispatch v2 contracts:
  - Input Dispatch v2 TODO tracker: `docs/workstreams/input-dispatch-v2-todo.md`
  - Overlay arbitration TODO tracker: `docs/workstreams/overlay-input-arbitration-v2-todo.md`
  - Worktree overlay tracker: `docs/workstreams/overlay-input-arbitration-v2-todo-input-dispatch-v2.md`
