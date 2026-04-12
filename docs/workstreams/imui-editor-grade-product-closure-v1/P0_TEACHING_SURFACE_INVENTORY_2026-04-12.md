# P0 Teaching Surface Inventory - 2026-04-12

Status: focused inventory / decision note

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `M0_BASELINE_AUDIT_2026-04-12.md`
- `docs/workstreams/imui-stack-fearless-refactor-v2/TEACHING_SURFACE_AUDIT_2026-03-31.md`

## Purpose

P0 needs two things before it can close:

1. a bounded inventory of the first-party immediate-mode teaching surfaces that readers are most
   likely to copy today,
2. and an explicit answer to which proof surface should pair with
   `apps/fret-examples/src/imui_editor_proof_demo.rs` as the second default path.

This note answers both.

## Audited surfaces

- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-examples/src/imui_hello_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_floating_windows_demo.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`
- source-policy gates in:
  - `apps/fret-cookbook/src/lib.rs`
  - `apps/fret-examples/src/lib.rs`
- `docs/examples/README.md`

## Classification table

| Surface | Classification | Why |
| --- | --- | --- |
| `apps/fret-cookbook/examples/imui_action_basics.rs` | Golden path (generic immediate authoring) | Uses `fret::app::prelude::*`, `cx.state()` / `cx.actions()` on the app lane, and demonstrates the immediate facade as one consumer of the same action pipeline as declarative and GenUI. |
| `apps/fret-examples/src/imui_editor_proof_demo.rs` | Golden path (editor-grade immediate proof) | Keeps the generic immediate facade plus official `fret_ui_editor::imui` adapters on the intended owner layers and is already guarded as the canonical editor proof. |
| `apps/fret-examples/src/imui_hello_demo.rs` | Reference / smoke | Useful for tiny facade smoke, but it still teaches the advanced prelude path and explicit model plumbing, so it should not be the default first-contact proof. |
| `apps/fret-examples/src/imui_response_signals_demo.rs` | Reference / behavior proof | Good for click/right-click/double-click/drag/context-menu semantics, but it is a capability probe, not the smallest golden-path authoring example. |
| `apps/fret-examples/src/imui_floating_windows_demo.rs` | Reference / product-validation proof | Explicitly labeled advanced/reference and already classified in `docs/examples/README.md` as a floating-window contract proof. |
| `apps/fret-examples/src/imui_shadcn_adapter_demo.rs` | Reference / adapter proof | Good proof that immediate control flow can host shadcn visuals plus table/virtual-list surfaces, but it is an adapter/product proof rather than the base teaching path. |
| `apps/fret-examples/src/imui_node_graph_demo.rs` | Historical / compatibility-only | Already explicitly labeled as retained-bridge compatibility proof and non-default downstream guidance. |

## Findings

### 1) The right second default proof is `imui_action_basics`, not `imui_hello_demo`

`imui_action_basics` is the strongest generic immediate teaching surface today because it teaches
the current public app lane rather than the older "advanced-first" posture:

- `use fret::app::prelude::*;`
- `fn init(app: &mut App, _window: WindowId) -> Self`
- `cx.state().local_init(...)`
- `cx.actions().local(...).update::<...>(...)`
- `use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;`
- `ui.action_button_with_options(...)`

That combination makes it the best companion to `imui_editor_proof_demo`:

- `imui_action_basics` teaches the smallest generic/default path,
- `imui_editor_proof_demo` teaches the heavier editor-grade path.

### 2) `imui_editor_proof_demo` should stay the editor-grade canonical proof

The editor proof remains the right canonical heavy proof because it already locks the owner split
in code and source-policy gates:

- generic immediate vocabulary stays on `fret_ui_kit::imui`,
- editor-owned nouns stay on `fret_ui_editor::imui`,
- and the proof explicitly uses `editor_imui::property_grid(...)` and
  `editor_imui::gradient_editor(...)`.

This means the golden pair should not be "two generic examples." It should be:

- one generic/default proof,
- one editor-grade proof.

### 3) `imui_hello_demo` should be demoted from "default first-contact" to "tiny smoke/reference"

`imui_hello_demo` is still useful, but it is not the best default teaching path anymore because it
leans on the advanced prelude and explicit `update_in(...)` / `value_in(...)` plumbing.

That makes it good for:

- a minimal facade smoke,
- or "smallest possible immediate demo" references,

but not for the main default authoring narrative when `imui_action_basics` already exists on the
app lane.

### 4) The other example surfaces are reference proofs, not golden-path docs

The remaining example set is still valuable, but each one is specialized:

- `imui_response_signals_demo` = interaction-query proof,
- `imui_floating_windows_demo` = overlap/floating proof,
- `imui_shadcn_adapter_demo` = adapter/shadcn proof,
- `imui_node_graph_demo` = retained-bridge compatibility proof.

They should stay in the repo and stay gated, but they should not be the default examples a new P0
guide points to first.

## Decision

The P0 golden pair is now:

1. Generic/default immediate authoring:
   `apps/fret-cookbook/examples/imui_action_basics.rs`
2. Editor-grade immediate authoring:
   `apps/fret-examples/src/imui_editor_proof_demo.rs`

Everything else remains valuable, but as reference or compatibility evidence.

## Immediate execution consequence

From this point forward:

1. use `imui_action_basics` as the second default proof surface named in P0,
2. keep `imui_editor_proof_demo` as the canonical editor-grade proof,
3. stop treating `imui_hello_demo` as the primary first-contact teaching surface,
4. keep `imui_floating_windows_demo`, `imui_response_signals_demo`, and
   `imui_shadcn_adapter_demo` in the reference/product-validation bucket,
5. keep `imui_node_graph_demo` explicitly compatibility-only.
