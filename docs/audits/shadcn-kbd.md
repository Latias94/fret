# shadcn/ui v4 Audit - Kbd

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Kbd` against the current upstream shadcn/ui v4 docs,
registry examples, and the in-repo layout/docs gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/kbd.mdx`
- Component implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/kbd.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/kbd-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/kbd-group.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/kbd-button.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/kbd-tooltip.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/kbd-input-group.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/kbd.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/kbd.rs`
- Gallery snippets: `apps/fret-ui-gallery/src/ui/snippets/kbd/*.rs`
- Gallery docs-surface gate: `apps/fret-ui-gallery/tests/kbd_docs_surface.rs`

## Audit checklist

### Authoring surface

- Pass: `Kbd::new(text)` covers the default textual key token authoring path.
- Pass: `Kbd::from_children([...])` covers explicit icon-only or mixed-content keycaps without widening the default teaching lane.
- Pass: `KbdGroup::new([...])` covers the documented grouped shortcut surface without widening the recipe unnecessarily.
- Pass: no extra generic composition API is needed; upstream compositions place `Kbd` inside buttons, tooltips, and input groups, and Fret already matches that layering.

### Layout & default-style ownership

- Pass: the fixed-height keycap chrome and text centering stay recipe-owned.
- Pass: surrounding button padding, tooltip content layout, and input-group placement stay caller-owned composition concerns.
- Pass: no mechanism gap was identified in this pass; current work is docs/public-surface parity only.

### Docs-surface findings

- Fixed: the gallery `Usage` section now uses a real snippet-backed single-key example (`Kbd::new("Ctrl")`) instead of an abbreviated page-local code string.
- Fixed: the lead `Demo`, `Button`, `Tooltip`, and `Input Group` snippets now prefer the upstream textual/glyph lane (`⌘`, `⇧`, `⌥`, `⌃`, `⏎`) before falling back to Fret-only icon escape hatches.
- Fixed: the page now exposes stable section-scoped `ui-gallery-kbd-*` ids for docs diagnostics (`Demo`, `Usage`, `Group`, `Button`, `Tooltip`, `Input Group`, `API Reference`, `RTL`), and the `ui-gallery-kbd-docs-smoke` diag gate now asserts those docs-path anchors end to end.

### Gallery / docs parity

- Pass: the gallery now mirrors the current upstream docs path first: `Demo`, `Usage`, `Group`, `Button`, `Tooltip`, `Input Group`, and `API Reference`.
- Pass: the `RTL` section is explicitly retained after `API Reference` as a Fret-only follow-up because the current upstream Kbd docs do not define an RTL section.
- Pass: the `Usage` section now matches the upstream single-key teaching lane, while grouped shortcuts stay in the dedicated `Group` example.
- Pass: the `API Reference` section records the public surface plus the explicit `from_children([...])` escape hatch without promoting a broader generic children API.
- Pass: `apps/fret-ui-gallery/tests/kbd_docs_surface.rs` locks the page order, copyable textual/glyph snippets, API Reference-before-RTL diagnostics order, and the raw-free docs lane.

## Validation

- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail kbd`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail kbd`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_kbd --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --test fret_kbd_tooltip_slot --status-level fail fret_kbd_in_tooltip_content_overrides_bg_and_fg`
- `cargo nextest run -p fret-ui-gallery --test kbd_docs_surface`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail kbd`
- `python -m json.tool tools/diag-scripts/ui-gallery/kbd/ui-gallery-kbd-docs-smoke.json | Out-Null`
