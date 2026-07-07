---
type: "Work Progress"
title: "Input Group docs surface gate"
description: "Work Progress for Input Group docs surface gate."
timestamp: 2026-07-07T22:57:58Z
tags: ["fret", "shadcn", "input-group", "ui-gallery", "public-surface", "documentation-ordering"]
---

# Summary

- Added a dedicated UI Gallery Input Group docs-surface gate for the public-surface cleanup lane.
- Scope is docs/teaching-surface alignment only: no recipe or runtime implementation changes.

# Details

- `apps/fret-ui-gallery/tests/input_group_docs_surface.rs` locks the page order through
  `API Reference` before Tooltip / Label Association / Button Group / Notes follow-ups.
- The page now states that `Parts Usage` is Fret's typed translation of upstream `Composition`,
  while `Usage` remains the compact `InputGroup::new(model)` shorthand lane.
- The gate keeps copyable snippets free of `shadcn::raw::*`, `advanced::*`, `InputGroup::compose`,
  root `InputGroup::children(...)`, and `build_parts(...)` on the default docs surface.
- The same gate anchors the docs-smoke, text non-overlap, label focus, addon tab-order, dropdown
  relation/action-state, and RTL addon-order diagnostic scripts.
- Updated `docs/audits/shadcn-input-group.md` and `docs/shadcn-declarative-progress.md` to cite the
  new gate.

# Verification

- Passed: `cargo nextest run -p fret-ui-gallery --test input_group_docs_surface`.
- Passed:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app input_group --no-fail-fast`.
- Passed: `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`.
- Passed: `python3 tools/check_layering.py`.
- Passed: `python3 tools/report_largest_files.py --top 30 --min-lines 800` (report only; no failure).
- Passed: `cargo fmt --all --check`.
- Passed: JSON parse for the cited Input Group diagnostic scripts and suite manifests.
- Passed: `git diff --check`.
- Passed with existing warnings only: engineering wiki memory validation for
  `docs/knowledge/engineering`.

# Next Action

- Commit and push this Input Group docs-surface gate slice.
- Remaining similar docs-order candidate from the tracker scan: `radio_group`.

# Citations

- `apps/fret-ui-gallery/tests/input_group_docs_surface.rs`
- `apps/fret-ui-gallery/src/ui/pages/input_group.rs`
- `apps/fret-ui-gallery/src/ui/snippets/input_group/`
- `repo-ref/ui/apps/v4/content/docs/components/base/input-group.mdx`
- `docs/audits/shadcn-input-group.md`
- `docs/shadcn-declarative-progress.md`
