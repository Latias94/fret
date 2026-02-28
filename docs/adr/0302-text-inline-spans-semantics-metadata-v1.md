---
title: Text Inline Spans Semantics Metadata v1
status: Draft
date: 2026-02-19
---

# ADR 0302: Text Inline Spans Semantics Metadata v1

## Context

Fret document-like surfaces (Markdown, logs, rich prose) need **interactive inline spans** such as:

- links that can be activated by pointer and keyboard,
- hover affordances (cursor, underline),
- correct text wrapping behavior for long tokens inside a single paragraph layout model.

We added `SelectableText` span hit-testing and activation hooks, but our portability surface for
accessibility/automation (`SemanticsSnapshot`) only describes **node-level** semantics.

As a result:

- inline links are not represented as semantics (cannot be queried / asserted as links),
- platform accessibility backends have no structured hint that parts of a text node are interactive,
- diagnostics bundles cannot expose “which spans exist” without parsing widget-specific debug data.

At the same time, we do *not* want to prematurely commit to a full DOM-style inline formatting model
or to virtual per-span semantics nodes with stable ids and bounds (that would expand contracts and
automation selectors substantially).

## Decision

Extend `fret-core` semantics with **inline span metadata** attached to a `SemanticsNode`:

- Add `SemanticsInlineSpan` and `SemanticsNode::inline_spans: Vec<SemanticsInlineSpan>`.
- Inline spans are defined in terms of **UTF-8 byte ranges** into `SemanticsNode::value`.
- v1 is **metadata-only**:
  - it does not introduce additional semantics nodes,
  - it does not require bounds/rects for spans,
  - it does not change platform accessibility mapping yet.

This provides a stable, portable hook for:

- diagnostics/automation to assert link spans exist,
- future work to map inline spans to platform-specific a11y constructs when feasible,
- ecosystem components to describe “interactive text inside one paragraph” without tokenization.

## Semantics (v1)

### Range model

- `SemanticsInlineSpan::range_utf8` is a half-open byte range `(start, end)` into
  `SemanticsNode::value`.
- `start <= end` and `end <= value.len()` must hold.
- `start` and `end` must be UTF-8 character boundaries.
- Ranges may be empty (allowed but discouraged; consumers may ignore them).

### Role model

- `SemanticsInlineSpan::role` uses the existing `SemanticsRole` enum.
- v1 consumers should primarily expect `SemanticsRole::Link`.
- Future variants may include additional roles (e.g. `Button` for “inline command chips”).

### Tag

- `SemanticsInlineSpan::tag` is an opaque, component-defined string used for activation and/or
  automation (e.g. a URL for Markdown links).
- v1 does not define privacy/redaction semantics for tags; diagnostics tooling may choose to scrub
  or truncate tags when `redact_text` is enabled.

## Alternatives considered

1) **Virtual semantics nodes per span**
   - Pros: platform a11y could treat links as navigable children.
   - Cons: requires a stable id scheme, bounds/rect computation, selector/protocol expansion, and
     larger churn across diagnostics, accesskit mapping, and cross-backend parity.

2) **Diag-only span export**
   - Pros: no contract change in `fret-core`.
   - Cons: diagnostics becomes the only consumer; automation cannot depend on semantics; platform
     a11y integration remains blocked.

The chosen approach is the smallest contract that keeps future options open without committing to
virtual nodes yet.

## Consequences

- `SemanticsSnapshot` becomes strictly more expressive for text nodes.
- Producers (widgets/elements) must ensure that ranges are valid and bounded.
- AccessKit backends can ignore `inline_spans` initially; later milestones can map them when we have
  text geometry (span bounds/rects) and stable navigation semantics.

## Evidence / implementation anchors

- Semantics surface:
  - `crates/fret-core/src/semantics.rs`
- SelectableText semantics producer:
  - `crates/fret-ui/src/declarative/host_widget/semantics.rs`
- Semantics snapshot builder:
  - `crates/fret-ui/src/tree/ui_tree_semantics.rs`
