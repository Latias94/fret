# Fret Launch + App Surface (Fearless Refactor v1) 鈥?Milestones

This workstream is staged to keep the launch stack landable while tightening public contracts.

## M0 鈥?Audit captured + documentation aligned

**Outcome**

- A dedicated workstream folder exists with design, milestones, and TODO documents.
- The current surface split (`fret`, `fret-framework`, `fret-launch`) is documented clearly.
- Known hazards are recorded before code changes begin.

**Gates**

- Links from `docs/README.md` point to this folder.
- The design doc includes evidence anchors for all major claims.

## M1 鈥?Export inventory and contract classification

**Outcome**

- Every root export from `crates/fret-launch/src/lib.rs` is classified as:
  - stable public,
  - transitional public,
  - internal plumbing.
- `fret` wrappers/re-exports are mapped to the minimum lower-level launch surface they require.

**Evidence anchors**

- `crates/fret-launch/src/lib.rs`
- `crates/fret-launch/README.md`
- `ecosystem/fret/src/lib.rs`

**Gates**

- A reviewable export table exists in the implementation PR or linked audit note.
- No new launch exports are added without classification.

## M2 鈥?Single advanced driver recommendation

**Outcome**

- `FnDriver` becomes the clearly documented advanced driver path.
- Any remaining `WinitAppDriver`-only requirements are either:
  - moved into `FnDriverHooks`, or
  - explicitly justified as compatibility surface.
- `fret-framework::launch` stops re-exporting compatibility-only `WinitAppDriver`.

**Evidence anchors**

- `crates/fret-launch/src/runner/common/fn_driver.rs`
- `crates/fret-launch/src/runner/common/winit_app_driver.rs`
- `apps/fret-examples/src/chart_demo.rs`
- `apps/fret-examples/src/bars_demo.rs`
- `apps/fret-examples/src/error_bars_demo.rs`

**Gates**

- `cargo nextest run -p fret-launch`
- Any touched docs/examples build or type-check if compile-checked in the relevant crate.
- Representative advanced examples prefer `FnDriver` over bespoke `WinitAppDriver` impls.
- Any remaining direct `WinitAppDriver` examples are verified to stay within current `FnDriver` hook coverage until they migrate.
- `python tools/gate_fret_launch_root_surface_snapshot.py`
- `python tools/gate_fret_framework_launch_surface.py`
- `python tools/gate_fn_driver_example_naming.py`
- `python tools/gate_winit_driver_example_hook_coverage.py`

## M3 鈥?Config curation without capability loss

**Outcome**

- Launch configuration is documented in app-facing vs backend-heavy groups.
- Beginner-facing docs stop teaching low-level tuning by default.
- Advanced host-integration knobs remain reachable.

**Evidence anchors**

- `crates/fret-launch/src/runner/common/config.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/app_entry.rs`

**Gates**

- `cargo nextest run -p fret`
- No regression in examples that depend on GPU init customization or window-create hooks.

## M4 鈥?Cross-surface docs and naming closure

**Outcome**

- `fret`, `fret-framework`, and `fret-launch` each have a distinct one-line role statement.
- The app-author path and integration path are both documented with minimal ambiguity.
- Docs stop implying that large internal launch namespaces are stable by accident.

**Evidence anchors**

- `docs/README.md`
- `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- `crates/fret-framework/src/lib.rs`
- `crates/fret-launch/README.md`
- `tools/gate_fret_builder_only_surface.py`
- `tools/gate_fret_framework_launch_surface.py`
- `tools/gate_fn_driver_example_naming.py`

**Gates**

- `cargo nextest run -p fret -p fret-launch -p fret-framework`
- `python tools/check_layering.py`
- `python tools/gate_fret_builder_only_surface.py`
- `python tools/gate_fret_framework_launch_surface.py`
- `python tools/gate_fn_driver_example_naming.py`

## M5 鈥?Optional follow-up: web/high-level symmetry

**Outcome**

- We make an explicit decision on whether `fret` should expose a peer high-level web entry surface.
- If not, docs say so clearly; if yes, a separate workstream owns the design.

**Constraint**

- This is not required to land the core launch/public-surface cleanup.

**Gates**

- Decision recorded in docs with a clear scope boundary.
