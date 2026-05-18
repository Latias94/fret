# WGPU Image Registry Metadata Prune Follow-on v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Registry Metadata Prune

- [x] WIRP-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src/images.rs]
  Goal: Remove retained `ImageEntry.color_space` state while preserving `ImageDescriptor.color_space`
  validation in `register` and `update`.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`.
  Evidence: `ImageEntry` now stores only view, size, format, and alpha mode; the format/color-space
  debug assertions remain.
  Status: Done on 2026-05-18.

## M1 - Gates And Closeout

- [x] WIRP-020 [owner=codex] [deps=WIRP-010] [scope=docs/workstreams/wgpu-image-registry-metadata-prune-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on.
  Validation: `python tools/check_workstream_catalog.py`; `python -m json.tool docs/workstreams/wgpu-image-registry-metadata-prune-followon-v1/WORKSTREAM.json`; `git diff --check`.
  Evidence: closeout audit names the pruned field and compile/catalog gates.
  Status: Done on 2026-05-18.
