# Fearless Refactor: User-Facing Authoring APIs (Draft)

Date: 2026-02-09  
Audience: app authors and ecosystem authors  
Goal: improve day-to-day authoring ergonomics without collapsing kernel contracts or forcing a flag day.

This document is intentionally "user-facing" in the sense that it focuses on what authors write and
what stability guarantees we can make while evolving the API surface.

## Guardrails (Non-Negotiables)

1) Kernel vs ecosystem separation stays intact.

- `fret-ui` remains mechanism/contract-oriented (tree, layout, routing, paint orchestration).
- Policy-heavy interaction design and component taxonomy remains in ecosystem crates (`fret-ui-kit`,
  `fret-ui-shadcn`, and higher-level kits).

2) One golden path, optional escape hatches.

- The default path should be obvious, well documented, and used by templates/demos.
- Alternative surfaces can exist, but must be explicitly labeled as advanced/opt-in.

3) No silent contract collapse when embedding other UI.

- Foreign UI mixing remains isolated surfaces only (see ADR 0189 and embedded viewport surfaces).

## Current Golden Path (As Of Today)

- View wiring: `fn(&mut ElementContext<'_, App>, &mut State) -> Elements`
- Element identity: `ElementContext::{scope,keyed,named}` + stable `GlobalElementId`
- Policy surface: `UiExt::ui()` / `UiBuilder` in `fret-ui-kit`, shadcn taxonomy in `fret-ui-shadcn`

Rationale:

- `fn` pointer wiring keeps hotpatch boundaries predictable and avoids closure capture concerns.
- `Elements` as a concrete return type keeps the view signature nameable in a `type ViewFn = ...` alias.

## Roadmap Structure

We classify changes into:

- "Small wins": incremental improvements that are low-risk and do not require a large migration.
- "Fearless refactors": larger shifts that may require an opt-in driver, a branch/worktree, and explicit migration guidance.

## Small Wins (Recommended Next Work)

### SW1) Reduce authoring boilerplate that has no semantic value

- Prefer single-call helpers for extremely common, contract-safe patterns.
- Example: `AnyElement::test_id(...)` (diagnostics/automation only).

Success criteria:

- common code paths in demos shrink without introducing new conceptual machinery.
- layering boundaries are preserved (no "utility traits" leaking across layers).

### SW2) Make iterator-friendly children the norm

Policy for ecosystem components:

- accept `children: impl IntoIterator<Item = AnyElement>` at public boundaries,
- store `Vec<AnyElement>` internally,
- avoid forcing callers into `vec![...]` unless they truly need ownership.

Success criteria:

- templates and shadcn recipes can compose children using arrays/iterators without `collect()`.

### SW3) Converge examples/templates onto `UiBuilder` composition

Goal:

- reduce "two dialect" confusion by using `UiExt::ui()` / `UiBuilder` as the default authoring surface in
  examples and templates.

Success criteria:

- a new user can follow a single consistent style across all demos and docs.

### SW4) Document the "why" (and the tradeoffs) once

Keep short, authoritative docs:

- `docs/audits/api-ergonomics-audit.md` summarizes the surface tradeoffs.
- prefer linking to specific evidence paths rather than re-explaining concepts in many places.

## Fearless Refactors (Big Rocks)

These are intentionally deferred until the small wins land, because they change authoring feel and
require coordination across kernel + ecosystem.

### FR1) Optional captured-closure view driver (opt-in)

Add a non-default driver that accepts captured closures:

- `Box<dyn for<'a> FnMut(&mut ElementContext<'a, App>, &mut State) -> Elements + 'static>`

Why:

- enables some app architectures (dependency injection, composition, test doubles) without forcing the kernel to carry new generics.

Risks:

- dynamic dispatch + allocation,
- hotpatch complexity,
- potential performance regressions if abused.

### FR2) Commit to a single declarative tree story

Today: there is a retained prototype (`UiTree`) plus the declarative element tree direction.

Fearless refactor goal:

- converge toward a single "per-frame rebuild + externalized state" authoring story,
- keep stability via shims, and remove the need for "compat retained" surfaces over time.

### FR3) High-density authoring layer (optional macro DSL)

Consider an ecosystem-level macro DSL (not in kernel) for high-density UI authoring, inspired by
RSX-like patterns in other frameworks.

Constraints:

- must preserve stable identity and explicit invalidation concepts,
- must not hide the kernel/contract boundary in a way that makes debugging impossible.

## Migration Strategy (How We Avoid Breaking Users)

- Prefer additive APIs first (helpers, iterator-friendly constructors).
- Deprecate only after templates/examples no longer teach the old way.
- Provide a "compat adapter" layer for ecosystem code when a signature has to change.
- Use `fretboard diag` scripts to lock behavior and prevent regressions during refactors.

## When To Use a Worktree/Branch

Any change that:

- rewires the default app driver API,
- changes view function calling conventions,
- changes identity rules (`GlobalElementId` generation),
- or requires cross-crate coordinated migrations,

should be developed in a separate worktree + branch under `F:\\SourceCodes\\Rust\\fret-worktrees\\`.

## Related Docs

- `docs/audits/api-ergonomics-audit.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`

