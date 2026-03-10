# Shimmer Text Style Source (Fearless Refactor v1) — TODO

Status: In progress.

Primary contract references:

- `docs/adr/0314-inherited-text-style-cascade-and-refinement-v1.md`
- `docs/adr/0315-shimmer-resolved-text-style-source-v1.md`

## A. Call-site inventory

| Surface | File | Current state | Desired end state | Why it matters | Priority | Status | Evidence / notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AI / plan | `PlanTitle` streaming path | `ecosystem/fret-ui-ai/src/elements/plan.rs` | Shared title scope + `Shimmer::use_resolved_passive_text()` | Landed on subtree-resolved style so streaming and non-streaming title share one semantic path | P0 | `[x]` | `ecosystem/fret-ui-ai/src/elements/plan.rs`, `plan_title_streaming_scopes_inherited_title_typography_for_shimmer` |
| AI / plan | `PlanDescription` streaming path | `ecosystem/fret-ui-ai/src/elements/plan.rs` | Shared description scope + `Shimmer::use_resolved_passive_text()` | Landed on subtree-resolved style so streaming and non-streaming description copy share one semantic path | P0 | `[x]` | `ecosystem/fret-ui-ai/src/elements/plan.rs`, `plan_description_streaming_scopes_inherited_description_typography_for_shimmer` |
| AI / reasoning | default thinking message | `ecosystem/fret-ui-ai/src/elements/reasoning.rs` | Trigger-owned subtree scope + `Shimmer::use_resolved_passive_text()` for streaming copy | Landed on trigger-local semantic typography so shimmer and settled copy share the same `text-sm text-muted-foreground` contract | P2 | `[x]` | `ecosystem/fret-ui-ai/src/elements/reasoning.rs`, `reasoning_trigger_default_streaming_message_scopes_inherited_typography_for_shimmer`, `reasoning_trigger_default_settled_message_scopes_inherited_typography` |
| AI / transcription | segment override path | `ecosystem/fret-ui-ai/src/elements/transcription.rs` | Caller-owned explicit text override on `TranscriptionSegment` | Intentionally remains explicit: segment typography is a public authoring seam, not a semantic shimmer default | P1 | `[~]` | `transcription_segment_text_style_override_applies_to_text_child` |
| AI / terminal | streaming status label | `ecosystem/fret-ui-ai/src/elements/terminal.rs` | Status-owned subtree scope + `Shimmer::use_resolved_passive_text()` | Landed on the header status slot typography so the default streaming label follows upstream `text-xs muted` ownership | P3 | `[x]` | `ecosystem/fret-ui-ai/src/elements/terminal.rs`, `terminal_status_default_streaming_message_scopes_inherited_typography_for_shimmer` |
| UI Gallery | shimmer demos | `apps/fret-ui-gallery/src/ui/snippets/ai/shimmer*.rs` | Default, explicit, and inherited-style examples now co-exist | Landed an inherited subtree-typography example while keeping explicit override demos as compatibility references | P3 | `[x]` | `apps/fret-ui-gallery/src/ui/snippets/ai/shimmer_elements_demo.rs`, `apps/fret-ui-gallery/src/ui/pages/ai_shimmer_demo.rs` |

Suggested review states:

- `[ ]` not done
- `[~]` audited / intentionally explicit / follow-up later
- `[x]` implemented and verified
- `[!]` blocked by contract decision

## B. Mechanism contract

- [x] SHIMMER-contract-001 Decide the minimum mechanism-level outcome for custom visual-text recipes.
  - Minimum outcome:
    - resolve the same effective passive text style / foreground as passive text leaves,
    - late enough for inherited text-style cascade to participate,
    - stable enough for measure + base text + overlay paint.
- [x] SHIMMER-contract-002 Decide whether the mechanism returns:
  - a resolved snapshot,
  - a resolver callback,
  - or another runtime-owned bridge.
- [x] SHIMMER-contract-003 Decide whether inherited foreground is part of the same bridge or a
  sibling contract.
  - Landed as a resolved-snapshot bridge on `CanvasPainter`, with inherited foreground exposed alongside
    resolved passive text style in `crates/fret-ui/src/canvas.rs`.

## C. `Shimmer` bridge

- [x] SHIMMER-impl-010 Keep `.text_style(TextStyle)` as a compatibility path.
- [x] SHIMMER-impl-011 Add a subtree-resolved style source mode for `Shimmer`.
- [x] SHIMMER-impl-012 Ensure base text and overlay painter share one resolved snapshot.
- [x] SHIMMER-impl-013 Prove wrap / overflow / baseline remain aligned under the new mode.
  - Locked by `shimmer_resolved_mode_keeps_wrap_overflow_and_baseline_aligned` in `ecosystem/fret-ui-ai/src/elements/shimmer.rs`.

## D. Semantic migration

- [x] SHIMMER-migrate-020 Migrate `PlanTitle` streaming path to the subtree-resolved mode.
- [x] SHIMMER-migrate-021 Migrate `PlanDescription` streaming path to the subtree-resolved mode.
- [x] SHIMMER-migrate-022 Re-audit `Reasoning` / `Terminal` / gallery demos after the bridge lands.
  - `Reasoning` and `TerminalStatus` now consume subtree-resolved typography, and gallery demos now include an inherited-style example alongside explicit compatibility cases.
- [x] SHIMMER-migrate-023 Keep `Transcription` explicit override behavior intact.
  - Locked by `transcription_segment_text_style_override_applies_to_text_child`.

## E. Gates

- [x] SHIMMER-gates-030 Add a test proving subtree-resolved shimmer style matches passive text
  resolution under the same inherited scope.
- [x] SHIMMER-gates-031 Add a test proving explicit `.text_style(...)` still wins over inherited
  subtree style.
- [x] SHIMMER-gates-032 Add a focused plan streaming gate (unit test or diag script) proving the
  streaming description path no longer rebuilds card typography manually.

## F. Docs / alignment

- [x] SHIMMER-docs-040 Add this workstream doc set.
- [x] SHIMMER-docs-041 Add ADR 0315 as the proposed contract note.
- [x] SHIMMER-docs-042 Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` once the mechanism lands.
- [x] SHIMMER-docs-043 Update the text-style cascade workstream when `PlanDescription` moves from
  partial to fully aligned.
