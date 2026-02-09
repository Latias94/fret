---
title: Shadcn Extras (`fret-ui-shadcn::extras`) — Milestones
status: draft
date: 2026-02-09
scope: ecosystem/fret-ui-shadcn
---

# Shadcn Extras (`fret-ui-shadcn::extras`) — Milestones

This is a one-screen milestone board for the shadcn extras workstream.

Narrative + boundaries: `docs/workstreams/shadcn-extras.md`  
TODO tracker: `docs/workstreams/shadcn-extras-todo.md`

## M0 — Skeleton + conventions

Exit criteria:

- `extras` module exists and is documented.
- `fret-ui-shadcn` crate root exports `extras` as a module (no root re-exports).
- At least one snapshot gate exists for an extras component.
- A “how to add an extras component” checklist exists (controlled/uncontrolled, semantics, gates).

## M1 — App-common, low-risk blocks

Exit criteria:

- Land 3–5 small extras components with snapshot gates:
  - `Banner`
  - `Announcement`
  - `Tags` (static)
  - `Rating` (with at least one deterministic keyboard interaction gate)
  - `RelativeTime` (display-only)
- No changes to `crates/fret-ui` public contract surface.

## M2 — Medium complexity composition

Exit criteria:

- `AvatarStack` lands with a stable visual/semantics snapshot.
- A decision note exists for `Snippet`/`CodeBlock` ownership:
  - stays in extras, or
  - owned by `fret-ui-ai` / `fret-markdown` ecosystems (preferred when policy-heavy).

Decision note: documented in `docs/workstreams/shadcn-extras.md` (“Ownership decision: `Snippet` / `CodeBlock`”).

## M3 — Scheduling-heavy blocks (deferred)

Exit criteria:

- A stable authoring pattern is documented for timer-driven / continuously animating extras.
- Each scheduling-heavy component ships with:
  - deterministic scripted gates, and
  - at least one perf regression note or diag gate where appropriate.
