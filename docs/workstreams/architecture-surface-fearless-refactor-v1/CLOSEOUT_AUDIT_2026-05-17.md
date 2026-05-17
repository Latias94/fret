# Architecture Surface Fearless Refactor v1 — Closeout Audit

Status: Closed

Last updated: 2026-05-17

## Scope

This closeout covers the architecture-surface lane opened to narrow public app-authoring,
bootstrap, ecosystem taxonomy, shared menu/select policy, and renderer facade ownership.

Compatibility with old in-repo interfaces was explicitly a non-goal for this lane.

## Findings

### 1. Backend-free app authoring now has an honest dependency profile

`fret --no-default-features` and `fret --no-default-features --features app` no longer imply the
native launch/render/backend stack. The `desktop` feature now owns native runner/render wiring, while
`app` remains a backend-free authoring baseline.

Evidence:

- `ecosystem/fret/Cargo.toml`
- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret/tests/backend_free_app_authoring_profile.rs`
- `tools/check_consumption_profiles.py`

### 2. Bootstrap planning is separated from concrete launch/render adapters

`fret-bootstrap --no-default-features` now exposes backend-free planning/default policy without
pulling `fret-launch`, `fret-render`, `wgpu`, `winit`, native platform crates, or runner crates.
Concrete launch/render behavior is owned by the launch adapter lane.

Evidence:

- `ecosystem/fret-bootstrap/src/assets.rs`
- `ecosystem/fret-bootstrap/tests/backend_free_bootstrap_profile.rs`
- `docs/crate-usage-guide.md`

### 3. The `fret` app-facing surface is narrower and owner-oriented

The app prelude is now a closed Golden Path budget. LocalState and related tracked-state helpers now
have a private owner module while keeping the existing public re-export path.

Evidence:

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/src/view/local_state.rs`
- `docs/crate-usage-guide.md`

### 4. Ecosystem taxonomy has concrete proof slices

Boolean controls prove the headless/primitives/recipe split for pure optional-bool state, and the
carousel recipe consumes headless engines directly instead of routing pure behavior through broad kit
shims.

Evidence:

- `ecosystem/fret-ui-headless/src/boolean_control.rs`
- `ecosystem/fret-ui-kit/src/primitives/{checkbox.rs,switch.rs}`
- `ecosystem/fret-ui-shadcn/src/{checkbox.rs,switch.rs,carousel.rs}`
- `docs/audits/shadcn-carousel.md`

### 5. Menu/select shared policy has a real owner and a follow-on lane

Input-modality-gated entry-focus target selection now lives in `fret-ui-headless::entry_focus` and
is consumed by menu/select runtime adapters. Remaining shadcn menu/select policy cleanup is split to
`docs/workstreams/shadcn-menu-select-policy-followon-v1/`.

Evidence:

- `ecosystem/fret-ui-headless/src/entry_focus.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/root.rs`
- `ecosystem/fret-ui-kit/src/primitives/select.rs`
- `ecosystem/fret-ui-shadcn/src/select.rs`
- `docs/workstreams/shadcn-menu-select-policy-followon-v1/`

### 6. The renderer facade decision is closed

`fret-render` remains the curated default renderer facade. It should not be collapsed into
`fret-render-wgpu`. The closed renderer-modularity lane already locked the facade buckets,
backend-specific diagnostics escape hatch, and host-provided GPU topology proof.

Evidence:

- `crates/fret-render/src/lib.rs`
- `crates/fret-render/tests/facade_surface_snapshot.rs`
- `crates/fret-render-wgpu/tests/host_provided_gpu_topology_smoke.rs`
- `docs/workstreams/renderer-modularity-fearless-refactor-v1/CLOSEOUT_AUDIT.md`
- `docs/workstreams/architecture-surface-fearless-refactor-v1/JOURNAL/2026-05-17-asf-070.md`

## Final Gates

The lane records task-specific gates in `EVIDENCE_AND_GATES.md` and journal notes. Closeout used the
following final checks:

- `python tools/check_layering.py`
- `python tools/check_consumption_profiles.py`
- `python tools/check_workstream_catalog.py`
- `git diff --check`

All four commands passed on 2026-05-17.

## Follow-ons

- `docs/workstreams/shadcn-menu-select-policy-followon-v1/` owns remaining shadcn menu/select
  policy work, starting with the pointer-open ArrowDown select contract conflict.
- Future renderer semantic/capability work should open a renderer-specific workstream. Do not
  reopen this architecture-surface lane or the closed renderer-modularity v1 lane just to continue
  splitting renderer internals.

## Closure Decision

Close `architecture-surface-fearless-refactor-v1` as complete. Remaining work has narrower owners,
and no architecture-surface task remains open in this lane.
