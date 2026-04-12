# P0 Demote/Delete Plan - 2026-04-12

Status: decision note

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `P0_TEACHING_SURFACE_INVENTORY_2026-04-12.md`
- `P0_FOOTGUN_AUDIT_2026-04-12.md`

## Purpose

The earlier P0 notes answered:

- which surfaces form the golden pair,
- and which remaining footguns are mostly teaching/proof-shape problems.

This note turns those findings into a concrete demote/keep/delete plan for the current first-party
surfaces.

## Plan summary

### Promote as the immediate-mode golden pair

Keep these as the explicit first-open path for readers who intentionally want the immediate-mode
lane:

1. Generic/default immediate proof:
   - `apps/fret-cookbook/examples/imui_action_basics.rs`
2. Editor-grade immediate proof:
   - `apps/fret-examples/src/imui_editor_proof_demo.rs`

### Demote from first-contact teaching to reference/smoke

- `apps/fret-examples/src/imui_hello_demo.rs`

Reason:

- it is still useful,
- but it is no longer the best default teaching surface once `imui_action_basics` exists on the
  app lane.

### Keep as reference/product-validation proofs

- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/imui_floating_windows_demo.rs`

Reason:

- these surfaces are valuable, but they are capability or product proofs rather than the smallest
  golden-path teaching surfaces.

### Keep as compatibility-only proof

- `apps/fret-examples/src/imui_node_graph_demo.rs`

Reason:

- it is still useful retained-bridge evidence,
- but it should remain explicitly non-default.

### Delete now?

No immediate file deletion is recommended.

Why:

- the problem is currently message clarity, not surplus example count,
- and each of the non-golden examples still carries distinct proof value.

If a later cleanup lane wants to delete or merge any of them, that should happen only after:

- the public docs consistently route readers to the golden pair first,
- and the corresponding proof value is either moved elsewhere or no longer needed.

## Required doc and gate changes

### Public docs

- `docs/examples/README.md` should explicitly name the immediate-mode golden pair.
- The same doc should explicitly classify:
  - `imui_hello_demo` as smoke/reference,
  - `imui_response_signals_demo` and `imui_shadcn_adapter_demo` as reference proofs,
  - `imui_floating_windows_demo` as advanced/reference,
  - `imui_node_graph_demo` as compatibility-only.

### Source-policy gates

Keep the existing "current facade surface" gate, but add explicit classification checks for:

- the public docs naming the golden pair,
- `imui_hello_demo` staying demoted,
- and `imui_node_graph_demo` staying compatibility-only.

## Immediate execution consequence

From this point forward:

1. stop treating `imui_hello_demo` as the best first-contact immediate example,
2. route public immediate-mode readers through:
   `imui_action_basics` -> `imui_editor_proof_demo`,
3. keep the reference and compatibility examples, but make their role explicit in docs and tests,
4. postpone actual file deletion until the product-message cleanup has clearly stuck.
