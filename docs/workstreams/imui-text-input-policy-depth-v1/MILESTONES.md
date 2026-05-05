# ImUi Text Input Policy Depth v1 Milestones

Status: Closed
Last updated: 2026-05-04

Closeout note (2026-05-04): the original text-input policy-depth package is complete. Remaining
numeric, editor-assist, and deeper multiline work should continue as narrower follow-ons.

## M0 - Baseline

Exit criteria:

- Dear ImGui text-input flags relevant to the first slice are identified.
- Existing Fret text input ownership is mapped across `fret-ui`, `fret-ui-kit::imui`, and
  `fret-imui` tests.

State: Complete for the first slice.

## M1 - Read-Only And Select-All-On-Focus

Exit criteria:

- Runtime text controls expose a read-only mechanism and enforce it across commands/events/platform
  replacement.
- IMUI text options expose read-only and select-all-on-focus without moving policy into
  `crates/fret-ui`.
- Regression tests cover read-only mutation blocking and focus-time selection behavior.
- `ImUi::push_id` uses explicit key identity rather than source-location identity, keeping
  model-backed `changed()` stable across repeated render closures and reorder.

State: Complete for the first slice.

## M2 - Public Cookbook Proof

Exit criteria:

- A real app-facing cookbook example reaches the new IMUI text-input options through
  `fret::imui::{prelude::*, kit}`.
- The example continues to avoid direct `fret-ui-kit` / `fret-imui` imports and remains locked by a
  source-policy test.

State: Complete for this slice.

## M3 - Multiline Tab Input Policy

Exit criteria:

- Runtime text area exposes a mechanism flag for Tab insertion.
- IMUI textarea defaults to not inserting Tab, matching Dear ImGui's opt-in `AllowTabInput` posture.
- `TextAreaOptions::allow_tab_input=true` inserts `\t` and reports `changed()`.
- Focused runtime and IMUI tests cover both blocked/default and opt-in paths.

State: Complete for this slice.

## M4 - Callback And Editing Policy Audit

Exit criteria:

- History/completion/filter/callback-edit parity is split into smaller lanes with explicit owner
  layers.
- No broad public IMUI API widening happens without proof surfaces and gates.

State: Complete for this lane. History/completion command routing, named/custom insertion filters,
app-owned undo/redo command routing, visible completion/history picker UI, and picker keyboard
navigation were split and closed in narrower follow-ons. Numeric scalar text-edit fallback,
editor-owned ranking/storage, active-descendant picker accessibility, and deeper multiline behavior
remain future follow-ons rather than open scope here.
