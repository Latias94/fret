---
type: Work Progress
title: Custom effect reference explicit surface owners
timestamp: 2026-07-07T18:03:43Z
tags:
  - fret-examples
  - surface-policy
  - custom-effect
  - reference
status: verified
---

# Summary

Replaced filename-derived custom-effect reference surface owners with explicit owner/reason records
for the V1, V2, V3 native, and V3 web custom-effect demos.

# Truth

- `custom_effect_v1_demo.rs` is now owned as `examples-custom-effect-v1-native` and names
  `EffectParamsV1` in its policy reason.
- `custom_effect_v2_demo.rs` is now owned as `examples-custom-effect-v2-native` and names
  user-image sampling in its policy reason.
- `custom_effect_v3_demo.rs` is now owned as `examples-custom-effect-v3-native` and names
  diagnostic source binding in its policy reason.
- `custom_effect_v3_web_demo.rs` is now owned as `examples-custom-effect-v3-web` and names manual
  web runner/bootstrap ownership in its policy reason.
- The manual chart/plot, streaming import, and custom-effect reference grouped classifications now
  have no filename-derived fallback files.

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

- This is a source-policy precision slice, not a runtime migration.
- Engineering wiki validation passed with existing migration warnings about missing `registry/`,
  large rollups, historical absolute paths, and historical progress/audit artifacts.
