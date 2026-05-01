# M3 Pane Proof Closure - 2026-04-21

Status: closed on 2026-04-21

## Decision

1. Keep `apps/fret-examples/src/workspace_shell_demo.rs` as the pane-first M3 proof surface.
2. Close M3 with a shell-mounted pane proof inside the existing workspace shell demo.
3. Keep `ecosystem/fret-ui-kit/src/imui/child_region.rs` unchanged for M3.
4. No narrower pane-only diagnostics path is required at M3 because the existing workspace shell diag floor remains sufficient.

## What changed

- `apps/fret-examples/src/workspace_shell_demo.rs` now mounts a shell-owned immediate pane proof
  inside pane content instead of only showing a flat placeholder string.
- The pane proof now composes nested `child_region` surfaces for:
  - toolbar,
  - tabs,
  - inspector,
  - and status
  inside a shell-mounted pane host.
- `apps/fret-examples/tests/workspace_shell_pane_proof_surface.rs` now locks the pane proof source
  markers in the example surface.
- `tools/gate_imui_workstream_source.py` now locks the M3 pane proof markers with lane-local
  source-policy assertions.

## Why this closes M3

- The repo now has one first-party pane composition proof that exercises the current
  `child_region` seam without inventing a narrower child-region-only demo.
- The proof mounts real editor/workspace chrome:
  the workspace shell still owns frame slots and tab strips while the pane content now carries a
  nested immediate composition instead of an isolated helper snippet.
- The helper decision is now explicit for this milestone:
  the current `child_region` surface is sufficient for the M3 proof, so helper widening is not
  justified yet.

## Evidence

- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/tests/workspace_shell_pane_proof_surface.rs`
- `tools/gate_imui_workstream_source.py`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`

## Gates

- `python tools/gate_imui_workstream_source.py`
- `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --test workspace_shell_pane_proof_surface --test editor_notes_editor_rail_surface --no-fail-fast`
- `cargo run -p fretboard-dev -- diag suite diag-hardening-smoke-workspace --launch -- cargo run -p fret-demo --bin workspace_shell_demo --release`
