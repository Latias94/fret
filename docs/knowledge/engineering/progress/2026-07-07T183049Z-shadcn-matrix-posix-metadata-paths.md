---
type: Work Progress
title: Shadcn matrix metadata uses POSIX paths
timestamp: 2026-07-07T18:30:49Z
tags:
  - shadcn
  - parity-matrix
  - tooling
status: verified
---

# Summary

Made `tools/parity-discovery/shadcn_component_harness_matrix.py` write POSIX-style paths in generated
matrix metadata regardless of host platform.

# Truth

- `source_docs.progress_doc`, `manifest`, `suite_report`, and `extra_reports` now use the shared
  `_display_path(...)` helper.
- Extra report summaries also use `_display_path(...)` for their `output` field.
- This prevents Windows-generated matrix artifacts from recording backslash-separated repo paths.
- The checked-in generated matrix artifact was not synchronized in this slice.

# Artifacts

- `tools/parity-discovery/shadcn_component_harness_matrix.py`
- `tools/test_shadcn_component_harness_matrix.py`

# Verification

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_shadcn_component_harness_matrix`
- `python3 tools/parity-discovery/shadcn_component_harness_matrix.py --output-json /tmp/.../matrix.json --output-md /tmp/.../MATRIX.md`
- Temporary generated JSON check asserted all `source_docs` paths contain no backslashes.
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `python3 $HOME/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

# Notes

- This builds on the repo-external output path fix and keeps the generator behavior deterministic
  across macOS/Linux/Windows path separators.
- Engineering wiki validation passed with existing migration warnings about missing `registry/`,
  large rollups, historical absolute paths, and historical progress/audit artifacts.
