# Page Shell Audit — 2026-04-02

Status: Closed

## Scope

This audit closes `M3` for `shell-composition-fearless-refactor-v1`.

Question:

- Do first-party page-shell consumers already share one reusable layout contract that should move
  into a dedicated ecosystem owner now?

Short answer:

- No. Current first-party consumers represent different shell semantics and should remain
  app-owned in v1.

## Decision

Do not promote a shared page-shell surface in this lane.

Keep the current owners explicit:

- `apps/fret-cookbook/src/scaffold.rs` remains the cookbook lesson shell.
- `apps/fret-ui-gallery/src/ui/doc_layout.rs` remains the UI Gallery docs scaffold.
- `apps/fret-examples/src/todo_demo.rs` keeps its demo-specific responsive shell local.

Promotion remains gated on a future source audit that proves at least three aligned first-party
consumers with materially similar layout semantics.

## Findings

### 1) Cookbook uses a lesson shell, not a general product shell

`apps/fret-cookbook/src/scaffold.rs` is explicit about its scope:

- the file-level comment says the helper keeps the cookbook page shell consistent without turning
  the cookbook crate into a reusable product API,
- `centered_page` builds one centered primary surface,
- it owns only background token choice, outer padding, centering, and a stable root `test_id`.

Evidence:

- `apps/fret-cookbook/src/scaffold.rs:1`
- `apps/fret-cookbook/src/scaffold.rs:10`
- `apps/fret-cookbook/src/scaffold.rs:25`

The cookbook crate also already protects this teaching surface as app-owned:

- source tests assert the scaffold uses `fret::app::prelude::*`,
- the helper returns `Ui` directly from an app surface,
- examples are expected to use the unified centered-page helpers rather than inventing another
  local shell shape.

Evidence:

- `apps/fret-cookbook/src/lib.rs:637`
- `apps/fret-cookbook/src/lib.rs:659`
- `apps/fret-cookbook/src/lib.rs:710`

### 2) UI Gallery uses a docs scaffold, not the same centered-card shell

`apps/fret-ui-gallery/src/ui/doc_layout.rs` solves a different problem:

- `DocSection` carries doc metadata, preview content, optional code blocks, tab sizing, and
  per-section shell toggles,
- `render_doc_page` builds a docs column with multiple sections and intro copy,
- `wrap_preview_page` keeps preview harness pages inside the same docs scaffold,
- `render_section` can selectively add or skip shell chrome per section via `shell` and
  `.no_shell()`,
- `demo_shell` is only one decoration step inside a larger docs/page framework.

Evidence:

- `apps/fret-ui-gallery/src/ui/doc_layout.rs:6`
- `apps/fret-ui-gallery/src/ui/doc_layout.rs:153`
- `apps/fret-ui-gallery/src/ui/doc_layout.rs:214`
- `apps/fret-ui-gallery/src/ui/doc_layout.rs:514`
- `apps/fret-ui-gallery/src/ui/doc_layout.rs:620`

This is a gallery/docs scaffold with preview/code composition, not a generic "ordinary app page"
helper.

### 3) `todo_demo` uses a product-specific responsive shell

`apps/fret-examples/src/todo_demo.rs` does not consume a shared page-shell helper. It defines its
own shell behavior from viewport and pointer capability:

- `TodoResponsiveLayout` derives page padding, top padding, footer stacking, row-action visibility,
  card max height, and rows max height from viewport width/height and hover capability,
- the card shell clamps width and height for the demo's product composition,
- `todo_page` applies the outer page padding/background separately from the card.

Evidence:

- `apps/fret-examples/src/todo_demo.rs:55`
- `apps/fret-examples/src/todo_demo.rs:67`
- `apps/fret-examples/src/todo_demo.rs:594`
- `apps/fret-examples/src/todo_demo.rs:640`
- `apps/fret-examples/src/todo_demo.rs:648`

This is product-demo composition, not the cookbook lesson shell and not the UI Gallery docs shell.

## Why promotion is rejected now

The three audited consumers do not share one stable layout contract:

- cookbook: single primary surface centered inside a simple lesson page,
- UI Gallery: multi-section docs page with optional preview shells and preview/code tabs,
- todo demo: responsive app-specific composition driven by viewport and input capability.

Because the semantics differ, extracting a shared surface now would create a false abstraction and
likely force another breaking rename or split later.

## Promotion rule retained for future work

A page-shell helper may graduate into a reusable ecosystem surface only when all of the following
are true:

1. At least three first-party consumers are aligned on materially similar layout semantics.
2. The helper shape is no longer tied to one app's teaching surface or docs scaffold.
3. One explicit owning ecosystem lane is selected.
4. The promoted helper stays off the default `fret` root and prelude shortcuts.
5. A source-policy or first-party usage gate proves that the chosen owner is real and stable.

## Outcome for this workstream

`M3` is complete with a "do not promote" decision.

The next useful slices stay focused on:

- `M4` workspace-shell owner consolidation,
- `M5` teaching-surface cleanup,
- `M6` gates that prevent shell-owner drift and old names from returning.
