---
type: "Verification Evidence"
title: "Tooltip shadcn pass and pressable semantics invalidation"
description: "Verification Evidence for Tooltip shadcn pass and pressable semantics invalidation."
timestamp: 2026-07-07T22:00:18Z
tags: ["tooltip", "shadcn", "semantics", "verification"]
verified_by: "cargo nextest targeted tooltip and semantics gates"
---

# Verification

Tooltip shadcn pass work verified a framework semantics invalidation fix plus focused Tooltip
behavior and web-golden gates.

# Result

- Pressable declarative a11y relation changes now dirty semantics snapshots, which clears stale
  `aria-describedby`/`described_by` relations when Tooltip closes.
- Tooltip trigger event state is consolidated on the Tooltip root model so Escape, outside press,
  trigger press, hover, and focus close suppression share one lifecycle state.
- Tooltip placement fixture uses a plain pressable trigger for the test harness because shadcn
  Button's outer `AnyElement` id is not necessarily the same as the inner pressable id that Tooltip
  focus/hover policy observes.

# Evidence

- `cargo nextest run -p fret-ui --lib declarative::tests::semantics::declarative_pressable_a11y_relation_changes_refresh_semantics_snapshot`
- `cargo nextest run -p fret-ui-shadcn --lib tooltip::tests`
- `cargo nextest run -p fret-ui-shadcn --test tooltip_hover_and_escape`
- `cargo nextest run -p fret-ui-gallery --test tooltip_docs_surface`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome tooltip::fixtures`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement misc_overlays::fixtures::web_vs_fret_misc_overlays_tooltip_cases_match_web_fixtures`
- `cargo nextest run -p fret-ui-shadcn --test radix_web_primitives_state tooltip`
- `cargo fmt --all --check`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_layering.py`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/report_largest_files.py --top 30 --min-lines 800`

# Follow-up

Run broader workspace gates before release-level changes; this pass intentionally focused the
Tooltip and semantics blast radius.

# Citations

- [Pressable semantics diff](../../../../crates/fret-ui/src/declarative/mount.rs)
- [Pressable relation regression](../../../../crates/fret-ui/src/declarative/tests/semantics.rs)
- [Tooltip recipe](../../../../ecosystem/fret-ui-shadcn/src/tooltip.rs)
- [Tooltip placement fixture](../../../../ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs)
- [Tooltip audit](../../../audits/shadcn-tooltip.md)
