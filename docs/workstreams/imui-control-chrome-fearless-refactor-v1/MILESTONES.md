# ImUi Control Chrome Fearless Refactor v1 - Milestones

Status: active execution lane
Last updated: 2026-04-20

## M0 - Baseline and lane split

Exit criteria:

- the lane is wired into the repo indexes,
- the assumptions-first baseline audit exists,
- the owner split is explicit,
- and the first repro/gate surface is frozen.

Current state:

- Achieved on 2026-04-14.

## M1 - Shared control chrome substrate

Exit criteria:

- `fret-ui-kit::imui` has one explicit shared control chrome owner for immediate controls,
- combo triggers no longer depend on selectable-row visuals,
- and button/switch/slider/combo/input migration has started on top of that shared owner.

Current state:

- Achieved on 2026-04-14 by landing `imui/control_chrome.rs` plus the first migration set.
- The lane also absorbed a popup keep-alive generation fix so IMUI popup lifetime still tracks real
  render passes after the control surface rewrite.

## M2 - Default control surface rewrite

Exit criteria:

- the first migration set no longer renders text-like default interactive surfaces,
- the old text-only visuals are deleted rather than kept as the default path,
- and compact editor-rail behavior is handled by the shared surface instead of demo patches.

Current state:

- In progress.
- The shared controls now ship button/field chrome by default.
- `imui_interaction_showcase_demo` now proves the shared button-family/radio surface directly.
- `bullet_text` now exists as a default IMUI informational helper, and the imgui audit no longer
  under-counts `separator_text`.
- `imui_shadcn_adapter_demo` now acts as the compact downstream-facing proof that shared IMUI
  helpers still read like controls when hosted in a shadcn shell.
- The fresh imgui component-family audit now makes the remaining follow-on order explicit.

## M3 - Proof and gates

Exit criteria:

- `imui_interaction_showcase_demo` proves the migrated surface,
- the current focused tests and showcase diag scripts pass,
- and the lane leaves one screenshot/diag evidence set that future follow-ons can compare against.

Current state:

- Achieved on 2026-04-20.
- Focused Rust gates are green and the compact showcase screenshot script produced a reviewable
  artifact.
- Focused compile/runtime gates now also cover the new button-family/radio surface.
- The adapter demo now has a direct compact control-discoverability screenshot/layout gate with a
  passing before/after artifact pair at `900x620`.
- A narrower field-width regression lane may still be warranted if future shared-field work moves
  beyond the current proof.

## M4 - Follow-on decision

Exit criteria:

- either the remaining work is small enough to close here,
- or the leftover scope is explicitly narrowed into a new follow-on instead of silently growing the
  lane.
