# ImUi Collection Second Proof Surface v1 - M2 Shell-Mounted Collection Surface Slice

Date: 2026-04-23
Status: landed

## What changed

`editor_notes_demo.rs` now carries the first materially different shell-mounted collection proof
surface for this lane.

The left rail still uses the existing `WorkspaceFrame` slot and the existing app-owned selection
actions, but the old single-purpose button group is now an explicit `Scene collection` surface with:

1. a stable collection root test id,
2. a stable collection summary test id,
3. a stable collection list test id,
4. app-owned row labels that include title, role, and active/available state,
5. and the existing `SelectMaterial`, `SelectLight`, and `SelectCamera` commands as the owner path.

## Why this is enough for M2

This is materially different from the first collection proof because:

- it is mounted in a workspace-shell rail rather than a standalone asset-browser grid,
- it composes with reusable editor inspector content instead of owning the whole editor proof,
- it keeps collection state aligned with text-editing/inspector state,
- and it proves collection pressure without creating a dedicated asset-grid/file-browser demo.

## Owner split

Owned here:

- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `apps/fret-examples/src/lib.rs` source-policy coverage

Not owned here:

- no `fret-imui` public facade widening,
- no `fret-ui-kit::imui` collection helper widening,
- no `crates/fret-ui` runtime contract change,
- no new dedicated asset-grid/file-browser demo.

## Gate floor

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_closes_the_p1_collection_second_proof_surface_follow_on --no-fail-fast`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test workspace_shell_pane_proof_surface --test workspace_shell_editor_rail_surface --no-fail-fast`

These gates now prove that the second proof-surface lane has a real shell-mounted collection
surface in `editor_notes_demo.rs` while `workspace_shell_demo.rs` remains supporting evidence.
