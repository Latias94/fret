# shadcn/ui v4 Audit - Textarea

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Textarea` against the upstream shadcn/ui v4 base docs,
base examples, and the in-repo textarea web gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/textarea.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/textarea.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/textarea-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/textarea-field.tsx`, `repo-ref/ui/apps/v4/examples/base/textarea-disabled.tsx`, `repo-ref/ui/apps/v4/examples/base/textarea-invalid.tsx`, `repo-ref/ui/apps/v4/examples/base/textarea-button.tsx`, `repo-ref/ui/apps/v4/examples/base/textarea-rtl.tsx`
- Existing chrome gates: `goldens/shadcn-web/v4/new-york-v4/textarea-demo.json`, `goldens/shadcn-web/v4/new-york-v4/textarea-demo.invalid.json`, `goldens/shadcn-web/v4/new-york-v4/textarea-demo.focus.json`, `goldens/shadcn-web/v4/new-york-v4/textarea-demo.invalid-focus.json`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/textarea.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/textarea.rs`

## Audit checklist

### Authoring surface

- Pass: `Textarea::new(model)` covers the documented core authoring path.
- Pass: `placeholder(...)`, `disabled(...)`, `aria_invalid(...)`, `aria_required(...)`, `min_height(...)`, and `rows(...)` cover the practical control-level surface exposed by the upstream examples.
- Pass: `Field::build(...)` is the focused field-local association lane for docs-path `Field`/`RTL` examples; explicit `control_id(...)` plus `FieldLabel::for_control(...)` remains the follow-up click-to-focus path without widening `Textarea` itself.
- Pass: `Textarea` is a leaf text control, so Fret intentionally does not add a generic `compose()` builder here.

### Layout & default-style ownership

- Pass: root `w-full min-w-0`, control chrome, minimum height, and resize behavior remain recipe-owned.
- Pass: surrounding width caps such as `max-w-xs`, stacked button layout, and helper-text composition remain caller-owned.
- Pass: default minimum height matches the upstream `min-h-16` outcome (64px).
- Pass: `rows(...)` raises the initial minimum height when the caller wants a taller starting textarea, while preserving the recipe-owned 64px floor for default and one-row cases.
- Pass: `aria-invalid=true` border/ring outcomes already match the textarea web chrome gates.

### Semantics

- Pass: exposes `SemanticsRole::TextField` and supports explicit `a11y_label`.
- Pass: control registry integration supports label/described-by wiring via `control_id(...)`.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream base Textarea docs path first: `Demo`, `Usage`, `Field`, `Disabled`, `Invalid`, `Button`, `RTL`, and `API Reference`.
- Pass: the docs-path `Field` and `RTL` snippets now use the upstream feedback copy and a four-row initial composition rather than a hand-tuned fixed-height override.
- Pass: `With Text` and `Label Association` stay as focused Fret follow-ups after the upstream path.
- Pass: this work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-textarea cargo check -p fret-ui-gallery --message-format short`
- `CARGO_TARGET_DIR=target-codex-textarea-check cargo check -p fret-ui-shadcn --lib --tests --message-format short`
- Focused unit gate: `cargo test -p fret-ui-shadcn --lib textarea::tests::textarea_rows_raise_initial_min_height_without_lowering_defaults -- --exact`
- Docs-path diag gate: `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/textarea/ui-gallery-textarea-docs-screenshot.json --dir <out-dir> --session-auto --pack --ai-packet --timeout-ms 240000 --launch -- env CARGO_TARGET_DIR=target-codex-ui-gallery-textarea cargo run -p fret-ui-gallery`
- Existing chrome + focus gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`textarea-demo`, `textarea-demo.invalid`, `textarea-demo.focus`, `textarea-demo.invalid-focus`)
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`textarea-demo`)
