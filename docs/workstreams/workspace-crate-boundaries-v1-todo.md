# Workspace Crate Boundaries Audit v1 — TODO Tracker

Status: Active (fearless refactor friendly; pre-1.0)

Related plan:

- `docs/workstreams/workspace-crate-boundaries-v1.md`

## Milestones

### M0 — Governance and tracking

- [x] Add this workstream to `docs/workstreams/README.md`.
- [x] Add a short entry to `docs/todo-tracker.md` pointing to this workstream (optional).
- [ ] Confirm the intended wasm “golden path” bundles in `crates/fret` (`web` vs `web-winit`).

### M1 — Render split (`fret-render-core` + `fret-render-wgpu`)

- [ ] Create `crates/fret-render-core` (portable render contracts/types).
- [ ] Create `crates/fret-render-wgpu` (wgpu backend implementation).
- [ ] Keep `crates/fret-render` as a compatibility facade (re-exports / feature glue).
- [ ] Update `crates/fret-launch` to depend on `fret-render-wgpu` for native.
- [ ] Ensure `tools/check_layering.ps1` still passes.

### M2 — Web runner becomes real (`fret-runner-web`)

- [ ] Implement DOM/canvas event translation in `crates/fret-runner-web`.
- [ ] Provide RAF scheduling and “wake” hooks for drivers.
- [ ] Move wasm-only DOM glue out of `crates/fret-runner-winit`.
- [ ] Update `crates/fret` feature bundles:
  - [ ] `web` uses `fret-runner-web` (default),
  - [ ] `web-winit` (optional) remains available if needed.

### M3 — Layout feature fork removal (if committed)

- [x] Remove the deprecated layout engine feature fork from `crates/fret-ui` (Cargo features + code paths).
- [x] Update any downstream crates that relied on the removed feature.

### M4 — Router/query consolidation decision

- [ ] Decide: merge `fret-router` into `fret-query` vs rename to clarify layer intent.
- [ ] Apply the decision and update downstream crates (`apps/*` + ecosystem callers).
- [ ] Add/adjust doc pointers if the crate name or surface changes.

### M5 — Follow-up cleanups (optional)

- [ ] Re-audit `crates/fret` default features to ensure they stay “portable by default”.
- [ ] Consider whether `fret-code-editor-view` should remain a standalone crate (only if it reduces churn).

## Validation checklist (run at each milestone)

- [x] `pwsh -NoProfile -File tools/check_layering.ps1`
- [x] `cargo fmt`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo nextest run` (or `cargo test --workspace` if nextest is unavailable)

Notes:

- The workspace is not currently clippy-clean under `-D warnings`; use `nextest` as the primary gate for this workstream until the lint backlog is addressed.
