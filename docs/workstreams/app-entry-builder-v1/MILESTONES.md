# App Entry Builder v1 (Milestones)

## M0 — Design approved

**Exit criteria**

- This folder contains an agreed design and TODO list.
- Naming and minimal API surface are agreed.
- Feature/preset strategy is agreed (at least for `desktop`, `batteries`, `config-files`).

## M1 — Builder prototype (no template switch)

**Scope**

- Add builder types in `ecosystem/fret` only.
- Implement MVU and UI entry variants that wrap existing `fret-bootstrap` wiring.
- Preserve existing entry points (no breaking changes).

**Exit criteria**

- `cargo check -p fret` passes (defaults and minimal features).
- A small doc example compiles using the new builder chain.

## M2 — Onboarding switch (templates + docs)

**Scope**

- Update `fretboard new hello/simple-todo/todo` templates to use the builder chain.
- Update onboarding docs to match.

**Exit criteria**

- New templates compile and run in the repo’s demo shells.
- Docs are consistent and do not mention internal runner types in the “first hour” path.

## M3 — Ecosystem extension polish

**Scope**

- Add the most important extension seams to the builder surface (without bloat), e.g.:
  - icon pack selection convenience
  - ui-assets budgets convenience
  - router/workspace-shell opt-ins (feature gated)

**Exit criteria**

- A “golden path + one extension” example exists (e.g. router commands or workspace shell).
- Users can reach advanced wiring via `install_*` hooks without leaving the `fret` surface.

## M4 — Optional closure entry (if chosen)

**Scope**

- Introduce closure-based entry behind explicit opt-in.
- Document hotpatch tradeoffs clearly.

**Exit criteria**

- Closure entry exists without impacting the default fn-pointer entry.
- Documentation and type naming make the tradeoffs unambiguous.

