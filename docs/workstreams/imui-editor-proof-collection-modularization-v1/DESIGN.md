# ImUi Editor Proof Collection Modularization v1

Status: closed closeout reference
Last updated: 2026-04-23

Status note (2026-04-23): this document remains the lane-opening rationale. The shipped verdict now
lives in `M1_DEMO_LOCAL_COLLECTION_MODULE_SLICE_2026-04-23.md` and
`CLOSEOUT_AUDIT_2026-04-23.md`.

Related:

- `M0_BASELINE_AUDIT_2026-04-23.md`
- `M1_DEMO_LOCAL_COLLECTION_MODULE_SLICE_2026-04-23.md`
- `CLOSEOUT_AUDIT_2026-04-23.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-collection-inline-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `apps/fret-examples/tests/imui_editor_collection_modularization_surface.rs`

This lane exists because:

The closed collection inline rename lane already landed the current app-owned collection product depth, but the host proof still kept too much collection implementation in one file.

The narrow remaining question is now:

> can the current collection-first proof move into a demo-local module so the host stops
> accumulating app-owned collection helpers, while still keeping shared IMUI/runtime surface growth
> explicitly out of scope?

## Why this is a new lane

This work should not be forced back into `imui-collection-inline-rename-v1`.

That folder is already closed on a bounded product-depth verdict. Reopening it would blur:

- product depth already shipped on the collection-first proof,
- structural maintenance cleanup that belongs to the owning demo,
- and future follow-ons such as a broader command package or a second real proof surface.

This lane also should not widen `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui`.
A host-file maintenance problem is not evidence for a new framework contract.

## Assumptions-first baseline

### 1) The closed collection inline rename lane already landed the current app-owned product depth.

- Evidence:
  - `docs/workstreams/imui-collection-inline-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this folder would muddy product-depth closeout instead of owning a narrow structural slice.

### 2) The current collection proof now spans enough owner-local state and behavior that host-file shape is an architecture concern.

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - we would treat a real maintenance hazard as harmless demo clutter and keep compounding it.

### 3) A demo-local submodule is sufficient to reduce the host-file pressure without creating a public API.

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `apps/fret-examples/src/imui_editor_proof_demo/`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would either under-deliver structurally or accidentally widen shared surface area.

### 4) The frozen proof-budget rule still blocks shared helper growth from one proof surface.

- Evidence:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - we would misread local modularization pressure as framework API justification.

## Goals

1. Move the collection-first proof into `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`.
2. Keep the host file on `mod collection;`, one render call, and drag-asset delegation.
3. Preserve the existing app-owned collection behavior and local unit/surface/source-policy gates.
4. Close the lane again once the structural slice is reviewable.

## Non-goals

- Widening `fret-imui`.
- Widening public `fret-ui-kit::imui`.
- Changing `crates/fret-ui` runtime contracts.
- Adding a broader collection command package.
- Adding a second proof surface.

## Initial target surface

The first correct target is therefore structural rather than behavioral:

1. move the collection proof into `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`,
2. keep the host file on `mod collection;` plus one render call and drag-asset delegation,
3. move collection unit tests beside the module,
4. add one explicit modularization surface/source-policy floor,
5. and leave broader collection command or helper pressure to later follow-ons.

## Default owner split

### `apps/fret-examples`

Owns:

- the demo-local collection module,
- the host/module boundary,
- the demo-local unit tests and surface tests,
- and the source-policy lane freeze.

### Not owned here

- `fret-imui`
  - no new immediate facade API.
- `fret-ui-kit::imui`
  - no new shared collection helper package.
- `crates/fret-ui`
  - no runtime/mechanism contract change.
- broader command-package / second-proof-surface questions
  - remain separate follow-ons.

## First landable target

Do not widen `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` for a demo-local maintenance problem.

The first correct target is:

- one demo-local `collection.rs` module that owns collection data/models/render/tests,
- one slim host file that calls into that module explicitly,
- one modularization surface/source-policy package that keeps the boundary visible,
- and one closeout that resets the next default non-multi-window priority to command-package
  breadth rather than more host-file accretion.
