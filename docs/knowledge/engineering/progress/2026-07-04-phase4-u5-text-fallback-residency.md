---
type: Work Progress
title: Phase 4 U5 text fallback residency signature
tags: fret,phase4,renderer,text,residency,fallback-fonts
timestamp: 2026-07-04T16:11:33Z
related_plan: ../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

This Phase 4 U5 sub-slice added characterization coverage for fallback-font identity in
cluster-aware text residency signatures.

# Design Finding

Phase 3 already moved WGPU text residency from glyph-only to cluster-aware metadata. The remaining
U5 risk is proof: future deletion of full-blob text oracles needs tests that show cluster/run facts
actually participate in residency keys.

The new characterization compares the same CJK character in two bundled-font environments:

- bootstrap fonts only, where the character records missing/tofu glyphs;
- bootstrap + CJK fallback fonts, where the same character resolves to the bundled CJK fallback.

The test asserts that cluster font fingerprint, residency cluster fingerprint, and residency glyph
fingerprint all change. This keeps fallback font identity visible before chunk-native text support
is widened.

# Verification

Passed:

- `cargo nextest run -p fret-render-wgpu cluster_residency_signature_changes_with_fallback_font_identity --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu text_shape_records_cluster_metadata_for_inter_ligature cluster_residency_pins_complete_multi_glyph_cluster visible_text_residency_pins_complete_combining_cluster_under_narrow_scissor cluster_residency_signature_changes_with_fallback_font_identity --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu text --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `git diff --check`

# Next Action

U5 still has room for a deterministic RTL proof if the repo gains a bundled RTL fixture font. Until
then, do not delete full-blob text oracles solely on the strength of Latin/CJK/emoji coverage.

# Citations

- [Phase 4 topology epoch plan](../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md)
- [Phase 3 U6 text cluster residency](2026-07-03-phase3-u6-text-cluster-residency.md)
