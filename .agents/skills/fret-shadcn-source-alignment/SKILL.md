---
name: fret-shadcn-source-alignment
description: Align Fret’s shadcn/Radix-inspired components with upstream sources (public docs + GitHub source; optional local pinned snapshots under `repo-ref/`), map changes to the correct Fret layer (`crates/fret-ui`, `ecosystem/fret-ui-kit`, `ecosystem/fret-ui-shadcn`), and add targeted tests + `fretboard diag` scripts to prevent regressions even when web goldens are incomplete.
---

# Shadcn / Radix source alignment

## Map the bug to the right layer

- `crates/fret-ui`: mechanisms/contracts (tree/layout/semantics/focus/overlay roots).
- `ecosystem/fret-ui-kit`: headless policy + reusable infra (roving focus, typeahead, overlay policy).
- `ecosystem/fret-ui-shadcn`: shadcn v4 taxonomy + recipes (composition + styling).

If the mismatch is “interaction policy” (dismiss rules, focus restore, hover intent, menu navigation),
it almost never belongs in `crates/fret-ui`.

## Find the upstream reference (source of truth)

1. Start with public docs (good enough for most alignment work):
   - shadcn components: https://ui.shadcn.com/docs/components
   - Radix primitives: https://www.radix-ui.com/primitives/docs/components
2. If you need exact implementation details, use source code:
   - shadcn/ui v4 source (New York v4 registry): https://github.com/shadcn-ui/ui/tree/main/apps/v4/registry/new-york-v4/ui
   - Radix Primitives source: https://github.com/radix-ui/primitives/tree/main/packages/react
   - Optional local pinned snapshots (if your checkout includes `repo-ref/`; not necessarily present on GitHub):
     - shadcn recipe: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/<component>.tsx`
     - Radix primitive: `repo-ref/primitives/packages/react/<primitive>/src/*`

Use shadcn to learn the *composition + Tailwind contracts* (sizes, spacing, tokens), and Radix to
learn the *interaction semantics* (focus, dismiss, keyboard nav, portal layering).

## Align + protect with tests (goldens are not enough)

1. Prefer fast, targeted Rust unit tests for invariants.
   - Add tests next to the component in `ecosystem/fret-ui-shadcn/src/<component>.rs` (`#[cfg(test)]`).
   - Assert relationships (centering, overlap, clamping, tab order, active descendant, etc.).
2. Add web-vs-fret parity checks when the mismatch is layout/style outcome.
   - Existing harness lives under `ecosystem/fret-ui-shadcn/tests/`.
   - Web goldens live under `goldens/shadcn-web/v4/new-york-v4/*.json`.
3. Add an interaction repro script when state machines are involved.
   - Create `tools/diag-scripts/<scenario>.json` and gate it via `fretboard diag run`.
   - Always add/keep stable `test_id` targets in the Fret UI so scripts survive refactors.

## When a golden is missing

1. Add a targeted invariant test first (so you stop bleeding regressions immediately).
2. If needed, generate the missing golden later:
   - Follow `docs/shadcn-web-goldens.md` (extraction from the upstream shadcn v4 app; local `repo-ref/ui` is optional).

## High-value regression targets (start here)

- Overlay families: `dropdown-menu`, `select`, `context-menu`, `tooltip`/`hover-card`, `dialog`/`sheet`, `navigation-menu`.
- Listbox-ish behavior: roving focus, typeahead, active-descendant semantics, scroll clamping in constrained viewports.
