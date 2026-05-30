# Material3 Tabs API Breadth v1

Status: Active
Last updated: 2026-05-30

## Why This Lane Exists

The Material3 token visual matrix closed token evidence for the current Tabs implementation, but
that implementation only models primary navigation tabs. Compose Material3 exposes primary and
secondary tab rows as separate public APIs with different default indicator geometry. That makes the
remaining gap API breadth plus recipe behavior, not missing token-matrix coverage.

This lane keeps the closed token matrix stable and adds the next narrow follow-on for Tabs:
primary and secondary tab variants should be explicit, token-backed, and covered by focused
geometry/semantics gates.

## Source Precedence

- Material Design 3 spec: owns the primary vs secondary tabs taxonomy and user-facing intent.
- Compose Material3: owns renderer-neutral API shape and behavior for `PrimaryTabRow`,
  `SecondaryTabRow`, `PrimaryScrollableTabRow`, and `SecondaryScrollableTabRow`.
- Material Web v30 tokens: remain the generated token source for primary navigation tabs.
- Compose `PrimaryNavigationTabTokens` and `SecondaryNavigationTabTokens`: provide the secondary
  token aliases that Material Web v30 does not generate.
- MUI/Base UI: fallback references only if future web-specific composition or accessibility drift is
  found.

## Source Facts

- Primary tab rows use a content-sized active indicator. Compose's default primary indicator is
  offset with `matchContentSize = true` and a 24 dp minimum width.
- Secondary tab rows use a full-tab-width active indicator. Compose's default secondary indicator is
  offset with `matchContentSize = false`.
- Primary and secondary scrollable tab rows share 52 dp edge padding and a 90 dp minimum tab width.
- `PrimaryNavigationTabTokens` owns the active indicator color, height, and rounded shape.
- `SecondaryNavigationTabTokens` owns secondary container/content colors and height, but does not
  define a rounded active-indicator shape; Compose's secondary indicator is a full-width rectangular
  line.
- The current Material Web v30 generated snapshot in this repo contains
  `md.comp.primary-navigation-tab.*` only, so secondary aliases must be documented and injected as
  Fret Material3 bridge tokens until an upstream generated source appears.

## Target Architecture

- `Tabs` remains the single Fret recipe root, with an explicit variant API rather than a second
  near-duplicate component.
- The public variant enum is small and stable: primary is the default, secondary is opt-in.
- Token key mapping lives in `tokens::tabs`; component code asks for typed primary/secondary token
  outcomes instead of spelling token keys inline.
- Indicator geometry remains recipe-owned because it depends on measured tab and label bounds.
- Shared active-indicator paint/motion remains in `foundation::active_indicator`.
- No Material-specific policy moves into `crates/fret-ui`.

## In Scope

- Add an explicit primary/secondary Tabs variant API.
- Route container, label, state-layer, focus, and active-indicator geometry through typed variant
  token helpers.
- Seed secondary navigation-tab aliases in the v30 theme config with Compose-backed source notes.
- Add focused tests proving primary content-sized and secondary full-width indicator geometry.
- Keep scrollable edge padding and minimum tab width behavior source-aligned for both variants.
- Update workstream gates and handoff as the implementation lands.

## Out Of Scope

- Full icon-and-label tab support.
- Divider rendering under tab rows.
- One-to-one Compose API duplication such as separate `PrimaryTabRow`/`SecondaryTabRow` Rust types.
- Reopening `material3-token-visual-matrix-v1`.
- Moving roving focus, selectable semantics, or layout mechanisms into Material3.

## Closeout Condition

This lane can close when:

- primary and secondary Tabs variants are explicit in the public API,
- secondary token aliases are resolved in the v30 theme config with source-backed notes,
- fixed and scrollable secondary indicator geometry is tested,
- existing primary Tabs behavior remains covered,
- targeted Material3 Tabs gates pass,
- remaining richer tabs breadth is split into narrow follow-ons instead of hidden in this lane.
