# Editor Ecosystem Token Inventory and Namespace Plan v1

Tracking doc: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/DESIGN.md`  
Related ADR: `docs/adr/0316-editor-ecosystem-token-namespaces-and-skinning-boundary-v1.md`

Status: Draft  
Last updated: 2026-03-14

## Purpose

This note closes the first documentation pass for:

- `EER-THEME-040` current token inventory,
- `EER-THEME-041` initial namespace plan,
- and the related seeding/aliasing decisions needed to keep `editor.*`, `workspace.*`, and
  docking-owned chrome from drifting across crates.

This is an ownership and migration note, not a new runtime contract.
ADR 0316 remains the normative boundary decision.

## Scope

This inventory covers the three crates that matter for editor ecosystem skinning ownership:

- `ecosystem/fret-ui-editor`
- `ecosystem/fret-workspace`
- `ecosystem/fret-docking`

It also records where stable seeding already exists today in adjacent crates such as
`ecosystem/fret-ui-shadcn`.

## Snapshot

| Crate | Current token posture | Key evidence |
| --- | --- | --- |
| `fret-ui-editor` | Has explicit `editor.*` token keys and an opt-in preset patch entrypoint. Still reads and seeds shared `component.*` and generic semantic palette keys for compatibility. | `ecosystem/fret-ui-editor/src/primitives/tokens.rs`, `ecosystem/fret-ui-editor/src/theme.rs`, `ecosystem/fret-ui-editor/src/primitives/chrome.rs` |
| `fret-workspace` | Has partial `workspace.*` reads for panes, shell frame/top bar/status bar, and tab strip. A small internal token resolver now centralizes shell fallback order, and shadcn adapter-side seeding now exists for the shell-level families. | `ecosystem/fret-workspace/src/theme_tokens.rs`, `ecosystem/fret-workspace/src/panes.rs`, `ecosystem/fret-workspace/src/tab_strip/theme.rs`, `ecosystem/fret-workspace/src/frame.rs`, `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs` |
| `fret-docking` | Owns docking-specific drag/drop chrome through `component.docking.*` reads. Tab chrome still mainly rides generic tokens. | `ecosystem/fret-docking/src/dock/paint.rs`, `ecosystem/fret-docking/src/dock/space.rs` |
| `fret-ui-shadcn` | Already seeds docking-owned `component.docking.*` keys and now seeds shell-level `workspace.*` families for shadcn new-york presets. The current shell seeding intentionally stops at `workspace.frame.*`, `workspace.top_bar.*`, `workspace.status_bar.*`, and `workspace.tabstrip.*`. | `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`, `docs/workstreams/theme-token-alignment-v1/todo.md` |

## Crate Inventory

### `ecosystem/fret-ui-editor`

| Category | Current families / keys | Notes |
| --- | --- | --- |
| Canonical editor-owned keys already modeled as constants | `editor.density.*`, `editor.numeric.*`, `editor.property.*`, `editor.popup.*`, `editor.checkbox.*`, `editor.enum_select.*`, `editor.axis.*`, `editor.vec.*`, `editor.color.*`, `editor.slider.*` | This is the clearest and healthiest namespace posture in the current editor ecosystem. The latest baseline cleanup also split `editor.property.group_border` from the outer panel frame and `editor.property.panel_header_*` from repeated section headers so nested inspector hierarchy can be tuned without one shared chrome weight for every level, and now also promotes shared popup surface background/border intent into `editor.popup.*` so dark editor popups stop inheriting a host theme's `popover` card. |
| Shared component compatibility keys still read | `component.text_field.*`, `component.checkbox.*`, `component.input.*`, `component.slider.*`, `component.card.*` | These are compatibility / reuse bridges, not editor-owned namespaces. |
| Generic semantic palette fallbacks still read | `background`, `foreground`, `muted`, `muted-foreground`, `accent`, `border`, `ring`, `primary`, `card`, `popover`, `destructive`, `secondary` | These are expected as safe fallbacks, but they should not be treated as editor ownership. |
| Seeding / preset entrypoint | `apply_editor_theme_preset_v1`, `EditorThemePresetV1::{Default, ImguiLikeDense}` | Today this is the only editor ecosystem crate with an explicit preset patch path. |

Important observation:

- `fret-ui-editor` already owns real `editor.*` keys,
- but the current preset still patches shared `component.text_field.*` keys and generic palette
  keys like `card`, `muted`, `border`, `foreground`, `accent`, and `ring`.
- The popup-surface cleanup now moves one more visible seam out of that host-palette dependency:
  shared popup shells can read `editor.popup.*` first instead of inheriting `popover`.

That is acceptable for a proof preset, but it is not the final clean ownership seam for reusable
editor skinning.

### `ecosystem/fret-workspace`

| Category | Current families / keys | Notes |
| --- | --- | --- |
| Workspace-owned reads already present | `workspace.frame.*`, `workspace.top_bar.*`, `workspace.status_bar.*`, `workspace.pane.*`, `workspace.tab.*`, canonical `workspace.tabstrip.*`, legacy compatibility `workspace.tab_strip.*` | Shell chrome and pane/tabstrip chrome now resolve through one crate-local ownership surface. |
| Generic semantic fallbacks still read | `background`, `foreground`, `muted`, `muted-foreground`, `accent`, `border`, `ring` | These remain the safe ADR 0270 fallback tier for shell and tabstrip chrome. |
| Token resolver surface | `theme_tokens.rs` | `WorkspaceFrame`, `WorkspaceTopBar`, `WorkspaceStatusBar`, and `WorkspaceTabStripTheme` now share one fallback order instead of scattering raw string lookups. |
| Seeding / preset entrypoint | none in the owner crate | Shell families are now seeded from `fret-ui-shadcn`, while `fret-workspace` itself still intentionally has no design-system preset module. |

Important observation:

- `WorkspaceFrame`, `WorkspaceTopBar`, and `WorkspaceStatusBar` now have real `workspace.*`
  override points,
- so apps can start restyling shell chrome independently while still falling back safely to the
  generic app theme.
- The remaining gap is no longer shell-level token reading.
  It is broader preset distribution beyond the current shadcn shell seeding, plus deciding whether
  any owner-local proof preset is still necessary.

### `ecosystem/fret-docking`

| Category | Current families / keys | Notes |
| --- | --- | --- |
| Docking-owned reads already present | `component.docking.drop_overlay.*`, `component.docking.tab_insert.*` | These are already real, targeted docking-specific chrome tokens. |
| Generic semantic fallbacks still read | `background`, `foreground`, `muted-foreground`, `accent`, `border`, `primary`, `card`, `popover` | Dock tab chrome and menus still lean heavily on shared palette tokens. |
| Seeding / preset entrypoint inside docking | none | Docking relies on app or design-system seeding. |
| Seeding path already proven elsewhere | `fret-ui-shadcn` seeds the `component.docking.*` families today | This matters for the adapter decision below. |

Important observation:

- `fret-docking` already has stable docking-specific families for drag/drop affordances,
- and those families should stay docking-owned,
- but generic tab chrome still needs visual alignment work via adapter aliasing or seeding.

## Collisions, Drift, and Gaps

| Issue | Current evidence | Why it matters | v1 plan |
| --- | --- | --- | --- |
| `workspace.tab_strip.*` vs `workspace.tabstrip.*` naming drift | `theme_tokens.rs` now resolves canonical `workspace.tabstrip.*` first and falls back to legacy `workspace.tab_strip.*` for compatibility. | Presets can migrate toward the canonical family without breaking older theme payloads immediately. | Keep `workspace.tabstrip.*` as the canonical family and retire legacy underscore spellings only after adapter/theme surfaces converge. |
| Workspace owner crate still has no local seeding entrypoint | `WorkspaceFrame`, `WorkspaceTopBar`, and `WorkspaceStatusBar` now read namespaced keys through `theme_tokens.rs`, while `fret-ui-shadcn` seeds shell families in `shadcn_themes.rs`. | Reading is no longer the blocker; the remaining question is where preset distribution should live. | Keep reader/fallback logic local to `fret-workspace`, and treat adapter-side seeding as the stable v1 path. |
| Editor preset still mutates shared component and palette keys | `fret-ui-editor/src/theme.rs` writes `component.text_field.*` and generic palette keys like `card`, `muted`, `border`, `accent`. | An editor preset can bleed into non-editor widgets in the same app. | Keep this path for proof/demo use, but move toward editor-owned alias families before calling the seeding surface stable. |
| Pane drop preview and docking drop overlay are visually adjacent but semantically different | `workspace.pane.drop_preview_*` in workspace and `component.docking.drop_overlay.*` in docking. | Without an ownership note, future contributors may merge or duplicate them incorrectly. | Keep pane-local split preview in `fret-workspace`; keep dock-graph-aware overlay and tab insert visuals in `fret-docking`. |
| Docking tab chrome and workspace tabstrip chrome can visually diverge | Workspace owns shell tabstrip reads; docking still mostly uses generic tab chrome. | Editor shells can feel inconsistent even when ownership is correct. | Align by preset aliasing / seeding in adapter crates, not by moving dock-aware chrome into `fret-workspace`. |
| Current shadcn shell seeding stops short of `workspace.tab.*` | `fret-ui-shadcn` now seeds `workspace.frame.*`, `workspace.top_bar.*`, `workspace.status_bar.*`, and `workspace.tabstrip.*`, but not `workspace.tab.*`. | This keeps the current closure small, but tab-item visuals can still drift until a stronger reason appears. | Keep `workspace.tab.*` out of v1 seeding until repeated pressure or a proof surface shows the need. |

## Initial Namespace Plan

| Canonical family | Owner | Status | Current read / seed posture | Notes |
| --- | --- | --- | --- | --- |
| `editor.density.*` | `fret-ui-editor` | Implemented | Read via `EditorTokenKeys`, seeded in `fret-ui-editor/src/theme.rs` | Stable for v1 use. |
| `editor.numeric.*` | `fret-ui-editor` | Implemented | Read via `EditorTokenKeys`, seeded in preset patch | Stable for v1 use. |
| `editor.property.*` | `fret-ui-editor` | Implemented | Read via `EditorTokenKeys`, seeded in preset patch | Stable for v1 use. The current set now includes separate outer-panel vs inner-group frame tuning (`editor.property.panel_border` and `editor.property.group_border`) plus a dedicated top-band pair (`editor.property.panel_header_bg` / `editor.property.panel_header_border`) above repeated group headers. |
| `editor.popup.*` | `fret-ui-editor` | Implemented | Shared popup surface background/border plus radius/shadow metrics now read via `EditorTokenKeys` and are seeded in the preset patch | Stable for v1 use. This is the current landing zone for assist/select/color popup chrome that should not depend on a host theme's `popover` tone or require per-control geometry tuning. |
| `editor.checkbox.*` | `fret-ui-editor` | Implemented | Read via `EditorTokenKeys`, seeded in preset patch | Stable for v1 use. |
| `editor.enum_select.*` | `fret-ui-editor` | Partial | Only `editor.enum_select.max_list_height` exists today | Good enough for current proof work. |
| `editor.axis.*` | `fret-ui-editor` | Implemented | Read via `VecEdit` axis color resolution | Stable for v1 use. |
| `editor.vec.*` | `fret-ui-editor` | Implemented | Read via `EditorTokenKeys`, seeded in preset patch | Stable for v1 use. |
| `editor.color.*` | `fret-ui-editor` | Partial | Swatch/popup metrics exist; broader color-edit chrome still falls back to shared component tokens | Expand only if repeated pressure appears. |
| `editor.slider.*` | `fret-ui-editor` | Implemented | Track/thumb metrics exist; color styling still falls back to shared component/palette keys | Acceptable for v1. |
| Shared `component.text_field.*`, `component.checkbox.*`, `component.slider.*`, `component.input.*` | shared component layer, not editor-owned | Implemented elsewhere | Read by editor widgets as compatibility fallback; some are also seeded by the editor proof preset | Do not promote these as editor ownership. |
| `workspace.frame.*` | `fret-workspace` | Partial | Read via `theme_tokens.rs` and consumed by `WorkspaceFrame` with generic fallback; shadcn new-york now seeds shell-level frame chrome in `fret-ui-shadcn` | Owner-local seeding is still intentionally absent. |
| `workspace.top_bar.*` | `fret-workspace` | Partial | Read via `theme_tokens.rs` and consumed by `WorkspaceTopBar` with generic fallback; shadcn new-york now seeds shell-level top-bar chrome in `fret-ui-shadcn` | Owner-local seeding is still intentionally absent. |
| `workspace.status_bar.*` | `fret-workspace` | Partial | Read via `theme_tokens.rs` and consumed by `WorkspaceStatusBar` with generic fallback; shadcn new-york now seeds shell-level status-bar chrome in `fret-ui-shadcn` | Owner-local seeding is still intentionally absent. |
| `workspace.pane.*` | `fret-workspace` | Partial | Active border, bg, radius, and drop-preview metrics/colors exist | Good starting point; continue here rather than inventing app-local pane families. |
| `workspace.tab.*` | `fret-workspace` | Partial | Active bg, dirty fg, hover bg, drop indicator, max width exist | Keep tab-item state under `workspace.tab.*`. |
| `workspace.tabstrip.*` | `fret-workspace` | Partial | Canonical reads now exist in `theme_tokens.rs`; compatibility reads keep legacy `workspace.tab_strip.*` alive during migration; shadcn new-york now seeds shell-level tabstrip chrome in `fret-ui-shadcn` | Keep canonical family in docs and new theme payloads. |
| `component.docking.drop_overlay.*` | `fret-docking` | Implemented | Read in docking, seeded in `fret-ui-shadcn` | Keep name and owner; no workspace alias needed. |
| `component.docking.tab_insert.*` | `fret-docking` | Implemented | Read in docking, seeded in `fret-ui-shadcn` | Keep name and owner; no workspace alias needed. |

## Seeding and Adapter Decision

### v1 decision

- Stable design-system seeding for editor/workspace/docking namespaces should live in adapter or
  recipe crates such as `fret-ui-shadcn`, not in `fret-ui-editor` or `fret-workspace`.
- `fret-ui-editor` may keep local proof-oriented preset modules while a surface is still being
  validated, but that is not a reverse dependency on shadcn/material and should remain optional.
- We do not need a dedicated `fret-ui-editor-shadcn` or `fret-workspace-shadcn` crate yet.
  The current evidence does not justify the extra surface area.

### Why this is the recommended v1 path

1. `fret-ui-shadcn` already proves the one-way seeding path for docking-owned chrome through
   `component.docking.*`.
2. `fret-workspace` does not yet have a stable enough token reader surface to justify a new adapter
   crate.
3. A dedicated adapter crate before namespace cleanup would lock in current naming drift too early.

### Update after the first shell seeding slice

- `fret-ui-shadcn` now seeds `workspace.frame.*`, `workspace.top_bar.*`,
  `workspace.status_bar.*`, and `workspace.tabstrip.*` in `shadcn_new_york_config(...)`.
- `apps/fret-ui-gallery` plus
  `tools/diag-scripts/ui-gallery/workspace-shell/ui-gallery-workspace-shell-chrome-shadcn-screenshot.json`
  now provide the end-to-end proof surface for this shell-level seeding.
- This intentionally does not seed `workspace.tab.*` yet.

## Rule for Future Skins

The rule for future shadcn / Material / custom-app skins is:

- seed or alias `editor.*`, `workspace.*`, and docking-owned families from the adapter layer,
- never add reverse dependencies from `fret-ui-editor` or `fret-workspace` back into design-system
  crates,
- and keep generic palette / `component.*` fallbacks as compatibility only, not ownership.

This is consistent with ADR 0316 and is the recommended standing rule for future token work.

## Recommended Next Implementation Slice

1. Decide whether `workspace.tab.*` needs adapter-side seeding as well, or can remain
   generic-fallback-first for v1.
2. Decide whether `fret-workspace` should keep only its small resolver surface or also gain an
   opt-in local preset helper for proof/demo use.
3. Decide whether the current `ui_gallery` workspace-shell diagnostics proof should be promoted
   into a recurring suite membership.
4. Audit the editor proof preset so more of its visual intent lands in editor-owned namespaces and
   less in shared palette mutation.

## Evidence Anchors

- `ecosystem/fret-ui-editor/src/primitives/tokens.rs`
- `ecosystem/fret-ui-editor/src/theme.rs`
- `ecosystem/fret-ui-editor/src/primitives/chrome.rs`
- `ecosystem/fret-workspace/src/frame.rs`
- `ecosystem/fret-workspace/src/panes.rs`
- `ecosystem/fret-workspace/src/theme_tokens.rs`
- `ecosystem/fret-workspace/src/tab_strip/theme.rs`
- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`
- `apps/fret-ui-gallery/src/driver/chrome.rs`
- `apps/fret-ui-gallery/src/driver/render_flow.rs`
- `apps/fret-ui-gallery/src/driver/settings_sheet.rs`
- `apps/fret-ui-gallery/src/driver/status_bar.rs`
- `tools/diag-scripts/ui-gallery/workspace-shell/ui-gallery-workspace-shell-chrome-shadcn-screenshot.json`
- `docs/workstreams/theme-token-alignment-v1/todo.md`
