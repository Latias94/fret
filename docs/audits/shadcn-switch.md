# shadcn/ui v4 Audit - Switch

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Switch` against the upstream shadcn/ui v4 base docs,
base examples, and the existing switch web gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/switch.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/switch.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/switch-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/switch-description.tsx`, `repo-ref/ui/apps/v4/examples/base/switch-choice-card.tsx`, `repo-ref/ui/apps/v4/examples/base/switch-disabled.tsx`, `repo-ref/ui/apps/v4/examples/base/switch-invalid.tsx`, `repo-ref/ui/apps/v4/examples/base/switch-sizes.tsx`, `repo-ref/ui/apps/v4/examples/base/switch-rtl.tsx`
- Existing chrome gates: `goldens/shadcn-web/v4/new-york-v4/switch-demo.json`, `goldens/shadcn-web/v4/new-york-v4/switch-demo.focus.json`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/switch.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/switch.rs`

## Audit checklist

### Authoring surface

- Pass: `Switch::new(model)` plus `size(...)`, `disabled(...)`, `aria_invalid(...)`, `control_id(...)`, and `a11y_label(...)` covers the documented control-level surface.
- Pass: `Switch::from_checked(...)` and `action(...)` / `action_payload(...)` remain available for action-first authoring without forcing a `Model<bool>` at every call site.
- Pass: `FieldLabel::for_control(...)` plus `FieldLabel::wrap(...)` covers the upstream description and choice-card compositions without widening `Switch` into a generic children API.
- Pass: `Switch` remains a leaf control; no extra generic `compose()` / `asChild` surface is needed here.

### Layout & default-style ownership

- Pass: track/thumb chrome, focus ring, and intrinsic switch sizes remain recipe-owned because the upstream switch source defines those defaults on the component itself.
- Pass: surrounding width caps such as `max-w-sm`, field-group stacking, and page/grid negotiation remain caller-owned and stay on gallery/example compositions.
- Pass: checked-track and focus outcomes remain covered by the existing switch web chrome gates, and this pass also closes a small public-surface gap by adding `aria_invalid(...)` directly on `Switch` rather than forcing callers through manual style overrides.
- Note: `SwitchStyle` remains a focused Fret follow-up for token-safe color overrides rather than part of the upstream docs path.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream base Switch docs path first: `Demo`, `Usage`, `Description`, `Choice Card`, `Disabled`, `Invalid`, `Size`, and `RTL`.
- Pass: `Label Association` and `Style Override` remain explicit Fret follow-ups after the upstream path because they document Fret-specific control-registry and styling escape hatches.
- Pass: this work is mostly docs/public-surface parity, with one small source-alignment fix: `Switch` now exposes `aria_invalid(...)` directly.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- Existing chrome + focus gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs` (`switch-demo`, `switch-demo.focus`)
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (`switch-demo`)
