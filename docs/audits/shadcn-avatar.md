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

- shadcn docs: `repo-ref/ui/apps/v4/content/docs/components/base/avatar.mdx`
- shadcn dropdown docs/examples: `repo-ref/ui/apps/v4/content/docs/components/base/dropdown-menu.mdx`
- shadcn/base source: `repo-ref/ui/apps/v4/examples/base/ui/avatar.tsx`
- shadcn/base examples: `repo-ref/ui/apps/v4/examples/base/avatar-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/avatar-basic.tsx`, `repo-ref/ui/apps/v4/examples/base/avatar-badge.tsx`, `repo-ref/ui/apps/v4/examples/base/avatar-badge-icon.tsx`, `repo-ref/ui/apps/v4/examples/base/avatar-group.tsx`, `repo-ref/ui/apps/v4/examples/base/avatar-group-count.tsx`, `repo-ref/ui/apps/v4/examples/base/avatar-group-count-icon.tsx`, `repo-ref/ui/apps/v4/examples/base/avatar-size.tsx`, `repo-ref/ui/apps/v4/examples/base/avatar-dropdown.tsx`, `repo-ref/ui/apps/v4/examples/base/avatar-rtl.tsx`
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

- Pass: the gallery page now mirrors the upstream docs path much more explicitly: `Demo`, `Usage`, `Basic`, `Badge`, `Badge with Icon`, `Avatar Group`, `Avatar Group Count`, `Avatar Group with Icon`, `Sizes`, `Dropdown`, `RTL`, and `API Reference`, before a Fret-only fallback check.
- Pass: the `Usage` snippet is copyable and complete enough for authors to lift directly.
- Pass: avatar-in-dropdown demos are exposed with stable `test_id` anchors for diagnostics.

### Gallery / docs parity

- Pass: the docs-aligned `Demo` now matches the upstream outcome more closely: basic avatar, badge avatar, and avatar group with count.
- Pass: `Badge with Icon` and `Avatar Group with Icon` are now dedicated gallery sections instead of being folded into neighboring examples, which keeps the page source-comparable against the upstream docs headings.
- Pass: `Fallback only` remains explicitly after the upstream path as a Fret-specific regression surface.

### Avatar authoring surface

- Pass: `Avatar::children(...)` now exists, so avatar content can be authored in a composable way
  without forcing only the convenience constructor path.
- Pass: `Avatar::empty().children([..])` now gives the docs-facing builder lane a natural entry
  point instead of forcing placeholder empty iterators in copyable snippets.
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

- Pass: `focus(trigger-avatar child button)`
- Pass: `activate(trigger-avatar child button) -> open`
- Pass: `click_stable(trigger-avatar child button) -> open`
- Pass: `click_stable open -> Escape -> focus restore`
- Pass: suite `ui-gallery-avatar-dropdown-attribution`

## Root-cause classification

### Not a visual-defaults problem

- The avatar parity work here was not blocked by size, radius, fallback paint, spacing, or docs-page
  layout defaults.
- After aligning the trigger composition, the visual result and interactive result both match the
  intended shadcn docs pattern closely enough for gallery/demo use.

### Not a pointer hit-test infrastructure failure

- Pointer-open and Escape focus restore were already behaving correctly before the final recipe fix.
- That ruled out a primary hit-test or overlay dismissal infrastructure bug.

### Diagnostic infrastructure gap was real, and is already fixed

- Semantic `activate` initially failed for the wrapper trigger because diagnostics invoked synchronously
  under the `ElementRuntime` lease and fell back to a generic `Space` key path.
- That has already been fixed by:
  - routing button-like roles through `Enter` in accessibility invoke;
  - dispatching `activate` outside the `ElementRuntime` mutable lease.

### Final remaining issue was recipe composition, not avatar mechanism

- The failing target was the nested presentational avatar node, not the authored pressable trigger.
- In upstream shadcn/Radix composition, `DropdownMenuTrigger asChild` is meant to reuse the authored
  interactive child (typically a `Button`), while the nested `Avatar` remains presentational content.
- Our previous gallery snippet put the diagnostic child `test_id` on the nested `Avatar`, which made
  pointer click look fine but semantic `focus` / `activate` target the wrong node.
- The fix was to align the demo with the upstream composition shape:
  - use `DropdownMenu::into_element_parts(...)`;
  - make the authored ghost icon `Button` the actual trigger surface;
  - move the trigger contract `test_id` to that button.

## Conclusion

- Result: `Avatar` itself was not the problem.
- Result: this was not a missing semantic-defaults fix in `fret-ui`, nor a hit-test bug in the renderer.
- Result: this was not a missing mechanism-layer children API issue. The useful follow-up was
  authoring-surface polish: keep `Avatar::new([..])` for direct construction, and expose
  `Avatar::empty().children([..])` for the docs-aligned builder lane.
- Result: Gallery media sourcing also belonged to the teaching surface, not the avatar mechanism:
  the snippets now use the shared UI Gallery demo asset bundle instead of inline RGBA generation.
- Result: the correct parity move for this component is **recipe alignment**:
  - the authored pressable child owns trigger semantics;
  - the nested avatar stays presentational.
- Result: after that alignment, the full avatar dropdown diag matrix now passes.

## Recommended next steps

### Short term

- Keep the avatar gallery dropdown snippet authored in the shadcn/Radix shape: authored ghost button
  as trigger, avatar nested inside it.
- Keep automation targeting the trigger button `test_id`, not the nested avatar leaf.

### Follow-up implementation work

- If you want stronger ergonomics across the ecosystem, consider adding an explicit
  `DropdownMenuTrigger::as_child(true)` authoring signal later. That would be an API ergonomics
  improvement, not a blocker for avatar parity anymore.
- Reuse this attribution pattern for other shadcn trigger recipes: first verify whether diagnostics are
  targeting the authored interactive child or only a nested presentational leaf.

## Validation

- `cargo build -p fret-ui-gallery`
- `cargo test -p fret-diag-protocol script_v2_roundtrip_ui_gallery_avatar_dropdown_activate_open -- --exact`
- `cargo test -p fret-diag-protocol script_v2_roundtrip_ui_gallery_avatar_dropdown_focus_trigger -- --exact`
- `cargo test -p fret-diag-protocol script_v2_roundtrip_ui_gallery_avatar_dropdown_activate_open_trigger -- --exact`
- `target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-activate-open.json --dir target/fret-diag-avatar-activate-open-main2 --session-auto --timeout-ms 900000 --launch -- target/debug/fret-ui-gallery.exe`
- `target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-focus-trigger.json --dir target/fret-diag-avatar-focus-main2 --session-auto --timeout-ms 900000 --launch -- target/debug/fret-ui-gallery.exe`
- `target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-escape-focus-restore.json --dir target/fret-diag-avatar-escape-main2 --session-auto --timeout-ms 900000 --launch -- target/debug/fret-ui-gallery.exe`
- `target/debug/fretboard.exe diag run tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-click-stable-open.json --dir target/fret-diag-avatar-click-main2 --session-auto --timeout-ms 900000 --launch -- target/debug/fret-ui-gallery.exe`
- `target/debug/fretboard.exe diag suite ui-gallery-avatar-dropdown-attribution --dir target/fret-diag-avatar-attribution-main4 --session-auto --timeout-ms 900000 --launch -- target/debug/fret-ui-gallery.exe`
