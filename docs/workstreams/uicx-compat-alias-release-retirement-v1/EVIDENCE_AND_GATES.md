# UiCx Compat Alias Release Retirement v1 — Evidence and Gates

Status: Active

## Smallest current repro

Use this sequence before changing alias code:

```bash
rg -n "pub type UiCx<'a>|UiCxActionsExt|UiCxDataExt|UiCxActions|UiCxData|UiCxActionLocal|UiCxLocalsWith" ecosystem/fret/src/lib.rs ecosystem/fret/src/view.rs
rg -n "semver_check = true|name = \"fret\"|publish = true" release-plz.toml
python3 tools/gate_no_raw_app_context_in_default_teaching_snippets.py
```

What this proves:

- the compatibility alias family still exists and where,
- the `fret` facade remains part of a semver-checked published release graph,
- and the canonical first-party default teaching surface already rejects `UiCx<'_>`.

## Gate set

### Source-policy gates

```bash
python3 tools/gate_no_raw_app_context_in_default_teaching_snippets.py
cargo nextest run -p fret --test render_authoring_capability_surface --test raw_state_advanced_surface_docs --test uicx_actions_surface --test uicx_data_surface --test crate_usage_grouped_query_surface --no-fail-fast
```

### Lane hygiene

```bash
python3 tools/check_workstream_catalog.py
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python3 -m json.tool docs/workstreams/uicx-compat-alias-release-retirement-v1/WORKSTREAM.json > /dev/null
git diff --check
```

## Current evidence at lane open

- `UiCx<'a>` remains a deprecated root compatibility alias to `AppComponentCx<'a>`.
- `fret::app` still explicitly reexports `UiCxActionsExt` and `UiCxDataExt` under
  `#[allow(deprecated)]`.
- `fret::advanced::view` still explicitly reexports `UiCxDataExt`.
- `ecosystem/fret/src/view.rs` still defines the hidden deprecated carrier aliases:
  `UiCxDataExt`, `UiCxData`, `UiCxActionsExt`, `UiCxActions`, `UiCxActionLocal`,
  `UiCxLocalsWith`.
- Default teaching gates and first-party docs already require or teach canonical names instead of
  `UiCx`.
- `release-plz.toml` treats the workspace as `semver_check = true` and includes published `fret`
  package release entries, so alias deletion needs an explicit release-facing justification.

## Evidence anchors

- `docs/workstreams/uicx-compat-alias-release-retirement-v1/DESIGN.md`
- `docs/workstreams/uicx-compat-alias-release-retirement-v1/M0_BASELINE_AUDIT_2026-04-19.md`
- `docs/workstreams/uicx-compat-alias-release-retirement-v1/TODO.md`
- `docs/workstreams/uicx-compat-alias-release-retirement-v1/MILESTONES.md`
- `docs/workstreams/uicx-compat-alias-release-retirement-v1/WORKSTREAM.json`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-04-19.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/UICX_COMPAT_ALIAS_DEPRECATION_AUDIT_2026-04-19.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `docs/crate-usage-guide.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/first-hour.md`
- `docs/shadcn-declarative-progress.md`
- `ecosystem/fret/Cargo.toml`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/tests/render_authoring_capability_surface.rs`
- `ecosystem/fret/tests/raw_state_advanced_surface_docs.rs`
- `ecosystem/fret/tests/uicx_actions_surface.rs`
- `ecosystem/fret/tests/uicx_data_surface.rs`
- `ecosystem/fret/tests/crate_usage_grouped_query_surface.rs`
- `tools/gate_no_raw_app_context_in_default_teaching_snippets.py`
- `tools/pre_release.py`
- `release-plz.toml`
