# Workstream TODO: Text Infrastructure v1

This is a checklist-style tracker. It is **non-normative**.

## Mechanism (`crates/`)

- [x] Span semantics v1: expose interactive spans as semantics metadata on the parent node (ADR 0283).
- [ ] Span semantics v2: expose interactive spans as semantics children (role=link, name=text, bounds).
- [x] Span hit-test bounds v1 (diagnostics): expose interactive span bounds for scripted clicks.
- [ ] Span hit-test bounds v2 (a11y): map span bounds to semantics children / platform accessibility.
- [x] Line box placement v1: add an opt-in mechanism for “half-leading baseline placement inside a fixed bounds height”.
  - Goal: avoid per-component y-nudge hacks in fixed-height controls (tabs, pills, buttons).
  - Evidence: `repo-ref/zed/crates/gpui/src/text_system/line.rs` (half-leading baseline formula).
  - Evidence: `crates/fret-core/src/text/mod.rs` (`TextVerticalPlacement::BoundsAsLineBox`)
  - Evidence: `ecosystem/fret-ui-kit/src/typography.rs` (`with_intent`, `fixed_line_box_style`)
- [ ] Span hover routing v1: stable “hovered span” state in `SelectableText`.
- [x] Span activation invariants: add tests that span activation does not interfere with selection drag.
- [ ] Mixed-direction staging: expand fixtures for RTL/mixed-direction link spans.

## Ecosystem (`ecosystem/`)

- [ ] Markdown link affordance: hover style policy (underline-on-hover vs always-underline).
- [ ] Markdown long-token policy: document when to use `Word` vs `WordBreak` per surface.
- [ ] UI kit cookbook: small “wrap recipes” doc with shadcn parity notes (labels vs prose vs tables).

## Tooling / gates

- [x] Add a UI Gallery screenshot gate that clearly shows link underline + long-token wrapping in the same frame.
- [x] Add a diagnostics gate for clickable spans (selector-based scripted click).
- [ ] Add a diagnostics gate for span semantics once span nodes exist (role/name/bounds selectors).

## Follow-ups (B: default policy)

- [ ] Evaluate flipping the default vertical placement policy once enough ecosystem components have migrated.
  - Prefer staging via UI Gallery screenshot gates before changing defaults.
