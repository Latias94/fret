# Shimmer Text Style Source (Fearless Refactor v1) — TODO

Status: In progress.

Primary contract references:

- `docs/adr/0314-inherited-text-style-cascade-and-refinement-v1.md`
- `docs/adr/0315-shimmer-resolved-text-style-source-v1.md`

## A. Call-site inventory

| Surface | File | Current state | Desired end state | Why it matters | Priority | Status | Evidence / notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AI / plan | `PlanTitle` streaming path | `ecosystem/fret-ui-ai/src/elements/plan.rs` | Manual `card_title_text_style(...)` + `Shimmer::text_style(...)` | Consume subtree-resolved style so streaming and non-streaming title share one semantic path | P0 | `[ ]` | `ecosystem/fret-ui-ai/src/elements/plan.rs:312` |
| AI / plan | `PlanDescription` streaming path | `ecosystem/fret-ui-ai/src/elements/plan.rs` | Shared description scope + `Shimmer::use_resolved_passive_text()` | Landed on subtree-resolved style so streaming and non-streaming description copy share one semantic path | P0 | `[x]` | `ecosystem/fret-ui-ai/src/elements/plan.rs`, `plan_description_streaming_scopes_inherited_description_typography_for_shimmer` |
| AI / reasoning | default thinking message | `ecosystem/fret-ui-ai/src/elements/reasoning.rs` | Explicit `Sm` visual text via `Shimmer::text_style(...)` | Likely intentional visual ownership; audit after bridge lands | P2 | `[~]` | `ecosystem/fret-ui-ai/src/elements/reasoning.rs:431` |
| AI / transcription | segment override path | `ecosystem/fret-ui-ai/src/elements/transcription.rs` | Caller-owned explicit override propagated into shimmer | Must remain supported; this is an explicit-authoring compatibility case | P1 | `[~]` | `ecosystem/fret-ui-ai/src/elements/transcription.rs:489` |
| AI / terminal | streaming status label | `ecosystem/fret-ui-ai/src/elements/terminal.rs` | Default shimmer with no explicit style override | May stay on theme default unless terminal typography later becomes semantic | P3 | `[~]` | `ecosystem/fret-ui-ai/src/elements/terminal.rs:460` |
| UI Gallery | shimmer demos | `apps/fret-ui-gallery/src/ui/snippets/ai/shimmer*.rs` | Mix of default and explicit style demo cases | Keep explicit override demos working while adding one inherited-style demo later | P3 | `[~]` | Demo-only compatibility surface |

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
- [ ] SHIMMER-impl-013 Prove wrap / overflow / baseline remain aligned under the new mode.

## D. Semantic migration

- [ ] SHIMMER-migrate-020 Migrate `PlanTitle` streaming path to the subtree-resolved mode.
- [x] SHIMMER-migrate-021 Migrate `PlanDescription` streaming path to the subtree-resolved mode.
- [ ] SHIMMER-migrate-022 Re-audit `Reasoning` / `Terminal` / gallery demos after the bridge lands.
- [ ] SHIMMER-migrate-023 Keep `Transcription` explicit override behavior intact.

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
