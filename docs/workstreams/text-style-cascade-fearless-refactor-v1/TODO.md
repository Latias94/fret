# Text Style Cascade (Fearless Refactor v1) — TODO

Status legend:

- `[ ]` not started
- `[~]` in progress / partially aligned
- `[x]` landed
- `[!]` blocked by a larger mechanism decision

## A. Audit / contract closure

- [x] TSC-audit-001 Confirm that this is a hard contract and deserves an ADR.
  - Outcome: yes; see ADR 0314.
- [x] TSC-audit-002 Audit GPUI/Zed's subtree text refinement model.
  - Evidence:
    - `repo-ref/zed/crates/gpui/src/style.rs`
    - `repo-ref/zed/crates/gpui/src/styled.rs`
    - `repo-ref/zed/crates/gpui/src/window.rs`
    - `repo-ref/zed/crates/gpui/src/elements/text.rs`
- [x] TSC-audit-003 Record the current Fret baseline.
  - Evidence:
    - `crates/fret-ui/src/element.rs`
    - `crates/fret-ui/src/text/props.rs`
    - `ecosystem/fret-ui-kit/src/declarative/current_color.rs`
- [ ] TSC-audit-004 Audit all passive text leaves that currently bypass subtree-local defaults.
- [ ] TSC-audit-005 Decide whether align/wrap/overflow belong in v1 cascade or remain leaf-owned.

## B. Mechanism design (`crates/fret-core` / `crates/fret-ui`)

- [x] TSC-mech-010 Add `TextStyleRefinement` to the portable text contract.
- [x] TSC-mech-011 Define `TextStyle + TextStyleRefinement` merge/refine semantics.
- [x] TSC-mech-012 Add inherited text-style runtime propagation in `crates/fret-ui`.
- [x] TSC-mech-013 Teach passive text leaves to resolve `explicit > inherited > theme default`.
- [x] TSC-mech-014 Ensure inherited text style participates in text measure/cache keys.
- [x] TSC-mech-015 Keep the initial consumer set narrow:
  - `Text`
  - `StyledText`
  - `SelectableText`
- [x] TSC-mech-016 Leave `TextInput`, `TextArea`, editor/code surfaces unchanged in v1.

## C. Ecosystem authoring (`fret-ui-kit`)

- [x] TSC-kit-020 Add subtree text-style helpers.
  - Landed in `ecosystem/fret-ui-kit/src/typography.rs`:
    - `scope_text_style(...)`
    - `scope_text_style_with_color(...)`
    - `scope_description_text(...)`
- [x] TSC-kit-021 Add a preset-to-refinement bridge for stable semantic typography.
  - Landed as `fret_ui_kit::typography::preset_text_refinement(...)`.
- [x] TSC-kit-022 Align the helper story with `ui-typography-presets-v1`.
  - The subtree helper and preset bridge now live in `ecosystem/fret-ui-kit/src/typography.rs`.
- [x] TSC-kit-023 Document the “one boring path” for subtree-local passive text defaults.
  - See the “Landed authoring surface” section in `docs/workstreams/text-style-cascade-fearless-refactor-v1/DESIGN.md`.
- [ ] TSC-kit-024 Decide whether the description family should expose a shared composable `children` API now that subtree refinement exists.

## D. Component enhancement / migration matrix

| Area | Surface | File | Current pattern | Target after cascade | Why migrate | Priority | Status | Gate / evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| shadcn / feedback | `AlertDescription` | `ecosystem/fret-ui-shadcn/src/alert.rs` | Scoped inherited foreground + inherited text refinement (including `new_children`) | Landed on subtree refinement; nested passive text stays unpatched | Canonical baseline for description/body copy | P0 | `[x]` | `alert_description_children_scope_inherited_text_style` |
| shadcn / overlay | `DialogDescription` | `ecosystem/fret-ui-shadcn/src/dialog.rs` | Scoped inherited foreground + inherited text refinement on the returned part root | Landed on subtree refinement without leaf overrides | Repeated description logic; high reuse pattern | P0 | `[x]` | `dialog_description_scopes_inherited_text_style` |
| shadcn / overlay | `SheetDescription` | `ecosystem/fret-ui-shadcn/src/sheet.rs` | Scoped inherited foreground + inherited text refinement on the returned part root | Landed on subtree refinement without leaf overrides | Same pattern as dialog; easy drift source | P0 | `[x]` | `sheet_description_scopes_inherited_text_style` |
| shadcn / overlay | `PopoverDescription` | `ecosystem/fret-ui-shadcn/src/popover.rs` | Scoped inherited foreground + inherited text refinement on the returned part root | Landed on subtree refinement without leaf overrides | Keeps description policy consistent across overlay family | P1 | `[x]` | `popover_description_scopes_inherited_text_style` |
| shadcn / content | `CardDescription` | `ecosystem/fret-ui-shadcn/src/card.rs` | Scoped inherited foreground + inherited text refinement on the returned part root | Landed on subtree refinement without leaf overrides | Common docs/UI surface; currently duplicated | P1 | `[x]` | `card_description_scopes_inherited_text_style` |
| shadcn / forms | `FieldDescription` | `ecosystem/fret-ui-shadcn/src/field.rs` | Scoped inherited foreground + inherited text refinement; field description detection also understands inherited styles | Landed on subtree refinement without leaf overrides | High-value form surface; many call sites | P0 | `[x]` | `field_description_scopes_inherited_text_style` |
| AI / approvals | `ConfirmationTitle` | `ecosystem/fret-ui-ai/src/elements/confirmation.rs` | Inherited text-style scope + muted foreground | Landed on the inherited text-style mechanism; no descendant patching | Direct compound-children pressure point; first migration proves the runtime path | P0 | `[x]` | `confirmation_title_scopes_alert_description_typography_without_patching_nested_plain_text` |
| AI / artifacts | `ArtifactDescription` | `ecosystem/fret-ui-ai/src/elements/artifact.rs` | Root-owned inherited description refinement + bare text leaf | Landed on shared description helper with component metric fallback | Likely to benefit from shared description policy | P1 | `[x]` | `artifact_description_scopes_inherited_description_typography` |
| AI / package info | `PackageInfoDescription` | `ecosystem/fret-ui-ai/src/elements/package_info.rs` | Container root scopes inherited description refinement + bare text child | Landed on shared description helper with `component.text.sm_*` fallback | Reduces bespoke text setup | P1 | `[x]` | `package_info_description_scopes_inherited_description_typography` |
| AI / queue | `QueueItemDescription` | `ecosystem/fret-ui-ai/src/elements/queue.rs` | Root-owned composable preset refinement + bare text/styled-text leaf | Landed on composable preset refinement so completed/plain states share subtree typography | Repeated muted/supporting copy pattern | P1 | `[x]` | `queue_item_description_scopes_inherited_typography_for_plain_and_completed_states` |
| AI / schema | `SchemaDisplayDescription` + `schema_inline_description` | `ecosystem/fret-ui-ai/src/elements/schema_display.rs` | Container root scopes inherited description refinement + bare raw-text child | Landed on shared description helper with `component.text.sm_*` fallback for public + inline helper surfaces | Keeps AI docs-like surfaces consistent | P1 | `[x]` | `schema_display_description_scopes_inherited_description_typography`, `schema_inline_description_scopes_inherited_description_typography` |
| shadcn / dialog | `AlertDialogDescription` | `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` | Root-owned inherited description refinement + bare text leaf | Landed on the shared shadcn description helper path | Keeps alert-like overlays on one description path | P1 | `[x]` | `alert_dialog_description_scopes_inherited_text_style` |
| shadcn / list / menu-ish | `ItemDescription` | `ecosystem/fret-ui-shadcn/src/item.rs` | Component-owned supporting copy | Prefer subtree-local inherited refinement if the part remains passive text | Repeated list/collection supporting copy pattern | P2 | `[ ]` | Audit needed |
| shadcn / empty state | `EmptyDescription` | `ecosystem/fret-ui-shadcn/src/empty.rs` | Root-owned inherited description refinement + bare text leaf | Landed on the shared description helper path while keeping empty-state layout semantics local | Keeps empty-state body copy aligned with other description surfaces | P2 | `[x]` | `empty_description_scopes_inherited_text_style` |
| AI / voice selector | `VoiceSelectorDescription` | `ecosystem/fret-ui-ai/src/elements/voice_selector.rs` | Component-owned muted supporting copy | Migrate to subtree-local inherited refinement or composable preset bridge | Repeated docs-like helper copy in AI settings surfaces | P2 | `[ ]` | Audit needed |
| AI / plan | `PlanDescription` | `ecosystem/fret-ui-ai/src/elements/plan.rs` | Static path delegates to `CardDescription`; streaming path still owns explicit `Shimmer` text style | Keep static path on shared description helper and migrate streaming shimmer once that surface can consume inherited refinement without losing overlay metrics | Likely same policy as card/description family, but `Shimmer` is still a visual-text exception | P2 | `[~]` | `ecosystem/fret-ui-ai/src/elements/plan.rs:354` |
| AI / empty state | `ConversationEmptyState` description | `ecosystem/fret-ui-ai/src/elements/conversation_empty_state.rs` | Component-owned description branch | Prefer subtree-local inherited refinement for the optional description subtree | Prevent future nested-child patching pressure | P2 | `[ ]` | Audit needed |
| AI / reasoning | `ChainOfThoughtStep` description text slot | `ecosystem/fret-ui-ai/src/elements/chain_of_thought.rs` | Slot-owned explicit `text_xs` style | Prefer subtree-local inherited refinement so text and children slots converge | Important mixed text/children pressure point | P2 | `[ ]` | Audit needed |
| kit / foundation | `Label` + passive text helpers | `ecosystem/fret-ui-kit/src/primitives/label.rs`, `ecosystem/fret-ui-kit/src/declarative/text.rs` | Root-owned inherited refinement + bare passive text leaves | Landed on composable subtree refinements; leaf style/color stay unset unless the surface semantically owns an override | Foundation surface must not fight the cascade | P0 | `[x]` | `label_defaults_match_shadcn_expectations`, `selectable_label_scopes_inherited_refinement_without_leaf_style`, `text_sm_scopes_inherited_refinement_without_leaf_style`, `prose_variants_and_code_wrap_install_semantic_inherited_overrides` |

## E. Gates

- [x] TSC-gates-030 Add a `crates/fret-ui` test proving inherited text style affects passive text measurement.
- [x] TSC-gates-031 Add a precedence test proving explicit leaf style wins over inherited refinement.
- [x] TSC-gates-032 Add a guard proving `TextInput` / `TextArea` are unaffected in v1.
- [x] TSC-gates-033 Add shadcn description-family regression gates.
- [x] TSC-gates-034 Add at least one AI/direct-children regression gate after mechanism migration.

## F. Cleanup / de-duplication

- [x] TSC-cleanup-040 Remove component-local recursive descendant patching once inherited text style lands.
  - First candidate: `ecosystem/fret-ui-ai/src/elements/confirmation.rs`
- [x] TSC-cleanup-041 Consolidate repeated description typography metric lookup logic across shadcn surfaces.
  - Initial family:
    - `alert`
    - `dialog`
    - `sheet`
    - `popover`
    - `card`
    - `field`
- [~] TSC-cleanup-042 Stop introducing new passive-text components that manually rebuild the same description/body text style contract.
  - Guardrail now lives in `ecosystem/fret-ui-kit/src/typography.rs`; broader adoption still depends on migrating more passive-text surfaces.
- [ ] TSC-cleanup-043 Remove temporary compatibility shims once migrated surfaces are covered by tests.

## G. Docs / alignment

- [x] TSC-docs-050 Add ADR 0314.
- [x] TSC-docs-051 Add this workstream doc set.
- [x] TSC-docs-052 Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` as implementation progresses.
- [x] TSC-docs-053 Update user-facing authoring guidance after the helper surface lands.
