# shadcn/ui v4 Audit - Input

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Input` against the upstream shadcn/ui v4 base docs,
base examples, and the in-repo web goldens that currently gate input chrome.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/input.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/input.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/input-basic.tsx`, `repo-ref/ui/apps/v4/examples/base/input-field.tsx`, `repo-ref/ui/apps/v4/examples/base/input-fieldgroup.tsx`, `repo-ref/ui/apps/v4/examples/base/input-disabled.tsx`, `repo-ref/ui/apps/v4/examples/base/input-invalid.tsx`, `repo-ref/ui/apps/v4/examples/base/input-file.tsx`, `repo-ref/ui/apps/v4/examples/base/input-inline.tsx`, `repo-ref/ui/apps/v4/examples/base/input-grid.tsx`, `repo-ref/ui/apps/v4/examples/base/input-required.tsx`, `repo-ref/ui/apps/v4/examples/base/input-badge.tsx`, `repo-ref/ui/apps/v4/examples/base/input-input-group.tsx`, `repo-ref/ui/apps/v4/examples/base/input-button-group.tsx`, `repo-ref/ui/apps/v4/examples/base/input-form.tsx`, `repo-ref/ui/apps/v4/examples/base/input-rtl.tsx`
- Existing chrome gates: `goldens/shadcn-web/v4/new-york-v4/input-demo.json`, `goldens/shadcn-web/v4/new-york-v4/input-demo.invalid.json`, `goldens/shadcn-web/v4/new-york-v4/input-demo.focus.json`, `goldens/shadcn-web/v4/new-york-v4/input-demo.invalid-focus.json`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/input.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/input.rs`

## Audit checklist

### Authoring surface

- Pass: `Input::new(model)` plus `placeholder(...)`, `disabled(...)`, `aria_invalid(...)`, and `obscure_text(...)` covers the documented core input surface.
- Pass: `control_id(...)` is the right Fret hook for label/description association; `FieldLabel::for_control(...)` and `FieldDescription::for_control(...)` cover the focused association story without widening `Input` itself.
- Pass: no extra generic children / compose API is needed for `Input`; upstream composition happens around the control via `Field`, `InputGroup`, and `ButtonGroup`, and Fret already matches that layering.

### Layout & default-style ownership

- Pass: root `w-full min-w-0` plus control height remain recipe-owned because the upstream input source defines width and intrinsic control chrome on the component itself.
- Pass: surrounding width caps such as `max-w-xs` / `max-w-sm`, grid placement, and form-row layout remain caller-owned and stay on the gallery/example compositions.
- Pass: `aria_invalid` border/ring outcomes are already covered by existing web chrome gates; no new mechanism gap was found in this pass.
- Note: native file inputs stay a composed `Input` + `Browse` button pattern in Fret rather than mirroring DOM `type="file"` directly.
- Note: required visuals remain label/call-site composition, matching the upstream examples where the star is authored outside the input recipe itself.

### Gallery / docs parity

- Pass: the gallery mirrors the upstream base Input docs path first: `Usage`, `Basic`, `Field`, `Field Group`, `Disabled`, `Invalid`, `File`, `Inline`, `Grid`, `Required`, `Badge`, `Input Group`, `Button Group`, `Form`, and `RTL`.
- Pass: `Usage` now has a dedicated snippet source so the copyable code tab stays tied to compiled sample code instead of an inline page string.
- Pass: `Label Association`, `API Reference`, and `Notes` remain explicit Fret follow-ups after the upstream path because they document the `control_id(...)` bridge, ownership notes, and diagnostics guidance rather than upstream section headings.
- Pass: each Input docs section now exposes a page-scoped stable `test_id` prefix (`ui-gallery-input-*`), which lets geometry and screenshot gates target the real docs structure instead of only snippet-local nodes.
- Pass: this work is docs/public-surface parity and diagnostics-surface alignment, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- `cargo test -p fret-ui-gallery --lib gallery_input_core_examples_keep_upstream_aligned_targets_present`
- `cargo test -p fret-ui-gallery --lib notes_sections_keep_stable_height_while_scrolling_into_view`
- Existing chrome gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`input-demo`, `input-demo.invalid`, `input-demo.focus`, `input-demo.invalid-focus`)
- Existing layout gate coverage remains referenced from `docs/audits/shadcn-input.md` history and input-related web-vs-fret tests.
- Existing diag scripts: `tools/diag-scripts/ui-gallery/input/ui-gallery-input-label-click-focus.json`, `tools/diag-scripts/ui-gallery/input/ui-gallery-input-file-browse-mocked.json`, `tools/diag-scripts/ui-gallery/input/ui-gallery-input-docs-screenshots.json`
