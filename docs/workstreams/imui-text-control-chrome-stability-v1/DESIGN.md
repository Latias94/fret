# ImUi Text Control Chrome Stability v1

Status: closed closeout record
Last updated: 2026-04-28

Related:

- `WORKSTREAM.json`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `M1_TEXT_CHROME_STABILITY_2026-04-28.md`
- `CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-edit-lifecycle-hardening-v1/CLOSEOUT_AUDIT_2026-04-25.md`

## Problem

The shared IMUI control-chrome rewrite closed with `text_controls.rs` listed as part of the shipped
surface, but the implementation still reused `crate::recipes::input::default_text_input_style`.

That recipe is correct for shadcn/new-york-v4 form inputs: focus changes the border color and paints
an outset ring. It is the wrong default for IMUI controls, where compact editor fields should keep a
stable visual footprint and use the immediate-control chrome vocabulary.

## Assumptions

- Area: lane routing
  - Assumption: this is a narrow follow-on, not a reopening of the closed shared control-chrome lane.
  - Evidence: `imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md` routes new generic IMUI
    chrome drift to narrower follow-ons.
  - Confidence: Confident
  - Consequence if wrong: work could be buried in a historical lane and become hard to resume.

- Area: layer ownership
  - Assumption: the fix belongs in `ecosystem/fret-ui-kit::imui`, not in `crates/fret-ui`.
  - Evidence: the runtime text widgets already accept paint-only `focus_ring` style, while IMUI
    chooses which chrome to pass.
  - Confidence: Confident
  - Consequence if wrong: a policy drift would be pushed into the mechanism layer.

- Area: shadcn parity
  - Assumption: shadcn keeps the outset ring for its own recipe surface, while IMUI should not borrow
    that recipe.
  - Evidence: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input.tsx` and `textarea.tsx` use
    `ring-[3px]` as visual chrome, not layout sizing.
  - Confidence: Confident
  - Consequence if wrong: Fret could regress shadcn parity while fixing IMUI.

## Target

IMUI text controls should:

- use compact IMUI field padding and 1px borders,
- keep fixed single-line input height at `FIELD_MIN_HEIGHT`,
- keep focus as a border-color change without an external ring,
- keep textarea padding/radius/border aligned with the same compact field vocabulary,
- leave shadcn form recipe chrome untouched.

## Non-Goals

- Widening `crates/fret-ui` text widget contracts.
- Changing shadcn input or textarea parity.
- Reworking text edit lifecycle, draft-buffer semantics, IME, or diagnostics publishing.
- Adding a public `fret-imui` styling API before a separate API-proof lane proves it.

## Proof Shape

The first proof is a direct `fret-ui-kit --features imui` unit test that renders
`input_text_model_with_options` and `textarea_model_with_options`, finds the resulting
`TextInputProps` / `TextAreaProps`, and asserts the compact chrome invariants.

The existing `fret-imui` bounds test remains a black-box guard for retained layout stability.
