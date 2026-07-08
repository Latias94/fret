---
type: Work Progress
title: Extras explicit facade module
timestamp: 2026-07-08T03:27:04Z
tags:
  - shadcn
  - public-surface
  - ui-gallery
  - verification
status: verified
---

# Summary

Moved Fret-specific shadcn extras from `shadcn::raw::extras::*` to an explicit
`shadcn::extras::*` facade submodule. Extras remain outside the stable top-level shadcn taxonomy,
but first-party gallery authoring no longer needs to reopen the raw namespace for them.

# Changed Files

- `ecosystem/fret-ui-shadcn/src/lib.rs`: added `facade::extras` and re-exported the existing extras
  public types inside that explicit submodule.
- `ecosystem/fret-ui-shadcn/src/extras/mod.rs`: updated the module contract text from raw escape
  hatch to explicit facade submodule.
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`: added a gate that keeps extras under
  `facade::extras` and prevents top-level facade promotion.
- `apps/fret-ui-gallery/src/ui/snippets/shadcn_extras/*.rs`: migrated all extras examples to
  `shadcn::extras::*`.
- `apps/fret-ui-gallery/tests/ui_authoring_surface_import_policies.rs`: removed raw extras from the
  raw allowlist and required the explicit extras facade module.

# Verification

- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app shadcn_extras`
- `cargo nextest run -p fret-ui-shadcn --lib extras`
- `cargo fmt --all`
- `cargo fmt --all --check`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`
- `python3 tools/check_layering.py`
- `python3 tools/report_largest_files.py --top 30 --min-lines 800`
- `git diff --check`

# Next Candidates

The gallery raw allowlist is now down to typography prose helpers and low-level icon helpers.
Typography has many callsites and likely needs an explicit `facade::typography` module, while icon
helpers should be reviewed separately because they may overlap with preferred icon-bearing widget
APIs.
