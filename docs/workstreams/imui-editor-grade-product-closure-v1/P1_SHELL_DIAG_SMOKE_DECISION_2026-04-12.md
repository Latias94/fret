# P1 Shell Diagnostics Smoke Decision - 2026-04-12

Status: focused P1 diagnostics decision / frozen shell smoke minimum

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `tools/diag-scripts/suites/diag-hardening-smoke-workspace/suite.json`

## Purpose

P1 needs one narrow diagnostics answer:

> which promoted launched smoke suite should represent the current workspace shell proof, and what
> minimum shell behaviors must it cover before P1 can claim reviewable shell diagnostics closure?

This note freezes that decision without creating a second competing shell suite.

## Audited evidence

- `tools/diag-scripts/suites/diag-hardening-smoke-workspace/suite.json`
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-closes-tab-smoke.json`
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-smoke.json`
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-drag-to-split-right-preview-invariants-smoke.json`
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-dirty-shows-prompt-and-discard-closes-smoke.json`
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tabstrip-escape-restores-content-focus-smoke.json`
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-file-tree-bounce-keep-alive.json`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`

## Assumptions-first resume set

1. Confident: `diag-hardening-smoke-workspace` already owns the promoted launched shell-smoke role.
   Evidence:
   - this lane already uses it as the canonical launched gate
   - `workspace-tabstrip-editor-grade-v1` already treats it as the promoted lightweight suite
   Consequence if wrong:
   - P1 would need a second suite name and a larger migration than this slice should carry.
2. Confident: the current suite is too tabstrip-heavy to stand in for the whole P1 shell story.
   Evidence:
   - the existing roster covers close/reorder/split-preview shell chrome,
   - but the P1 proof matrix also names dirty-close prompt, content focus restore, and file-tree
     split liveness as part of the coherent shell proof.
   Consequence if wrong:
   - this slice would widen diagnostics without a real product-closure need.
3. Confident: the missing shell-smoke categories already have reusable first-party scripts.
   Evidence:
   - `workspace-shell-demo-tab-close-dirty-shows-prompt-and-discard-closes-smoke.json`
   - `workspace-shell-demo-tabstrip-escape-restores-content-focus-smoke.json`
   - `workspace-shell-demo-file-tree-bounce-keep-alive.json`
   Consequence if wrong:
   - this slice would need new script authoring instead of a narrow promotion decision.
4. Likely: `editor_notes_demo` should remain secondary shell proof evidence, not a separate promoted
   diagnostics suite owner in this slice.
   Evidence:
   - `editor_notes_demo` is the minimal shell-mounted rail proof,
   - `workspace_shell_demo` remains the broadest launched shell proof and already owns the promoted
     diagnostics corpus.
   Consequence if wrong:
   - P1 would need a second diagnostics package before the shell proof order is stable enough.

## Current gap

Before this decision, `diag-hardening-smoke-workspace` proved that the workspace shell tabstrip can:

- close tabs,
- reorder tabs,
- split panes,
- and keep split-preview invariants stable.

That was useful, but insufficient for the broader P1 shell claim because it did not lock the
following shell behaviors into the promoted launched gate:

- dirty-close prompt and discard flow,
- content-focus restore via Escape after entering tabstrip focus,
- and left-rail / file-tree liveness under repeated shell interaction.

## Frozen shell smoke minimum

The promoted P1 shell smoke minimum must now cover these four behavior families:

1. Tab close / reorder / split preview
   - keep the existing close, reorder, and split-preview smoke scripts in the suite
2. Dirty-close prompt
   - require `workspace-shell-demo-tab-close-dirty-shows-prompt-and-discard-closes-smoke.json`
3. Content-focus restore via Escape
   - require `workspace-shell-demo-tabstrip-escape-restores-content-focus-smoke.json`
4. Left-rail / file-tree liveness
   - require `workspace-shell-demo-file-tree-bounce-keep-alive.json`

This is a minimum coverage floor, not a permanent full whitelist. Future shell smoke additions may
grow the suite, but P1 should not silently drop any of the four families above.

## Decision

From this point forward:

1. `diag-hardening-smoke-workspace` should remain the promoted P1 shell smoke suite.
2. Do not create a second parallel P1 shell suite yet.
3. The frozen minimum coverage for this suite must span:
   - tab close / reorder / split preview,
   - dirty-close prompt,
   - content-focus restore via Escape,
   - and left-rail / file-tree liveness.
4. `workspace_shell_demo` remains the launched target for this promoted suite.
5. `editor_notes_demo` stays secondary shell proof evidence and does not need its own promoted
   diagnostics suite in this slice.

## Immediate execution consequence

For this lane:

- widen the existing suite manifest instead of inventing a new suite name,
- keep the existing launched gate command unchanged,
- and treat the frozen minimum above as the default diagnostics floor before any P1 shell
  implementation-heavy follow-on starts.
