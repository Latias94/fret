---
type: "Work Progress"
title: "Markdown demo uses app text facade"
description: "Work Progress for Markdown demo uses app text facade."
timestamp: 2026-07-07T07:43:57Z
tags: ["ui-surface", "examples", "markdown", "ce-work"]
related_plan: "docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md"
git_branch: "refactor/markdown-text-facade"
---

# Summary

Moved `markdown_demo.rs` fixed chrome/status/image-placeholder text helpers onto the app-facing
`fret::app::text` and `AppElement` surface.

# Details

Changed files:

- `ecosystem/fret/src/lib.rs`
- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-examples/tests/markdown_demo_surface.rs`
- `tools/check_surface_policy.py`
- `tools/test_check_surface_policy.py`

Decision:

- Add `fret::app::text::paragraph_break_words_with_foreground` for app demos that need wrapped
  placeholder/prose text with inherited foreground.
- Keep markdown-specific image/SVG props and raw pressable placeholder behavior explicit; those are
  still advanced markdown render-hook seams.
- Remove `AnyElement` and `ElementContext` from the markdown demo allowed raw seam list.

# Verification

Passed before commit:

- `cargo fmt --all --check`
- `cargo nextest run -p fret app_and_style_modules_expose_explicit_secondary_app_nouns --no-fail-fast`
- `cargo nextest run -p fret-examples --test markdown_demo_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.test_check_surface_policy`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- raw text seam scan for `markdown_demo.rs` found no `AnyElement`, `fret_ui::ElementContext`,
  `fret_ui::UiHost`, `decl_text`, or `fret_ui::scroll::ScrollHandle` hits.

# Next Action

Continue reducing markdown only where a reusable app/render facade exists. Do not hide SVG/image
props or URL-opening effects unless a deliberate markdown asset/render policy facade is designed.

# Citations

- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-examples/tests/markdown_demo_surface.rs`
- `tools/check_surface_policy.py`
