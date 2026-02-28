# Hotpatch Devloop Alignment v1 — Milestones

Current status (2026-02-15):

- M0: Achieved (baseline UX + observability shipped)
- M1: In progress (fallback ladder mostly in place; crash-driven restart UX still open)
- M2/M3/M4: Achieved (theme/assets/literals no-compile channels shipped; fonts apply included)

## M0 — Baseline UX + observability (short)

Deliverables:

- Workstream docs exist and are linked:
  - `docs/workstreams/hotpatch-devloop-alignment-v1.md`
  - `docs/workstreams/hotpatch-devloop-alignment-v1-todo.md`
  - `docs/workstreams/hotpatch-devloop-alignment-v1-milestones.md`
- `fretboard dev native --hotpatch` is the recommended path and prints a stable startup summary.
- Logs are easy to find and referenced in the summary.

Exit criteria:

- A user can answer “am I actually hotpatching Rust code?” from the first 10 lines of output.

Evidence anchors:

- `apps/fretboard/src/dev.rs` (startup Hotpatch Summary)
- `apps/fretboard/src/hotpatch.rs` (`fretboard hotpatch status`)
- `.fret/hotpatch_runner.log`, `.fret/hotpatch_bootstrap.log` (stable log locations)

## M1 — Predictable fallback ladder (time-boxed)

Deliverables:

- A single documented fallback ladder is implemented:
  - patch applied → runner reload boundary
  - known-unsafe view-level hotpatch → explicit warning + boundary-only behavior
  - repeated crash → restart guidance (and optionally a supervised restart)

Exit criteria:

- Windows “patched view crash” does not present as a silent failure; the user gets a clear next action.

Evidence anchors:

- `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md` (Windows known issue)
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (view call strategy; Windows safety default)

## M2 — Theme reload (no compile) (time-boxed)

Deliverables:

- Theme reload contract + a minimal implementation for one demo.
- A simple repro gate exists (manual instructions or a `fretboard diag` script).

Exit criteria:

- Change a theme token file and see UI update without rebuilding.

Evidence anchors:

- `ecosystem/fret-bootstrap/src/dev_reload.rs` (theme watcher + apply)

## M3 — Asset reload (no compile) (time-boxed)

Deliverables:

- Asset invalidation contract + a minimal implementation for one asset type (svg/png/fonts).

Exit criteria:

- Replace an asset file and see UI update without rebuilding.

Evidence anchors:

- `ecosystem/fret-ui-assets/src/reload.rs` (`UiAssetsReloadEpoch`)
- `ecosystem/fret-ui-assets/src/image_source.rs` (epoch in cache keys)
- `ecosystem/fret-ui-assets/src/svg_file.rs` (epoch-gated SVG file bytes)
- `ecosystem/fret-bootstrap/src/dev_reload.rs` (trigger file watcher)

## M4 — Hot literals (no compile) (time-boxed)

Deliverables:

- Hot-literals file format + a minimal implementation for one demo.

Exit criteria:

- Update a label/tooltip via a data file and see UI update without rebuilding.

Evidence anchors:

- `ecosystem/fret-bootstrap/src/hot_literals.rs`
- `ecosystem/fret-bootstrap/src/dev_reload.rs`
