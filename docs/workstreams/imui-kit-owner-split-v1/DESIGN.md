# ImUi Kit Owner Split v1

Status: active execution lane
Last updated: 2026-05-13

Related:

- `WORKSTREAM.json`
- `M0_BASELINE_AUDIT_2026-05-13.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_EXECUTION_PRIORITY_REVIEW_2026-05-06.md`
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_PUBLIC_SURFACE_CATALOG_2026-05-06.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`

This lane is a narrow follow-on from the current Dear ImGui gap-closure audit. The objective is not
to chase every Dear ImGui API. The objective is to make the current `fret-ui-kit::imui` policy layer
safe enough for future Dear ImGui-class editor work by reducing large-owner coupling and deleting
local duplication when the evidence is strong.

## Assumptions First

### 1) `fret-imui` stays thin.

- Evidence:
  - `ecosystem/fret-imui/src/lib.rs`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_PUBLIC_SURFACE_CATALOG_2026-05-06.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would accidentally create a second widget/runtime layer and weaken the existing
    mechanism/policy split.

### 2) This lane owns private policy-layer structure, not runtime contracts.

- Evidence:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - any change to `crates/fret-ui` public behavior would need an ADR-backed runtime lane instead of
    being hidden inside an IMUI cleanup.

### 3) `facade_writer.rs` is the next structural risk after debug draw.

- Evidence:
  - `docs/workstreams/imui-debug-draw-owner-split-v1/CLOSEOUT_AUDIT_2026-05-06.md`
  - `docs/workstreams/imui-kit-owner-split-v1/M0_BASELINE_AUDIT_2026-05-13.md`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- Confidence:
  - Likely
- Consequence if wrong:
  - a different local owner, such as menu/table/floating policy, should become the first slice after
    a focused baseline audit.

### 4) Public names must stay stable during owner splits.

- Evidence:
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_PUBLIC_SURFACE_CATALOG_2026-05-06.md`
  - `ecosystem/fret/src/lib.rs`
  - `ecosystem/fret-ui-kit/src/imui.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the work becomes an API migration and needs facade/source teaching updates plus broader gates.

### 5) Deletion is allowed, but only after proving the code is duplicate or no longer taught.

- Evidence:
  - `docs/workstreams/imui-imgui-gap-closure-v1/P1_CLOSEOUT_AUDIT_2026-05-06.md`
  - `tools/gate_imui_facade_teaching_source.py`
- Confidence:
  - Confident
- Consequence if wrong:
  - deleting canonical `*_with_options(...)` entry points or adapter seams would break valid
    authoring paths.

## Goals

1. Split private `fret-ui-kit::imui` owners where one file owns unrelated policy concerns.
2. Delete local duplication when a shared helper or behavior kernel already exists.
3. Keep `fret-imui` policy-light and unchanged unless a narrow authoring-control-flow issue is
   proven.
4. Keep public `fret-ui-kit::imui` item names, `fret::imui` re-exports, and cookbook teaching
   paths stable during structural slices.
5. Leave every slice with repro, gate, and evidence.

## Non-Goals

- No broad "finish all Dear ImGui widgets" backlog.
- No public API widening from this lane.
- No `crates/fret-ui` runtime contract changes.
- No docking, multi-window, or backend behavior changes.
- No style-stack compatibility layer.
- No porting-sugar helpers such as `SameLine` or item-width stacks without the two-surface proof
  rule.

## Initial Target Split

### `facade_writer.rs`

Keep the public `ImUiFacade` method surface, but split private implementation helpers behind owner
modules where possible. The first likely slices are:

- command/action button and menu item glue,
- response/status population glue,
- table/tab/menu container construction helpers,
- text input facade wiring.

The public method names should remain in `ImUiFacade`; private element construction or response
assembly can move behind narrower modules.

### `facade_support.rs` and `interaction_runtime/*`

Audit the transient-key/status helpers and shared per-frame models. If the current key/value
transient flow remains the right contract, keep it. If duplication is visible, introduce private
typed helpers without widening `ResponseExt` or `fret-authoring::Response`.

### Existing specialized owners

Do not collapse already split owners such as `debug_draw_controls/*`, `interaction_runtime/*`,
`options/*`, or `response/*`.

## Execution Rules

1. Freeze public names before moving code.
2. Prefer private modules and `pub(super)` over new public exports.
3. Delete code only when the proof says it is duplicate, stale, or no longer taught.
4. Run the focused `fret-ui-kit` IMUI gates after each code slice.
5. Update this lane with a dated M-note after each meaningful owner split or deletion verdict.
