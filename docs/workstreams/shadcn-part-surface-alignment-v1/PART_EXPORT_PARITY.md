# shadcn Part Export Parity (radix base → fret-ui-shadcn)

Snapshot date: **2026-03-02**.

This document is a **mechanical** parity check: it compares the identifiers exported from
`repo-ref/ui/apps/v4/registry/bases/radix/ui/*.tsx` against the identifiers visible in
`ecosystem/fret-ui-shadcn/src/lib.rs`.

It does **not** validate behavior/visual parity. Use it to catch obvious “missing part” surfaces
that block copy/paste alignment with upstream docs.

## Results (missing exports)

Total upstream components scanned: **55**.  
Total missing exports detected: **4**.

| Component (upstream file) | Missing export(s) | Category | Notes / next action |
|---|---|---|---|
| `badge.tsx` | `badgeVariants` | style helper | We intentionally prefer typed enums (`BadgeVariant`) + the `Badge` recipe. Consider adding a compat helper only if copy/paste friction is high. |
| `button.tsx` | `buttonVariants` | style helper | Upstream uses this to style non-button nodes. If needed, map to a reusable `ChromeRefinement`/`LayoutRefinement` preset instead of a “class string” concept. |
| `button-group.tsx` | `buttonGroupVariants` | style helper | Same as above; likely not needed unless upstream snippets use it directly. |
| `toggle.tsx` | `toggleVariants` | style helper | We intentionally prefer typed enums (`ToggleVariant`, `ToggleSize`) + the `Toggle` recipe. |
