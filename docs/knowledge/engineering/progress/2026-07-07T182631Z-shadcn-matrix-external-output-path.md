---
type: Work Progress
title: Shadcn matrix generator accepts external output paths
timestamp: 2026-07-07T18:26:31Z
tags:
  - shadcn
  - parity-matrix
  - tooling
status: verified
---

# Summary

Fixed `tools/parity-discovery/shadcn_component_harness_matrix.py` so `--output-json` and
`--output-md` may point outside the repository.

# Truth

- The generator already wrote repo-external outputs successfully, but its final status print called
  `relative_to(ROOT)` unconditionally and crashed for `/tmp/...` paths.
- `_display_path(...)` now prints repo-local outputs as POSIX repo-relative paths and repo-external
  outputs as POSIX absolute paths.
- The generated matrix content was not synchronized in this slice.

# Artifacts

- `tools/parity-discovery/shadcn_component_harness_matrix.py`
- `tools/test_shadcn_component_harness_matrix.py`

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_shadcn_component_harness_matrix`
- `python3 tools/parity-discovery/shadcn_component_harness_matrix.py --output-json /tmp/.../matrix.json --output-md /tmp/.../MATRIX.md`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Notes

- The repo-local generated matrix still differs from a fresh generation by date and path-separator
  normalization only; that sync was left as a separate documentation/artifact decision.
- Engineering wiki validation passed with existing migration warnings about missing `registry/`,
  large rollups, historical absolute paths, and historical progress/audit artifacts.
