# Workstream TODO: Text Infrastructure v1

This is a checklist-style tracker. It is **non-normative**.

## Mechanism (`crates/`)

- [ ] Span semantics v1: expose interactive spans as semantics children (role=link, name=text).
- [ ] Span hit-test bounds v1: map span ranges → visual bounds for diagnostics/a11y (staged).
- [ ] Span hover routing v1: stable “hovered span” state in `SelectableText`.
- [ ] Span activation invariants: add tests that span activation does not interfere with selection drag.
- [ ] Mixed-direction staging: expand fixtures for RTL/mixed-direction link spans.

## Ecosystem (`ecosystem/`)

- [ ] Markdown link affordance: hover style policy (underline-on-hover vs always-underline).
- [ ] Markdown long-token policy: document when to use `Word` vs `WordBreak` per surface.
- [ ] UI kit cookbook: small “wrap recipes” doc with shadcn parity notes (labels vs prose vs tables).

## Tooling / gates

- [ ] Add a UI Gallery screenshot gate that clearly shows link underline + long-token wrapping in the same frame.
- [ ] Add a diagnostics gate for span semantics once span nodes exist (selector-based scripted click).

