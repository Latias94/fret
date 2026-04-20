# Closeout Audit - 2026-04-20

## Verdict

This lane is closed.

The repo deleted the full `UiCx*` compatibility family instead of carrying it forward as a
deprecated release window.

## Why delete was the correct answer

- canonical first-party teaching was already complete on
  `AppComponentCx<'a>` / `AppRenderCx<'a>` / `AppRenderContext<'a>`
- the remaining `UiCx*` names no longer expressed the real surface semantics
- keeping them would have preserved AI-era naming debt as a public contract on a published crate
- the repo accepted the breaking-change cost now rather than paying long-tail compatibility rent

## Removed surface

- `UiCx<'a>`
- `UiCxActionsExt`
- `UiCxDataExt`
- `UiCxData`
- `UiCxActions`
- `UiCxActionLocal`
- `UiCxLocalsWith`

## Shipped surface after closeout

- app-hosted component/snippet context:
  - `AppComponentCx<'a>`
- concrete closure-local helper context:
  - `AppRenderCx<'a>`
- named helper capability façade:
  - `AppRenderContext<'a>`
- grouped helper namespaces:
  - `AppRenderActionsExt`
  - `AppRenderDataExt`

## Release-facing consequence

`fret 0.1.0` already exists on crates.io, so this deletion is an explicit breaking change on the
published facade.

That is acceptable for this repo state, but the next published `fret` release must carry:

- the SemVer-appropriate version-group bump required by `release-plz` / semver checks
- an explicit breaking-change note that `UiCx*` compatibility aliases were removed

## Closure evidence

Commands for the final slice:

```bash
python3 tools/gate_no_raw_app_context_in_default_teaching_snippets.py
cargo nextest run -p fret --lib
cargo nextest run -p fret --test render_authoring_capability_surface --test raw_state_advanced_surface_docs --test uicx_actions_surface --test uicx_data_surface --test crate_usage_grouped_query_surface --no-fail-fast
python3 tools/check_workstream_catalog.py
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python3 -m json.tool docs/workstreams/uicx-compat-alias-release-retirement-v1/WORKSTREAM.json > /dev/null
git diff --check
```

Key evidence anchors:

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `ecosystem/fret/tests/uicx_actions_surface.rs`
- `ecosystem/fret/tests/uicx_data_surface.rs`
- `docs/crate-usage-guide.md`
- `docs/authoring-golden-path-v2.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `docs/adr/0326-query-vs-mutation-read-vs-submit-default-app-lane-v1.md`
