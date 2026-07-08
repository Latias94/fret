---
type: Work Progress
title: Typography helpers moved to explicit shadcn facade module
timestamp: 2026-07-08T03:54:36Z
tags:
  - fret-ui-gallery
  - fret-ui-shadcn
  - public-surface
  - typography
status: verified
---

# Summary

Closed the largest remaining UI Gallery raw-helper teaching surface by exposing Fret's
shadcn-style typography helpers through `fret_ui_shadcn::facade::typography::*` and migrating
first-party Gallery snippets/tests from `shadcn::raw::typography::*` to `shadcn::typography::*`.

# Decisions

- Kept typography under an explicit facade submodule instead of promoting helpers to the stable
  top-level `shadcn::*` component namespace, because typography is a Fret-owned docs/prose helper
  family rather than an upstream shadcn/ui default component taxonomy entry.
- Kept `apps/fret-examples` raw typography allowlist untouched. That examples lane still has a
  separate app/text-facade migration story and currently conflicts with direct `shadcn::typography`
  use.
- Fixed two stale `ui_authoring_surface_default_app` assertions found while running the full gate:
  one still required `IntoUiElementInExt as _` after the snippet had removed it, and the tooltip
  docs assertion had not caught up with the current `asChild` wording.

# Verification

- `cargo nextest run -p fret-ui-shadcn --lib typography --no-fail-fast`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies --no-fail-fast`
- `cargo nextest run -p fret-ui-gallery --test typography_docs_surface --no-fail-fast`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --no-fail-fast`
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast`
- `cargo nextest run -p fret-ui-gallery --test scroll_area_docs_surface --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`

# Known Existing Gate Drift

- `PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 tools/examples_source_tree_policy/gate.py`
  still fails on the existing examples source-tree policy inventory before this slice's scope.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/gate_imui_workstream_source.py` still fails because it
  references the missing historical path
  `ecosystem/fret-ui-editor/src/composites/property_row/slot.rs`.

# Next Action

The remaining raw UI Gallery helper family is low-level icon glue (`shadcn::raw::icon::*`). Audit it
separately because icon helpers are closer to declarative/icon-pack plumbing than prose helpers.
