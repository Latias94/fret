# RBX-M1-020 Readiness Note

Date: 2026-05-18
Status: Decision checkpoint; Option A implemented
Workstream: `retained-bridge-exit-v1`
Task: `RBX-M1-020`

## Strategic Stance

Declarative authoring is the primary Fret UI direction. `fret_ui::retained_bridge` should be treated
as a temporary compatibility path, not as a long-term public authoring API.

This does **not** mean deleting retained runtime semantics. The runtime may still keep retained
state internally for focus, IME, caching, hit testing, layout, and paint orchestration. The thing to
delete is the ecosystem-facing escape hatch that lets app/component code keep authoring retained
widgets or retained subtrees.

## Fresh Evidence

Current split-helper bridge consumers:

- `ecosystem/fret-docking/src/dock/layout.rs`
- `ecosystem/fret-docking/src/dock/hit_test.rs`
- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `ecosystem/fret-docking/src/dock/tests/mod.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`

Current `ResizeHandle` bridge consumer:

- `ecosystem/fret-docking/src/dock/paint.rs`

Important wrinkle:

- `apps/fret-examples/src/docking_arbitration_demo.rs` imports
  `fret_ui::retained_bridge::resizable_panel_group as resizable` directly for diagnostics/harness
  anchor geometry.
- Therefore, migrating only `fret-docking` will **not** make
  `retained_bridge::resizable_panel_group` deletable yet.

## Decision Options

### Option A - Do RBX-M1-020 now, but keep it docking-private

Scope:

- Add docking-private split geometry / handle paint helpers under `ecosystem/fret-docking/src/dock/`.
- Migrate `fret-docking` source and tests off `retained_bridge::resizable_panel_group` and
  `retained_bridge::ResizeHandle`.
- Keep `retained_bridge::resizable_panel_group` if `apps/fret-examples` still uses it.
- Remove `retained_bridge::ResizeHandle` only if repository-wide `rg` proves it has no remaining
  users after docking moves.

Pros:

- Shrinks the most important bridge client immediately.
- Avoids widening this slice into the diagnostics demo.
- Keeps the work behavior-preserving and easy to gate with `cargo nextest run -p fret-docking`.

Cons:

- Does not fully delete the bridge split-helper module yet.
- Leaves one app/demo retained bridge geometry consumer to handle in a follow-up.

### Option B - Do full helper deletion now

Scope:

- Do everything in Option A.
- Also migrate `apps/fret-examples/src/docking_arbitration_demo.rs` off
  `retained_bridge::resizable_panel_group`.
- Delete the bridge resizable helper exports if no users remain.

Pros:

- More directly shrinks `crates/fret-ui/src/retained_bridge.rs`.
- Avoids carrying a half-complete helper extraction.

Cons:

- Expands the slice from a docking crate refactor into app diagnostics harness work.
- The app harness currently duplicates docking geometry for stable diagnostic anchors; changing it
  deserves its own focused verification target.

### Option C - Defer RBX-M1-020

Scope:

- Leave implementation unchanged for now.
- Wait until a declarative docking host or diagnostics anchor replacement is designed.

Pros:

- Avoids churn before the bigger declarative host path is clearer.

Cons:

- Keeps `retained_bridge` looking useful for non-retained split geometry.
- Misses a low-risk chance to shrink docking's bridge usage.

## Recommendation

Proceed now with Option A.

Reason:

- It is the smallest useful refactor that matches the declarative-first direction.
- It removes bridge usage that is unrelated to retained widget lifecycle.
- It does not pretend the bridge is fully deletable while the diagnostics demo still consumes the
  split helper.

Open the follow-up only after Option A lands:

- `RBX-M1-021`: migrate `docking_arbitration_demo` diagnostics geometry off
  `retained_bridge::resizable_panel_group`, then delete the bridge resizable helper module if the
  repository-wide search is empty.

## Outcome

Option A landed on 2026-05-18:

- `fret-docking` source/tests no longer use `retained_bridge::resizable_panel_group` or
  `retained_bridge::ResizeHandle`.
- `retained_bridge::ResizeHandle` and the no-user bridge drag update helpers were deleted.
- `retained_bridge::resizable_panel_group::compute_layout` remains for
  `apps/fret-examples/src/docking_arbitration_demo.rs` and is tracked by `RBX-M1-021`.

## Gates

For Option A:

- `cargo nextest run -p fret-docking`
- `python3 tools/check_layering.py`
- `rg -n "retained_bridge::resizable_panel_group|retained_bridge::ResizeHandle" ecosystem/fret-docking crates apps`
- `git diff --check`

For Option B or `RBX-M1-021`:

- Option A gates.
- A targeted compile/test command that covers `apps/fret-examples/src/docking_arbitration_demo.rs`.
