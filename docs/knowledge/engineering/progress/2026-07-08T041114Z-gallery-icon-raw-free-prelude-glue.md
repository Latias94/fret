---
type: Work Progress
title: UI Gallery icon helpers moved off shadcn raw
timestamp: 2026-07-08T04:11:14Z
tags:
  - fret-ui-gallery
  - fret-ui-shadcn
  - public-surface
  - icon
status: verified
---

# Summary

Closed the remaining UI Gallery raw helper lane by moving first-party `shadcn::raw::icon::*`
callsites to the existing prelude-imported `icon::*` glue from `fret_ui_kit::declarative::icon`.

# Decision

Did not add a `facade::icon` module. Icon rendering is low-level declarative glue shared with
icon-pack plumbing, not a shadcn component family. First-party snippets already commonly use
`icon::icon(...)` and `icon::icon_with(...)` through `fret_ui_shadcn::prelude::*`; files that import
only `facade as shadcn` now import `fret_ui_kit::declarative::icon` explicitly.

# Verification

- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies --no-fail-fast`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --no-fail-fast`
- `cargo nextest run -p fret-ui-shadcn --lib first_party_code_avoids_root_authoring_glue_lane --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all --check`
- `git diff --check`

# Residual Risk

Historical workstream/audit documents still mention low-level `shadcn::raw::icon::*`; those were
left as historical records. Current UI Gallery source and import-policy gates now reject raw icon
usage.
