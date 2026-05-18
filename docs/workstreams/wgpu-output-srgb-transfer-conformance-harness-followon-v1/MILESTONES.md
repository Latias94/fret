# WGPU Output sRGB Transfer Conformance Harness Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Completed

- M0: `output_srgb_transfer_conformance.rs` now uses shared readback/pixel support.
- M1: verification evidence and closeout notes are recorded.

## Current Milestone

- Closed.

## Exit Criteria

- Local helper copies are removed from `output_srgb_transfer_conformance.rs`.
- The test still spells out its `Rgba8Unorm` output setup and sRGB transfer assertions.
- The targeted nextest gate passes.

## Next Milestone

- M1: record gates and close the lane.

## Closure

Closed on 2026-05-18.
