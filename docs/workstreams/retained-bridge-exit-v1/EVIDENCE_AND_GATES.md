# Retained Bridge Exit v1 Evidence and Gates

## 2026-05-18 - RBX-M1-010 Docking retained bridge audit

Claim verified:

- `fret-docking` retained bridge usage has been audited and classified.
- The first implementation slice has been selected as `RBX-M1-020`: extract docking split geometry
  and handle painting from `fret_ui::retained_bridge`.

Evidence:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_010_DOCKING_RETAINED_BRIDGE_AUDIT_2026-05-18.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking`
  - Result: passed, 111 tests.
  - Scope proven: existing docking behavior remains green after the audit/documentation update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: retained bridge allowlist and crate layering still pass.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed.
  - Scope proven: workstream catalog indexes remain valid.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed documentation has no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M1-010` is an audit/documentation task; the task-local docking gate and layering
    gate are sufficient.

## 2026-05-18 - RBX-M1-020 readiness checkpoint

Claim recorded:

- Retained bridge deletion remains the long-term target because declarative authoring is the
  primary Fret UI direction.
- `RBX-M1-020` should proceed as a docking-private extraction slice.
- Full deletion of `retained_bridge::resizable_panel_group` should wait if
  `apps/fret-examples/src/docking_arbitration_demo.rs` remains a consumer.

Evidence:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_020_READINESS_NOTE_2026-05-18.md`

Commands:

- `rg -n "resizable::|retained_bridge::resizable_panel_group|retained_bridge::ResizeHandle|ResizeHandle" apps/fret-examples/src/docking_arbitration_demo.rs ecosystem/fret-docking/src crates/fret-ui/src/retained_bridge.rs`
  - Result: found docking call sites and direct app/demo `resizable_panel_group` call sites.
  - Scope proven: deleting the bridge resizable helper in `RBX-M1-020` would widen the slice beyond
    `fret-docking`.

## 2026-05-18 - RBX-M1-020 docking-private split helper extraction

Claim verified:

- `fret-docking` no longer imports split geometry or handle painting through
  `fret_ui::retained_bridge`.
- No-user bridge exports/functions from this slice were deleted.
- `retained_bridge::resizable_panel_group::compute_layout` remains only because
  `apps/fret-examples/src/docking_arbitration_demo.rs` still consumes it; that follow-up is tracked
  as `RBX-M1-021`.

Evidence:

- `ecosystem/fret-docking/src/dock/split_geometry.rs`
- `ecosystem/fret-docking/src/dock/layout.rs`
- `ecosystem/fret-docking/src/dock/hit_test.rs`
- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `crates/fret-ui/src/retained_bridge.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean.
- `cargo nextest run -p fret-docking`
  - Result: passed, 111 tests.
  - Scope proven: docking split layout, hit-test, drag, drop preview, viewport, and runtime tests
    remain green after the helper extraction.
- `cargo clippy -p fret-docking --all-targets --no-deps -- -D warnings`
  - Result: passed.
  - Scope proven: touched `fret-docking` targets are warning-clean under clippy.
- `cargo check -p fret-demo --bin docking_arbitration_demo`
  - Result: passed.
  - Scope proven: the remaining app/demo bridge `compute_layout` consumer still compiles.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist still pass.
- `rg -n "retained_bridge::resizable_panel_group|retained_bridge::ResizeHandle|resizable::|ResizeHandle" ecosystem/fret-docking/src crates/fret-ui/src/retained_bridge.rs apps/fret-examples/src/docking_arbitration_demo.rs`
  - Result: only `apps/fret-examples/src/docking_arbitration_demo.rs` still directly uses
    `retained_bridge::resizable_panel_group`; no `fret-docking` call sites remain and
    `retained_bridge::ResizeHandle` is gone.
  - Scope proven: `RBX-M1-020` completed the docking-private extraction and identified the remaining
    app/demo follow-up.
- `rg -n "retained_bridge::ResizablePanelGroupStyle|retained_bridge::ResizablePanelGroupLayout|fret_ui::retained_bridge::\\{[^\\n]*ResizablePanelGroup|pub use crate::resize_handle::ResizeHandle" crates ecosystem apps -g '*.rs'`
  - Result: no direct repo consumers.
  - Scope proven: `retained_bridge::ResizablePanelGroupStyle` was safe to delete; the layout type is
    still retained only because `retained_bridge::resizable_panel_group::compute_layout` returns it.
- `rg -n "pub fn drag_update_fractions|pub fn drag_update_adjacent_fractions|pub use crate::resizable_panel_group::ResizablePanelGroupStyle" crates/fret-ui/src/retained_bridge.rs`
  - Result: no matches.
  - Scope proven: the no-user bridge drag helpers and style re-export were deleted.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

## 2026-05-18 - RBX-M1-021 demo diagnostics split helper migration

Claim verified:

- `apps/fret-examples/src/docking_arbitration_demo.rs` no longer depends on
  `fret_ui::retained_bridge::resizable_panel_group` for diagnostics split geometry.
- The remaining `retained_bridge::resizable_panel_group` helper module and
  `retained_bridge::ResizablePanelGroupLayout` re-export were deleted after repo-wide no-user
  proof.

Evidence:

- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `crates/fret-ui/src/retained_bridge.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the demo and bridge edits.
- `cargo check -p fret-demo --bin docking_arbitration_demo`
  - Result: passed.
  - Scope proven: the docking arbitration demo still compiles after migrating diagnostics geometry
    off the retained bridge helper.
- `cargo clippy -p fret-demo --bin docking_arbitration_demo --no-deps -- -D warnings`
  - Result: passed.
  - Scope proven: the touched demo target remains warning-clean under clippy.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering remains valid after shrinking the retained bridge surface.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.
- `rg -n "retained_bridge::resizable_panel_group|retained_bridge::ResizablePanelGroupLayout|resizable::compute_layout" crates ecosystem apps -g '*.rs'`
  - Result: no matches.
  - Scope proven: no Rust source still consumes the removed retained bridge split helper or
    retained bridge panel-group layout re-export.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M1-021` is a targeted demo diagnostics migration plus bridge surface deletion; the
    task-local demo compile/clippy gates and retained-bridge no-user proof cover the changed
    behavioral surface.
