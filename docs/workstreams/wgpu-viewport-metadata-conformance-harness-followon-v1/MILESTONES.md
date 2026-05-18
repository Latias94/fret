# WGPU Viewport Metadata Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Completed

- M0: `viewport_surface_metadata_conformance.rs` now uses shared final render/readback and pixel support.
- M1: verification evidence and closeout notes are recorded.

## Current Milestone

- Closed.

## Exit Criteria

- Local helper copies are removed from `viewport_surface_metadata_conformance.rs`.
- Source texture writers and `RenderTargetMetadata` assertions remain local.
- The targeted nextest gate passes.

## Next Milestone

- None. Start separate follow-ons for backend/platform-specific helper cleanup.

## Closure

Closed on 2026-05-18.
