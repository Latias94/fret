# Workstream TODO: Text Strut + Leading Distribution v1

This is a checklist-style tracker. It is **non-normative**.

## Mechanism (`crates/`)

- [x] Define v1 types in `crates/fret-core`:
  - `TextLeadingDistribution`
  - `TextStrutStyle` (force + leading distribution + height spec)
- [ ] Define `TextHeightBehavior` (disable first ascent / last descent knobs).
- [x] Plumb types through render-text shaping/layout (Parley).
- [ ] Decide precedence rules:
  - style line height vs strut height
  - strut leading distribution vs per-style placement
- [x] Ensure cache keys / stable hashing include the new fields.

## Ecosystem (`ecosystem/`)

- [ ] Add opt-in presets in `fret-ui-kit::typography` for multiline “control-like” text areas.
- [ ] Audit form/text-area surfaces to decide default:
  - stable strut (UI-like forms) vs expand-to-fit (content/prose).

## Tooling / gates

- [x] Add a regression gate for strut stability under emoji + fallback runs.
- [ ] Add a regression gate for multiline strut stability under emoji + fallback runs.
- [ ] Add a UI Gallery diag script that captures the multiline case (optional).

## References (informative)

- Flutter: `repo-ref/flutter/engine/src/flutter/lib/web_ui/lib/src/engine/canvaskit/text.dart`
- GPUI/Zed: `repo-ref/zed/crates/gpui/src/text_system/line.rs` (half-leading baseline model)
