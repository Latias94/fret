# ImUi Editor Cookbook Proof v1

Status: Closed
Last updated: 2026-05-04

Status note (2026-05-04): this lane closed after landing the public cookbook proof and facade
support noun re-export. Future editor-control behavior depth should start as a narrower follow-on.

## Problem

The IMUI stack already has editor-grade controls in `fret-ui-editor` and a thin adapter in
`fret-ui-editor::imui`, but the first public teaching surface still proves mostly action dispatch
and generic kit widgets. An app author should not have to discover or import `fret_ui_editor`
directly to use the editor-grade immediate-mode lane.

This lane closes that teaching-surface gap with one public cookbook proof:

- import `fret::imui::{editor, prelude::*}`,
- install the editor theme preset through `fret::imui::editor::theme`,
- build controls through `fret::imui::editor::{controls, ...}`,
- call thin immediate-mode adapters through `fret::imui::editor::*`,
- keep support nouns needed by the controls discoverable from the same facade.

## Ownership

- `ecosystem/fret`: app-facing facade and feature wiring.
- `ecosystem/fret-ui-editor`: editor-grade controls, support nouns, and immediate-mode adapters.
- `apps/fret-cookbook`: public teaching proof and source-policy regression tests.

Out of scope:

- changing `crates/fret-ui` runtime contracts,
- adding generic widgets to `fret-imui`,
- broadening `fret-ui-kit::imui` without a second proof surface,
- copying the large `imui_editor_proof_demo` into cookbook.

## Must-Be-True Outcomes

- App code can build editor-grade immediate-mode controls from `fret::imui::editor` without direct
  `fret_ui_editor` imports.
- The example is small enough to teach the public path, not a maintainer-only proof demo.
- The public path includes support nouns required by the controls, such as text-assist items.
- Source-policy tests prevent the cookbook example from drifting back to raw ecosystem crate imports.

## Reference Notes

Dear ImGui is the capability reference for the immediate authoring lane, but Fret keeps the
implementation split:

- `fret-imui`: authoring and host facade.
- `fret-ui-kit::imui`: generic policy-heavy widgets.
- `fret-ui-editor::imui`: editor-grade control adapters.
- `fret::imui`: app-facing root that teaches the usable path.
