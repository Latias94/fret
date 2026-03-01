# UI Gallery fearless refactor (v1)

## Goal

Make UI Gallery a reliable “component gallery + documentation” surface where **preview ≡ code**:

- The preview you see is compiled from a snippet module.
- The code tab shows the exact snippet source (via `include_str!`), optionally sliced by `// region:` markers.
- Pages become easy to audit for shadcn parity (layout constraints, tokens, interaction semantics), and easy to refactor
  without silently drifting the docs.

This workstream is intentionally scoped to **UI Gallery authoring + documentation plumbing**. Component parity work
still lands in the proper layer:

- `crates/fret-ui`: mechanism/contracts only.
- `ecosystem/fret-ui-kit`: headless policy + infra.
- `ecosystem/fret-ui-shadcn`: shadcn v4 recipes + composition defaults.

## Non-goals

- No “CSS/Tailwind emulation” in `fret-ui`.
- No attempt to make the gallery a full website; it remains an app surface with stable `test_id` selectors for diag.

## Source of truth

- shadcn docs content: `repo-ref/ui/apps/v4/content/docs/components/*.mdx`
- shadcn recipes (component composition): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/*.tsx`
- overlay/interaction semantics: `repo-ref/primitives` (Radix) + APG references
- headless patterns (parts/slots): `repo-ref/base-ui` when DOM-specific assumptions appear

## Current approach (preview ≡ code)

1. Put each example in a snippet module under `apps/fret-ui-gallery/src/ui/snippets/**`.
2. The page renders the preview by calling `snippets::<example>::render(cx)`.
3. The page’s code tab uses `DocSection::code_from_file_region("rust", include_str!(...), "example")`.
4. Snippet files wrap the copyable region with:

```rust
// region: example
// ...code...
// endregion: example
```

## Why this matters

Without a single source of truth, UI Gallery tends to drift in 3 ways:

1. The preview is “fixed” but the code snippet is not (copy/paste breaks).
2. The code snippet is “simplified” but the preview requires extra layout constraints.
3. Multiple pages copy the same recipe and diverge (tokens + semantics become inconsistent).

This refactor makes drift mechanically harder.

