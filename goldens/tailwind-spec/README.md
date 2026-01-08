# tailwind-spec goldens

This folder stores JSON goldens for a **Tailwind class parser + style computation** contract.

Goal: validate that Fret can interpret a subset of Tailwind/shadcn class strings (without a web
runtime) into a stable, typed set of style tokens (spacing/sizing/radius/etc.).

Layout:

- `goldens/tailwind-spec/v1/*.json`: per-case inputs + expected normalized tokens.

