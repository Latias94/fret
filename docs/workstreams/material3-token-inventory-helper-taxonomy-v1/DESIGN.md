# Material3 Token Inventory Helper Taxonomy v1 - Design

Status: closed
Date: 2026-05-31

## Intent

The Material3 token inventory originally tracked `shape`, `typography`, and family helpers as
"shared token helpers". After the FAB default-governance lane, that term became inaccurate:
`fab_common` is helper policy, but it is not shared across multiple component modules.

This lane fixes the taxonomy so future inventory reports can distinguish component token modules
from token helper policy modules without implying every helper is cross-component shared.

## Source And Boundary Truth

- This is tooling/docs taxonomy only.
- Runtime Material3 component behavior and token values stay unchanged.
- Existing JSON readers may still expect `shared_token_helper_modules`, so this lane keeps
  backward-compatible aliases while introducing the clearer `token_helper_modules` fields.

## Scope

In scope:

- Rename the generator implementation terminology from shared helper to token helper.
- Add new report fields for `token_helper_modules` and `token_helper_module_count`.
- Keep legacy `shared_token_helper_modules` fields as aliases.
- Regenerate a focused inventory artifact for this lane.
- Update the FAB closeout residual note that pointed to this follow-on.

Out of scope:

- Changing token scanner heuristics.
- Reclassifying historical artifacts.
- Changing component token modules or runtime behavior.

## Refactor Brief

Intent: remove misleading audit terminology before more helper modules are added.

Scope: `tools/parity-discovery/material3_token_inventory.py`, this workstream evidence, and the
latest FAB closeout note.

Deletion plan: delete misleading implementation names such as `SHARED_TOKEN_HELPER_MODULES` and
`scan_shared_token_helper_modules`.

Boundary plan: keep this in tooling/docs; no crate or component behavior changes.

Testing plan: regenerate inventory, validate JSON, run workstream catalog, layering, and diff
checks.

Risk plan: external readers may use old JSON fields. Preserve legacy aliases in the report.

Scale plan: small bounded fearless-refactor workstream with one commit.
