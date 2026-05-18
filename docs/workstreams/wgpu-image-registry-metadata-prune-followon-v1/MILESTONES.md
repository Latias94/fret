# WGPU Image Registry Metadata Prune Follow-on v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Registry State Matches Runtime Readers

Exit criteria:

- `ImageEntry.color_space` is removed.
- `ImageEntry.format` and `ImageEntry.alpha_mode` remain because render encoding reads them.
- `ImageDescriptor.color_space` validation remains unchanged.

Status: Complete on 2026-05-18.

## M1 - Closeout Evidence

Exit criteria:

- `cargo check -p fret-render-wgpu --locked --tests -j 1` passes.
- Workstream catalog and JSON checks pass.
- Diff whitespace check passes.

Status: Complete on 2026-05-18.
