# IMUI Plot Adapter Proof v1 - Evidence & Gates

Status: Closed
Last updated: 2026-05-25

## Evidence Anchors

- Workstream:
  - `docs/workstreams/imui-plot-adapter-proof-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-plot-adapter-proof-v1/DESIGN.md`
  - `docs/workstreams/imui-plot-adapter-proof-v1/TODO.md`
  - `docs/workstreams/imui-plot-adapter-proof-v1/MILESTONES.md`
  - `docs/workstreams/imui-plot-adapter-proof-v1/EVIDENCE_AND_GATES.md`
- Adapter implementation:
  - `ecosystem/fret-plot/Cargo.toml`
  - `ecosystem/fret-plot/src/lib.rs`
  - `ecosystem/fret-plot/src/imui.rs`
- Reference evidence:
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`

## Repro

```powershell
cargo check -p fret-plot --features imui
```

## Focused Gates

```powershell
cargo fmt --check -p fret-plot
cargo check -p fret-plot
cargo check -p fret-plot --features imui
cargo nextest run -p fret-plot imui_adapter_stays_opt_in_and_declarative_only --no-fail-fast
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-plot-adapter-proof-v1/WORKSTREAM.json
git diff --check
```

## 2026-05-25 Slice Results

- `cargo fmt --check -p fret-plot` passed after targeted `cargo fmt -p fret-plot`.
- `cargo check -p fret-plot` passed and proved the default feature set still compiles.
- `cargo check -p fret-plot --features imui` passed and proved the optional `UiWriter` adapter
  compiles over the existing declarative plot panels.
- `cargo nextest run -p fret-plot imui_adapter_stays_opt_in_and_declarative_only --no-fail-fast`
  passed with 1 focused test and proved the adapter stays opt-in and declarative-only.
- `python tools/gate_imui_workstream_source.py` passed and now freezes the plot adapter boundary:
  `fret-imui` and `fret-ui-kit` must not depend on `fret-plot`, and the retained plot bridge stays
  deleted.
- `python tools/check_workstream_catalog.py` passed and validated 439 dedicated directories plus 47
  standalone markdown files.
- `python -m json.tool docs/workstreams/imui-plot-adapter-proof-v1/WORKSTREAM.json` passed.
- `git diff --check` passed with Git CRLF/LF working-copy warnings for `Cargo.lock` and
  `apps/fret-examples/src/lib.rs`, but no whitespace errors.

Notes:

- The compile gates emitted existing `crates/fret-ui` warnings for `unexpected cfg:
  unstable-retained-bridge` and `current_effective_opacity`.
- The compile and test gates emitted existing `fret-plot` dead-code warnings for
  `apply_axis_locks` and `all_visible_axes_zoom_locked`.
- `Cargo.lock` now records `fret-authoring` in the `fret-plot` dependency list because
  `fret-plot/imui` uses `dep:fret-authoring`.
