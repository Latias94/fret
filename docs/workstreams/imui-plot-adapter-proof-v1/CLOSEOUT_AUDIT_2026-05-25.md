# IMUI Plot Adapter Proof v1 - Closeout Audit - 2026-05-25

Status: closed
Last updated: 2026-05-25

## Objective

Close the optional plot adapter proof after adding `fret-plot/imui` as a declarative `UiWriter`
adapter without restoring retained plot code or adding plot dependencies to `fret-imui` /
`fret-ui-kit::imui`.

## Completion Checklist

| Requirement | Evidence |
| --- | --- |
| Optional `imui` feature exists in `fret-plot` | `ecosystem/fret-plot/Cargo.toml` |
| Adapter delegates to declarative plot panels | `ecosystem/fret-plot/src/imui.rs`, `ecosystem/fret-plot/src/lib.rs` |
| Default feature set still compiles | `docs/workstreams/imui-plot-adapter-proof-v1/EVIDENCE_AND_GATES.md` |
| `imui` feature compiles | `docs/workstreams/imui-plot-adapter-proof-v1/EVIDENCE_AND_GATES.md` |
| Source-policy test proves opt-in/declarative boundary | `ecosystem/fret-plot/src/lib.rs`, `tools/gate_imui_workstream_source.py` |

## Residual Boundaries

- Cookbook or canonical-workbench adoption remains deferred until product routes show repeated plot
  authoring friction.
- Root `fret::imui` plot sugar remains deferred until at least two product surfaces prove the same
  shorthand is needed.
- `fret-imui` and `fret-ui-kit::imui` must stay free of `fret-plot` dependencies.

## Outcome

The optional adapter proof is closed. Future plot authoring sugar belongs in a new proof-led lane.
