---
type: Work Progress
title: Streaming import explicit surface owners
timestamp: 2026-07-07T17:59:19Z
tags:
  - fret-examples
  - surface-policy
  - streaming
  - image-import
status: verified
---

# Summary

Replaced the filename-derived streaming import surface owner with explicit owner/reason records for
`streaming_image_demo.rs`, `streaming_i420_demo.rs`, and `streaming_nv12_demo.rs`.

# Truth

- `streaming_image_demo.rs` is now owned as `examples-streaming-image` and its policy reason names
  `ImageUpdateRgba8`.
- `streaming_i420_demo.rs` is now owned as `examples-streaming-i420` and its policy reason names
  `ImageUpdateI420`.
- `streaming_nv12_demo.rs` is now owned as `examples-streaming-nv12` and its policy reason names
  `ImageUpdateNv12`.
- No runtime source changed; this slice only makes the advanced/manual surface classification more
  auditable.

# Artifacts

- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy.SurfacePolicyTests.test_fret_examples_public_scan_roots_stay_precise`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Notes

- The demos still intentionally own low-level image registration/update effects and manual
  `FnDriver` hooks; this is not a facade migration.
- Engineering wiki validation passed with existing migration warnings about missing `registry/`,
  large rollups, historical absolute paths, and historical progress/audit artifacts.
