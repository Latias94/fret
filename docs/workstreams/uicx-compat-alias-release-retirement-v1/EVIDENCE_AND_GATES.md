# UiCx Compat Alias Release Retirement v1 — Evidence and Gates

Status: Closed

## Smallest current repro

Use this sequence to reopen the shipped deletion evidence:

```bash
rg -n "pub type UiCx<'a>|UiCxActionsExt|UiCxDataExt|UiCxActions|UiCxData|UiCxActionLocal|UiCxLocalsWith" ecosystem/fret/src/lib.rs ecosystem/fret/src/view.rs
rg -n "semver_check = true|name = \"fret\"|publish = true" release-plz.toml
python3 tools/gate_no_raw_app_context_in_default_teaching_snippets.py
```

What this proves:

- the old compatibility alias family no longer exists on the live `fret` surface,
- the `fret` facade remains part of a semver-checked published release graph,
- and the canonical first-party default teaching surface still rejects `UiCx<'_>`.

## Gate set

### Source-policy gates

```bash
python3 tools/gate_no_raw_app_context_in_default_teaching_snippets.py
cargo nextest run -p fret --test render_authoring_capability_surface --test raw_state_advanced_surface_docs --test uicx_actions_surface --test uicx_data_surface --test crate_usage_grouped_query_surface --no-fail-fast
cargo nextest run -p fret --lib
```

### Lane hygiene

```bash
python3 tools/check_workstream_catalog.py
python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
python3 -m json.tool docs/workstreams/uicx-compat-alias-release-retirement-v1/WORKSTREAM.json > /dev/null
git diff --check
```

## Current evidence after deletion

- `ecosystem/fret/src/lib.rs` no longer exports `UiCx<'a>`.
- `fret::app` no longer reexports `UiCxActionsExt` or `UiCxDataExt`.
- `fret::advanced::view` no longer reexports `UiCxDataExt`.
- `ecosystem/fret/src/view.rs` no longer defines the hidden `UiCx*` carrier aliases.
- Default teaching gates and first-party docs remain on canonical names only.
- `release-plz.toml` still treats the workspace as `semver_check = true`, so the next published
  `fret` release must carry the breaking-change note for this deletion.

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
