# Workspace Crate Boundaries Audit v1 — TODO Tracker

Status: Active (fearless refactor friendly; pre-1.0)

Related plan:

- `docs/workstreams/workspace-crate-boundaries-v1.md`

## Milestones

### M0 — Governance and tracking

- [x] Add this workstream to `docs/workstreams/README.md`.
- [x] Add a short entry to `docs/todo-tracker.md` pointing to this workstream (optional).
- [x] Confirm the intended wasm “golden path” bundles in `crates/fret` (`web` vs `web-winit`).

### M1 — Render split (`fret-render-core` + `fret-render-wgpu`)

- [x] Create `crates/fret-render-core` (portable render contracts/types).
- [x] Create `crates/fret-render-wgpu` (wgpu backend implementation).
- [x] Keep `crates/fret-render` as a compatibility facade (re-exports / feature glue).
- [x] Update `crates/fret-launch` to depend on `fret-render-wgpu` for native.
- [x] Ensure `tools/check_layering.ps1` still passes.

### M2 — Web runner becomes real (`fret-runner-web`)

- [x] Implement DOM/canvas event translation in `crates/fret-runner-web`.
- [x] Provide RAF scheduling and “wake” hooks for drivers.
- [x] Move wasm-only DOM glue out of `crates/fret-runner-winit`.
- [x] Update `crates/fret` feature bundles:
  - [x] `web` uses `fret-runner-web` (default),
  - [x] `web-winit` (optional) remains available if needed.

### M3 — Layout feature fork removal (if committed)

- [x] Remove the deprecated layout engine feature fork from `crates/fret-ui` (Cargo features + code paths).
- [x] Update any downstream crates that relied on the removed feature.

### M4 — Router/query consolidation decision

- [x] Decide: keep `fret-router` and `fret-query` separate (no merge/rename).
- [x] Tighten boundaries:
  - [x] Remove unused/stale `fret-router` feature flags (keep web + query integration only).
  - [x] Make `fret-query` portable by default (no `ui` in `default` features); callers opt into `fret-query/ui`.
- [x] Update docs to reflect the new feature defaults and router feature surface.

### M5 — Follow-up cleanups (optional)

- [x] Re-audit `crates/fret` default features to ensure they stay “portable by default”.
- [x] Consider whether `fret-code-editor-view` should remain a standalone crate (keep: buffer/view are reusable seams).
- [x] Add a short “merge vs split” candidate list to the crate survey notes and revisit quarterly.

### M6 — Reduce accidental micro-crates (optional)

- [x] Merge `ecosystem/fret-ui-primitives` into `ecosystem/fret-ui-kit` (it was only consumed via `fret-ui-kit` shims).

## Validation checklist (run at each milestone)

- [x] `pwsh -NoProfile -File tools/check_layering.ps1`
- [x] `cargo fmt`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo nextest run` (or `cargo test --workspace` if nextest is unavailable)

Notes:

- The workspace is not currently clippy-clean under `-D warnings`; use `nextest` as the primary gate for this workstream until the lint backlog is addressed.
