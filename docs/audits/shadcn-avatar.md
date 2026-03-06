# shadcn/ui v4 Audit - Avatar

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui
- Radix UI Primitives: https://github.com/radix-ui/primitives
- MUI Base UI: https://github.com/mui/base-ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Avatar` recipe and avatar-in-dropdown usage against
upstream shadcn/ui v4 docs plus Radix/Base UI interaction expectations.

## Upstream references (source of truth)

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/avatar.mdx`
- shadcn dropdown docs/examples: `repo-ref/ui/apps/v4/content/docs/components/dropdown-menu.mdx`
- shadcn source: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/avatar.tsx`
- Radix avatar primitive: `repo-ref/primitives/packages/react/avatar/src/avatar.tsx`
- Base UI interaction references: `repo-ref/base-ui/packages/react/src/menu`

## Fret implementation anchors

- Avatar component: `ecosystem/fret-ui-shadcn/src/avatar.rs`
- Dropdown trigger/menu recipe: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/avatar.rs`
- Avatar dropdown demo: `apps/fret-ui-gallery/src/ui/snippets/avatar/dropdown.rs`
- Usage snippet: `apps/fret-ui-gallery/src/ui/snippets/avatar/usage.rs`

## What is aligned now

### Docs/gallery surface

- Pass: the gallery page follows a `Demo -> Usage -> Extras` structure that matches the component-doc
  reading order more closely.
- Pass: the `Usage` snippet is copyable and complete enough for authors to lift directly.
- Pass: avatar-in-dropdown demos are exposed with stable `test_id` anchors for diagnostics.

### Avatar authoring surface

- Pass: `Avatar::children(...)` now exists, so avatar content can be authored in a composable way
  without forcing only the convenience constructor path.
- Pass: image/fallback authoring remains close to the upstream Radix/shadcn mental model.

### Pointer/open/restore behavior

- Pass: pointer-driven opening on the nested avatar target works (`click_stable` passes).
- Pass: open -> `Escape` -> focus restore works for the avatar dropdown demo.
- Pass: focusing the wrapper trigger works through diagnostics.

## Diagnostic matrix

The following scripted repros were used:

- `tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-focus-trigger.json`
- `tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-click-stable-open.json`
- `tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-escape-focus-restore.json`
- `tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-activate-open.json`
- `tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-activate-open-trigger.json`

Observed outcomes:

- Pass: `focus(trigger)`
- Pass: `click_stable(trigger-avatar child) -> open`
- Pass: `click_stable open -> Escape -> focus restore`
- Fail: `activate(trigger-avatar child)` -> `activate_invoke_unavailable`
- Fail: `activate(wrapper trigger)` -> `wait_until_timeout` after activation; the menu does not open

## Root-cause classification

### Not a visual-defaults problem

- The current drift is not about avatar size, radius, fallback paint, spacing, or docs-page layout.
- Pointer interaction and overlay dismissal are working, which rules out the most obvious visual/policy
  default issues.

### Not primarily a pointer hit-test infrastructure failure

- `click_stable` on the nested avatar child opens the menu successfully.
- `Escape` focus restore also succeeds after pointer-open.
- This means the target is hittable enough for pointer-driven interaction, and overlay/focus-restore
  plumbing is present.

### Gap 1: missing full composable-child (`asChild`) parity

- The nested avatar child (`ui-gallery-avatar-dropdown-trigger-avatar`) is selectable by `test_id`, but
  it is not semantically activatable.
- Current demo markup is visually "asChild-like" (avatar inside a ghost icon button), but semantics and
  activation still belong to the wrapper trigger button.
- This is a shadcn/Radix composition-surface gap rather than an avatar paint/layout gap.

### Diagnostic infra gap fixed: semantic activate now routes correctly for wrapper triggers

- The wrapper trigger (`ui-gallery-avatar-dropdown-trigger`) now opens successfully through diagnostic
  `activate`, matching the existing `focus + Enter` outcome.
- The root cause was diagnostics infrastructure: the semantic `activate` path was dispatching synchronously
  while `ElementRuntime` was leased, and it also relied on a generic `Space` fallback.
- The fix was twofold:
  - route button-like roles through `Enter` in accessibility invoke;
  - avoid running the `activate` step under the `ElementRuntime` mutable lease in the scripted diag engine.

## Conclusion

- Result: the remaining avatar issue is **not** a docs-only or default-tokens issue.
- Result: the remaining avatar issue is now concentrated in one layer:
  - **recipe/composition layer**: no full `asChild`-style child semantics forwarding yet.
- Result: wrapper-trigger semantic activation is now aligned for diagnostics; the unresolved parity gap is
  specifically that the nested authored avatar child does not own invoke semantics.
- Result: if the goal is true shadcn/Radix parity for avatar-as-trigger recipes, supporting composable
  children API is the next required step.

## Recommended next steps

### Short term

- Keep docs/demo wording precise: the current avatar dropdown demo is visually aligned with shadcn docs,
  but it does not yet claim full Radix `asChild` behavior parity.
- Use the wrapper trigger `test_id` as the current contract for diagnostics and tests that expect trigger
  semantics.

### Follow-up implementation work

- Add a true composable-child trigger path (Radix-style `asChild` parity) so the authored child can own
  the interaction/semantics surface when desired.
- Once that lands, add a conformance gate proving `activate(test_id=nested-child)` opens the menu just like
  pointer click and wrapper-trigger activate.

## Validation

- `cargo test -p fret-diag-protocol script_v2_roundtrip_ui_gallery_avatar_dropdown_click_stable_open -- --exact`
- `cargo test -p fret-diag-protocol script_v2_roundtrip_ui_gallery_avatar_dropdown_activate_open_trigger -- --exact`
- `target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-focus-trigger.json --dir target/fret-diag-avatar-focus --session-auto --timeout-ms 900000 --launch -- target/debug/fret-ui-gallery.exe`
- `target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-click-stable-open.json --dir target/fret-diag-avatar-click-stable --session-auto --timeout-ms 900000 --launch -- target/debug/fret-ui-gallery.exe`
- `target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-escape-focus-restore.json --dir target/fret-diag-avatar-escape --session-auto --timeout-ms 900000 --launch -- target/debug/fret-ui-gallery.exe`
- `target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-activate-open.json --dir target/fret-diag-avatar-attribution --session-auto --timeout-ms 900000 --launch -- target/debug/fret-ui-gallery.exe`
- `target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-activate-open-trigger.json --dir target/fret-diag-avatar-activate-trigger-fixed4 --session-auto --timeout-ms 900000 --launch -- target/debug/fret-ui-gallery.exe`
