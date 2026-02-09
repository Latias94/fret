---
title: Shadcn Extras (`fret-ui-shadcn::extras`)
status: draft
date: 2026-02-09
scope: ecosystem/fret-ui-shadcn, component ecosystem, blocks/recipes
---

# Shadcn Extras (`fret-ui-shadcn::extras`) — Workstream

This workstream defines and tracks an **extras** surface under `ecosystem/fret-ui-shadcn`:

- Target module: `ecosystem/fret-ui-shadcn/src/extras/*`
- Goal: provide a small, high-signal set of **shadcn-styled blocks/recipes** that sit *above* the
  shadcn/ui v4-aligned taxonomy surface.
- Sources: permissive-licensed “shadcn blocks” ecosystems (e.g. `repo-ref/kibo`) and other
  permissive upstreams that are compatible with Fret’s layering rules.

This is **not** shadcn/ui v4 parity work.

- v4 parity and goldens remain owned by `ecosystem/fret-ui-shadcn` (root modules) and tracked in:
  `docs/shadcn-declarative-progress.md`.

## Why this workstream exists

shadcn/ui v4 provides a strong **taxonomy + primitives + recipes** baseline, but “real apps” quickly
need additional, composable building blocks (banners, announcements, rating controls, tag inputs,
etc.) that are still “shadcn-feeling”.

On the web, this gap is commonly filled by “shadcn blocks” registries (Kibo, AI Elements, etc.). In
Fret we want the same outcome, while keeping the **mechanism vs policy** boundary intact (ADR 0066).

## Layering (non-negotiable)

Follow the same split used by the rest of the component ecosystem:

- `crates/fret-ui`: mechanisms/contracts only.
- `ecosystem/fret-ui-headless` + `ecosystem/fret-ui-kit`: state machines + reusable infra/policy helpers.
- `ecosystem/fret-ui-shadcn`: shadcn v4 naming/taxonomy + recipes.
- `ecosystem/fret-ui-shadcn::extras`: **blocks/recipes that are not part of v4 taxonomy**.

Important boundary:

- AI-native, policy-heavy surfaces are **out of scope** here. They are owned by
  `ecosystem/fret-ui-ai` and tracked in `docs/workstreams/ai-elements-port.md`.

## Public surface rules (how we keep the boundary enforceable)

1. `extras` is a **module**, not a new taxonomy.
   - `pub mod extras;` is allowed.
   - Do **not** `pub use extras::*` from `fret-ui-shadcn` crate root.
     - Rationale: keep the default shadcn v4 surface “clean” and keep IDE autocomplete aligned with parity.
2. No “same name, different semantics” duplication:
   - If a component exists in the v4 surface, `extras` must not introduce another component with the
     same name but different behavior.
3. No runtime contract creep:
   - `extras` must not require expanding `crates/fret-ui` public contracts. If a gap is found, it
     must be proposed via an ADR and justified independently (ADR 0066).
4. Dependencies must remain ecosystem-safe:
   - `extras` may depend on `fret-ui-shadcn` internal modules, `fret-ui-kit`, `fret-ui-headless`,
     and other ecosystem crates that stay above runtime/platform boundaries.
   - If an extras component requires a “heavy” dependency or platform integration, it must be
     feature-gated or moved into a dedicated ecosystem crate.

## Validation & regression gates

Every extras component must ship with at least one stable regression gate:

- **Snapshot test** (preferred for early iterations): add/extend
  `ecosystem/fret-ui-shadcn/tests/snapshots.rs` snapshots under
  `ecosystem/fret-ui-shadcn/tests/snapshots/*.json`.
- **Scripted interaction test** (when behavior is stateful and hard to snapshot): add a
  `fretboard diag` script and gate it in the appropriate suite (see
  `docs/ui-diagnostics-and-scripted-tests.md`).

Optional:

- If there is a web reference golden (rare for non-v4 blocks), add a targeted “web vs fret” gate.

## Component template (recommended baseline)

Use this template to keep extras consistent and reviewable:

1. File layout:
   - Implementation: `ecosystem/fret-ui-shadcn/src/extras/<component>.rs`
   - Module entry: `ecosystem/fret-ui-shadcn/src/extras/mod.rs`
2. Public surface:
   - Expose under `fret_ui_shadcn::extras::<component>::...`
   - Re-export from `extras/mod.rs` for convenience
   - Do not re-export from the crate root (`fret_ui_shadcn::*`)
3. State (Radix-style controlled vs uncontrolled):
   - Prefer `fret_ui_kit::declarative::controllable_state::use_controllable_model`
   - Controlled = caller provides `Model<T>`
   - Uncontrolled = element-local model initialized once from `default_*`
4. Semantics + test hooks:
   - Assign stable roles/labels where it improves automation and accessibility outcomes
   - Add `test_id` for key subparts (root, triggers, close buttons, etc.) when stateful
5. Gates (minimum bar):
   - Snapshot: `ecosystem/fret-ui-shadcn/tests/snapshots.rs` + a new/updated
     `ecosystem/fret-ui-shadcn/tests/snapshots/*.json`
   - Scripted diag (when needed): add a `tools/diag-scripts/*.json` script and gate it via the
     appropriate suite (see `docs/ui-diagnostics-and-scripted-tests.md`)
6. Attribution:
   - Add a short rustdoc note with upstream inspiration + license, and record the source in the
     `docs/workstreams/shadcn-extras-todo.md` “sources table” if/when created.

## Component selection criteria (what we add first)

We prioritize components that are:

- common in general apps,
- low on platform coupling,
- composable (small parts, not monolithic widgets),
- easy to validate with snapshots and/or deterministic scripts,
- and do not duplicate existing shadcn v4 surfaces already present in `fret-ui-shadcn`.

## Staged roadmap (candidate list)

Milestones are tracked in `docs/workstreams/shadcn-extras-milestones.md`.
Executable TODOs live in `docs/workstreams/shadcn-extras-todo.md`.

### M0: Skeleton + conventions

- Create `extras` module skeleton.
- Add a minimal snapshot harness page for extras.
- Add a component template (docs-only) that standardizes:
  - naming and exports,
  - controlled vs uncontrolled model patterns,
  - `test_id` conventions,
  - and required gates.

### M1: Low-risk composition blocks (recommended first set)

Inspired by `repo-ref/kibo` (MIT), adapted to Fret primitives:

- `Banner` (dismissible row + optional action)
- `Announcement` (badge-like, composable header chip)
- `Tags` (static tag list / chips; editable tag input is a later milestone)
- `Rating` (radiogroup-like star rating; keyboard-first)
- `RelativeTime` (display-only; avoid timers at first)

### M2: Medium complexity (adds more interaction policy)

- `AvatarStack` (stacked avatars; implement with clipping/overlap rather than web-only mask tricks)
- `Snippet` / `CodeBlock` (decision: owned outside extras; see “Ownership decision” below)

### M3: Scheduling/animation-heavy blocks (defer until authoring patterns are stable)

- `RelativeTime` auto-updating modes (continuous frames today; switch to timers when available)
- `Marquee` / `Ticker` style components (continuous frames lease + perf gates)

## Notes on upstream sources and licensing

We only port outcomes from permissive sources:

- Prefer MIT / Apache-2.0 / BSD-style licenses.
- Avoid strong copyleft sources (GPL/AGPL) for direct code reuse.
- For each extras component, record its upstream inspiration in rustdoc (short) and in a small
  “sources table” in `docs/workstreams/shadcn-extras-todo.md`.

## Ownership decision: `Snippet` / `CodeBlock` (not in extras)

Decision:

- `Snippet` / `CodeBlock` do **not** land under `fret_ui_shadcn::extras`.

Rationale (keep surfaces aligned with layering):

- We already have a dedicated ecosystem for code rendering:
  - `ecosystem/fret-code-view` provides the core “code block” UI surface.
  - `ecosystem/fret-markdown` renders fenced code blocks via `fret-code-view`.
- The “Snippet / CodeBlock” product surface is usually policy-heavy:
  expand/collapse, copy actions, header slots, language labels, attachments, inline diffs, and
  interaction scripts. These policies are better owned by:
  - `ecosystem/fret-ui-ai` for AI/chat-driven “message parts” (see `docs/workstreams/ai-elements-port.md`), and
  - `ecosystem/fret-markdown` for document rendering policies.
- Keeping the implementation in `fret-code-view` avoids duplicating engines and reduces the risk of
  accidental runtime-contract creep (ADR 0066).

What extras may do later (optional, and only as a thin recipe):

- If we need a “shadcn-feeling wrapper” around `fret-code-view`, introduce a differently named block
  (e.g. `CodeSnippetCard`) that composes `fret_code_view::CodeBlock` without inventing a second
  `CodeBlock` type under `fret-ui-shadcn`.

Regression gates (where they belong):

- `fret-code-view`: keep unit/integration tests for wrapping/selection/scroll policies.
- `fret-ui-ai`: gate expand/collapse and message-part behaviors via deterministic `fretboard diag`
  scripts (e.g. the existing UI gallery scripts referenced in the AI Elements workstream).
