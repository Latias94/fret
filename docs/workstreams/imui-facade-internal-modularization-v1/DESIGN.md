# ImUi Facade Internal Modularization v1

Status: active execution lane
Last updated: 2026-04-21

Related:

- `M0_BASELINE_AUDIT_2026-04-21.md`
- `M1_OPTIONS_RESPONSE_SLICE_2026-04-21.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`

This lane exists because the immediate-mode umbrella already froze the rule that new
implementation-heavy pressure should move into narrower follow-ons instead of expanding the
maintenance umbrella again.

The smallest credible follow-on is now:

> keep the public `fret-ui-kit::imui` surface stable while restructuring internals around explicit
> owners, so future narrow parity lanes can keep landing without another monolithic `imui.rs`
> rewrite.

## Why this is a new lane

This should not be forced back into `imui-editor-grade-product-closure-v1` because the immediate
problem is not another user-facing parity ask.

The risk is internal:

- `ecosystem/fret-ui-kit/src/imui.rs` still mixes the module hub, public re-exports, facade glue,
  and a large set of convenience wrappers.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs` still mixes stores, hover timing,
  lifecycle bookkeeping, disabled-scope helpers, and drag state transitions.
- `ecosystem/fret-ui-kit/src/imui/options.rs` and `ecosystem/fret-ui-kit/src/imui/response.rs`
  already represent stable outward vocabularies, but their internal organization is still too flat
  for low-risk iteration.

This lane is narrower than the umbrella:

- the umbrella keeps phase ordering and follow-on policy,
- this lane owns only internal `fret-ui-kit::imui` module decomposition with a public-surface
  freeze.

## Assumptions-first baseline

### 1) The current blocker is refactor hazard, not missing public surface.

- Evidence:
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would under-scope a real parity gap and spend time reorganizing code when a different
    owner should land behavior instead.

### 2) `crates/fret-ui` must remain unchanged.

- Evidence:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would drift from internal cleanup into forbidden runtime widening.

### 3) Closed narrow lanes remain closed.

- Evidence:
  - `docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - modularization work would become a backdoor for reopening older helper-growth debates.

### 4) The safest first slice is `options.rs` + `response.rs`.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui/options.rs`
  - `ecosystem/fret-ui-kit/src/imui/response.rs`
  - `ecosystem/fret-ui-kit/src/imui.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - the lane would start with a more coupled file and create unnecessary regression risk.

### 5) `interaction_runtime.rs` and `imui.rs` still need later dedicated slices.

- Evidence:
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would stop too early and leave the highest-risk owners unchanged.

## Goals

1. Keep the public `fret-ui-kit::imui` authoring surface stable while restructuring internal
   ownership.
2. Move stable outward vocabularies such as options and responses behind smaller owner files.
3. Make future narrow lanes cheaper to review by reducing hot-file size and ownership overlap.
4. Leave one repro set, one gate set, and one evidence set for each landable slice.

## Non-goals

- Do not widen `crates/fret-ui`.
- Widening `crates/fret-ui`.
- Reopening key-owner surface growth.
- Reopening collection/pane proof breadth.
- Reopening broader menu/tab policy depth.
- Shipping new helper semantics under the cover of module motion.

## Initial target surface

This lane does not start from zero.
The current outward surface already exists and should remain stable:

- `fret-ui-kit::imui::options::*`
- `fret-ui-kit::imui::response::*`
- `fret-ui-kit::imui::UiWriterUiKitExt`
- the existing helper families re-exported from `ecosystem/fret-ui-kit/src/imui.rs`

The first implementation slice should stay structural:

1. turn `options.rs` into a re-export hub over smaller private owner files,
2. turn `response.rs` into a re-export hub over smaller private owner files,
3. keep all public type names, defaults, and method names unchanged,
4. then use the resulting pattern to guide later `interaction_runtime.rs` and `imui.rs` slices.

## Default owner split

### `ecosystem/fret-ui-kit::imui::options`

Owns:

- stable outward option structs and enums,
- internal grouping by concern such as menus, controls, collections, containers, and misc,
- and any internal deduplication that does not change public field shape.

### `ecosystem/fret-ui-kit::imui::response`

Owns:

- stable outward response helper structs and query flags,
- internal grouping by concern such as drag/drop, hover/query state, floating surfaces, and
  helper-owned widget aggregates,
- and any internal method organization that does not change names or behavior.

### `ecosystem/fret-ui-kit::imui`

Owns:

- the public re-export hub,
- facade glue,
- and later decomposition of the large root file once the smaller vocabulary hubs are already
  stable.

### Not owned here

- `crates/fret-ui`
  - runtime/mechanism contract stays unchanged.
- policy-depth follow-ons
  - menu/tab policy, collection interaction depth, and key-owner depth remain separate owners.
- runner/backend parity
  - still belongs to the docking multi-window lane.

## Execution rules

1. Start with the lowest-risk structural slice that preserves the public facade exactly.
2. Keep each slice tied to a current proof build and focused test floor.
3. If a slice wants to rename public items, add new helper behavior, or widen contracts, stop and
   move that pressure into a different narrow lane instead of hiding it inside this refactor.
