# ImUi Facade Internal Modularization v1 - Milestones

Status: closed closeout record
Last updated: 2026-04-21

## M0 - Baseline and first-slice choice

Exit criteria:

- the repo explicitly records why internal modularization is a new narrow lane,
- the hottest files and their risks are named,
- and one low-risk first slice is frozen.

Primary evidence:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-21.md`
- `WORKSTREAM.json`

Current status:

- Completed on 2026-04-21 via `M0_BASELINE_AUDIT_2026-04-21.md`.

## M1 - Options/response structural split

Exit criteria:

- `options.rs` remains the stable outward hub over smaller private owner files,
- `response.rs` remains the stable outward hub over smaller private owner files,
- and the current first-party build/test surfaces still pass unchanged.

Primary evidence:

- `M1_OPTIONS_RESPONSE_SLICE_2026-04-21.md`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/options/`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui/response/`
- `EVIDENCE_AND_GATES.md`

Current status:

- Completed on 2026-04-21 via `M1_OPTIONS_RESPONSE_SLICE_2026-04-21.md`.

## M2 - Interaction-runtime owner split

Exit criteria:

- `interaction_runtime.rs` no longer mixes unrelated internal concerns in one flat module,
- hover/lifecycle/drag/disabled bookkeeping become reviewable as separate owners,
- and current interaction gates stay green.

Primary evidence:

- `M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/`
- `EVIDENCE_AND_GATES.md`

Current status:

- Completed on 2026-04-21 via `M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md`.
- `interaction_runtime.rs` now remains the stable outward hub while the implementation lives in
  `models.rs`, `disabled.rs`, `lifecycle.rs`, `hover.rs`, and `drag.rs`.
- The remaining structural hotspot is now the root `ecosystem/fret-ui-kit/src/imui.rs` hub rather
  than another flat interaction-runtime file.

## M3 - Root facade hub split

Exit criteria:

- `imui.rs` stops acting as one monolithic hub for re-exports, facade glue, and helper utilities,
- current public re-export names stay unchanged,
- and later policy/proof lanes can land with smaller diffs.

Primary evidence:

- `M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_support.rs`
- `ecosystem/fret-ui-kit/src/imui/floating_options.rs`
- `EVIDENCE_AND_GATES.md`

Current status:

- Completed on 2026-04-21 via `M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md`.
- `imui.rs` now remains the stable outward hub while support helpers and floating-surface facade
  types live in `facade_support.rs` and `floating_options.rs`.
- The remaining large owner is now the `ImUiFacade` / `UiWriterImUiFacadeExt` writer glue rather
  than the root file's mixed support + type block.

## M4 - Closeout or split again

Exit criteria:

- the remaining hotspot pressure is materially reduced,
- and any further work clearly belongs either to another structural slice or a different parity
  lane.

Primary evidence:

- `M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md`
- `CLOSEOUT_AUDIT_2026-04-21.md`
- `WORKSTREAM.json`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`

Current status:

- Completed on 2026-04-21 via `M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md` and
  `CLOSEOUT_AUDIT_2026-04-21.md`.
- `imui.rs` is now a thin outward hub while the remaining facade writer implementation lives in
  `facade_writer.rs` as one explicit owner.
- Further work should open a narrower follow-on instead of reopening this now-closed generic
  internal modularization lane.
