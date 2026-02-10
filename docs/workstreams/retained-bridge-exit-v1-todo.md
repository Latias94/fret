# Retained Bridge Exit Plan v1 — TODO Tracker

Status: Active (fearless refactor friendly; pre-1.0)

Related plan:

- `docs/workstreams/retained-bridge-exit-v1.md`

## Milestones

### M0 — Governance gates (blast radius control)

- [x] CI: reject `crates/* -> ecosystem/*` reverse dependencies (`tools/check_layering.py`).
- [x] CI: restrict `fret-ui/unstable-retained-bridge` to an explicit allowlist (`tools/check_layering.py`).
- [x] Document the current allowlist and rationale per crate (docking/node/chart/plot).
  - Source of truth: `tools/check_layering.py` (`unstable_retained_bridge_allowlist`).
  - Current allowlist (workspace crate names):
    - `fret-docking`
      - Why: hosts retained subtrees for docking UI and reuses retained helpers (e.g. resizable panel group sizing / capture / hit-test policy) while the declarative surface is still closing.
      - Evidence: `ecosystem/fret-docking/Cargo.toml` enables `fret-ui/unstable-retained-bridge`; retained hosting in `ecosystem/fret-docking/src/imui.rs`.
      - Exit target: M1 (primary target).
    - `fret-node`
      - Why: node graph canvas + portal editors are still authored as retained widgets; it also exercises overlays/commands in the retained path.
      - Evidence: `ecosystem/fret-node/Cargo.toml` enables `fret-ui/unstable-retained-bridge`; retained widget surface in `ecosystem/fret-node/src/ui/canvas/widget.rs`.
      - Exit target: M2.
    - `fret-chart`
      - Why: retained canvas widget used for interactive charts; still depends on retained layout/paint/event wiring.
      - Evidence: `ecosystem/fret-chart/Cargo.toml` enables `fret-ui/unstable-retained-bridge`; retained canvas in `ecosystem/fret-chart/src/retained/canvas.rs`.
      - Exit target: M3.
    - `fret-plot`
      - Why: retained plotting surfaces still use `RetainedSubtreeProps` and retained canvas widgets for performance/interaction while declarative authoring migrates.
      - Evidence: `ecosystem/fret-plot/Cargo.toml` enables `fret-ui/unstable-retained-bridge`; retained subtree hosting in `ecosystem/fret-plot/src/imui.rs` and retained canvas in `ecosystem/fret-plot/src/retained/canvas/mod.rs`.
      - Exit target: M3.
    - `fret-plot3d`
      - Why: retained 3D plot surface uses retained viewport-surface helpers and widget lifecycle plumbing.
      - Evidence: `ecosystem/fret-plot3d/Cargo.toml` enables `fret-ui/unstable-retained-bridge`; retained widget in `ecosystem/fret-plot3d/src/retained.rs`.
      - Exit target: M3.

### M1 — Docking declarative closure (primary target)

- [ ] Identify the minimal declarative primitives missing for docking (if any).
- [ ] Replace retained subtree hosting in docking with declarative composition where feasible.
- [ ] Add/upgrade `fretboard diag` scripts to lock in docking drag + tear-off correctness.
- [ ] Remove `unstable-retained-bridge` from `ecosystem/fret-docking` dependencies.

### M2 — Node graph migration

- [ ] Split node graph into:
  - declarative composition for chrome/overlays/panels,
  - `Canvas`/`ViewportSurface`-style leaf for heavy rendering where needed.
- [ ] Remove `unstable-retained-bridge` from `ecosystem/fret-node` dependencies.

### M3 — Charts/plots migration

- [ ] Convert chart/plot surfaces to `Canvas`-first declarative authoring.
- [ ] Remove `unstable-retained-bridge` from `ecosystem/fret-chart`, `ecosystem/fret-plot`, `ecosystem/fret-plot3d`.

### M4 — Bridge shrink and delete (or quarantine)

- [ ] Audit `crates/fret-ui/src/retained_bridge.rs` exports; delete anything not required by remaining clients.
- [ ] If allowlist becomes empty: remove `fret-ui/unstable-retained-bridge` feature and all bridge code.
- [ ] Otherwise: quarantine the remaining retained path behind a narrower, clearly named compatibility facade with
  explicit “do not grow” policy and separate tracking.
