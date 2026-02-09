# ADR 1180: Link Semantics and `href` Activation (v1)

Status: Proposed

## Context

Fret already has:

- an accessibility role for links (`SemanticsRole::Link`),
- a semantics stamping mechanism (`AnyElement::attach_semantics(SemanticsDecoration)`, ADR 1161/1166),
- multiple ecosystem surfaces that want “link-like” behavior:
  - shadcn-style components exposing `.href(...)` (e.g. sidebar/menu items),
  - router adoption helpers (`ecosystem/fret-router-ui`) that build pressable navigation affordances.

Today, link behavior is inconsistent:

- Many “links” are implemented as `Pressable`, which by default activates on **Enter** *and* **Space**
  (button-like keyboard semantics).
- Some components stamp `SemanticsRole::Link`, but there is no shared contract for how to expose
  `href` for automation/diagnostics and copy-link affordances.

Links are a “hard-to-change” interaction contract surface because they affect:

- keyboard accessibility (expected activation keys),
- screen-reader semantics (role + value),
- overlay dismissal policy (menus should often treat modified-click differently),
- router integrations (navigation + prefetch + history).

This ADR locks a minimal v1 contract that can be enforced by tests and shared helpers.

## Decision

### 1) A Link is an interactive element with link semantics

A “Link” MUST produce a semantics node with:

- `role = SemanticsRole::Link`
- `actions.invoke = true` (activation is supported)

The Link MAY provide an accessible label via `label` (app/component-owned).

### 2) `href` is represented as semantics `value` (v1)

To keep the v1 surface small (no new semantics fields), a Link with an `href` MUST expose:

- `SemanticsNode.value = Some(href_string)`

This is intended for diagnostics/UI automation and desktop affordances like “Copy link”.

Notes:

- This does not imply that assistive technologies will announce `value` as a URL on every platform.
  Platform adapters may map it to an appropriate property when available.
- The `href` string should be canonical for the subsystem producing it (router links should use the
  router’s canonical `href`).

### 3) Keyboard activation keys: Link is **Enter-only**

For a Link:

- **Enter** and **NumpadEnter** MUST activate the link when focused.
- **Space** MUST NOT activate the link by default.

Rationale: align with common platform/web expectations (links activate on Enter; buttons on Space).

### 4) Pointer activation

For a Link:

- Primary-button click activates the link.
- Modified clicks (Ctrl/Meta/etc.) are forwarded to policy code (apps/components decide whether to
  open in a new window, copy link, suppress dismiss, etc.).

### 5) Shared helpers (recommended)

Ecosystem layers SHOULD avoid re-implementing link semantics ad-hoc.

Recommended v1 consolidation points:

- `ecosystem/fret-router-ui` link helpers SHOULD:
  - stamp `SemanticsRole::Link`
  - stamp `SemanticsDecoration.value(href)`
  - use Link keyboard activation semantics (Enter-only)
- shadcn `.href(...)` affordances SHOULD reuse a shared “link pressable” helper once available,
  rather than duplicating:
  - semantics stamping
  - modified-click suppression rules in menus
  - open-url fallback behavior

## Non-goals (v1)

- “Visited” state tracking or styling.
- Full `asChild`/slot-based prop merging (ADR 0117 explicitly avoids this in v1).
- Web-only anchor behaviors (download, target rel), beyond effect-level `OpenUrl` requests.
- A global policy for “open in new window/tab” (desktop apps vary).

## Consequences

- The `Pressable` mechanism needs a small configuration seam to support Link keyboard semantics
  without duplicating widgets.
- Router UI helpers can become a11y-correct by default, and shadcn `.href(...)` can be aligned
  without bespoke key handling.

## References

- `crates/fret-core/src/semantics.rs` (`SemanticsRole::Link`, `SemanticsNode.value`)
- ADR 1166: `docs/adr/1166-semantics-decoration-states-and-relations-v2.md`
- Router UI workstream: `docs/workstreams/router-ui-v1.md`
- GPUI reference link element (in-repo snapshot): `repo-ref/gpui-component/crates/ui/src/link.rs`

