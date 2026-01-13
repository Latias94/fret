# tailwind-spec goldens

This folder stores JSON goldens for a **Tailwind-like vocabulary conformance** contract.

Goal: validate that Fret's *typed* Tailwind-like primitives (`Space`, `Radius`, layout/chrome
refinements) stay aligned with the shadcn/Tailwind vocabulary.

Note:

- Fret does **not** ship a runtime CSS/Tailwind class parser (see `docs/tailwind-semantics-alignment.md`).
- These goldens are consumed by test/tooling code that may parse class-like strings to map upstream
  vocabulary into typed tokens for comparison.

Layout:

- `goldens/tailwind-spec/v1/*.json`: per-case inputs + expected normalized tokens.
