# Renderer Debug Dump Gate v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Shared Mechanism

Exit criteria:

- Render-plan and text dump owners no longer duplicate frame gate parsing.
- One-shot state remains per dump owner.
- Dump-specific env var names, directories, filenames, and schemas remain local.

Status: Done.

## M1 - Verified Closeout

Exit criteria:

- `fret-render-wgpu` test build passes.
- Shared gate behavior is covered by targeted unit tests.
- Workstream catalog and JSON metadata are valid.

Status: Done.
