# Workstream TODO: Text Infrastructure v1

This is a checklist-style tracker. It is **non-normative**.

## Mechanism (`crates/`)

- [x] Span semantics v1: expose interactive spans as semantics metadata on the parent node (ADR 0283).
- [ ] Span semantics v2: expose interactive spans as semantics children (role=link, name=text, bounds).
- [x] Span hit-test bounds v1 (diagnostics): expose interactive span bounds for scripted clicks.
- [ ] Span hit-test bounds v2 (a11y): map span bounds to semantics children / platform accessibility.
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
