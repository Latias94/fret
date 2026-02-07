# Retained Bridge Exit Plan v1 — TODO Tracker

Status: Active (fearless refactor friendly; pre-1.0)

Related plan:

- `docs/workstreams/retained-bridge-exit-v1.md`

## Milestones

### M0 — Governance gates (blast radius control)

- [x] CI: reject `crates/* -> ecosystem/*` reverse dependencies (`tools/check_layering.ps1`).
- [x] CI: restrict `fret-ui/unstable-retained-bridge` to an explicit allowlist (`tools/check_layering.ps1`).
- [ ] Document the current allowlist and rationale per crate (docking/node/chart/plot).

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

