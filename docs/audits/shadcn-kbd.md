# shadcn/ui v4 Audit - Kbd

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Kbd` against the upstream shadcn/ui v4 base docs,
base examples, and the existing in-repo chrome gate.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/kbd.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/kbd.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/kbd-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/kbd-group.tsx`, `repo-ref/ui/apps/v4/examples/base/kbd-button.tsx`, `repo-ref/ui/apps/v4/examples/base/kbd-tooltip.tsx`, `repo-ref/ui/apps/v4/examples/base/kbd-input-group.tsx`, `repo-ref/ui/apps/v4/examples/base/kbd-rtl.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/kbd.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/kbd.rs`

## Audit checklist

### Authoring surface

- Pass: `Kbd::new(text)` covers the default textual key token authoring path.
- Pass: `Kbd::from_children([...])` covers icon-based tokens used by the demo and RTL examples.
- Pass: `KbdGroup::new([...])` covers the documented grouped shortcut surface without widening the recipe unnecessarily.
- Pass: no extra generic composition API is needed; upstream compositions place `Kbd` inside buttons, tooltips, and input groups, and Fret already matches that layering.

### Layout & default-style ownership

- Pass: the fixed-height keycap chrome and text centering stay recipe-owned.
- Pass: surrounding button padding, tooltip content layout, and input-group placement stay caller-owned composition concerns.
- Pass: no mechanism gap was identified in this pass; current work is docs/public-surface parity only.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream docs path first: `Demo`, `Usage`, `Group`, `Button`, `Tooltip`, `Input Group`, `RTL`, and `API Reference`.
- Pass: the old `Notes` section is replaced by an explicit `API Reference` section that records the public surface and ownership decisions.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- Existing chrome gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`kbd-demo`)
