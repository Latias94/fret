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
- [x] TSC-kit-024 Decide whether the description family should expose a shared composable `children` API now that subtree refinement exists.
  - Decision: yes, but selectively at the component/recipe layer (not runtime). `AlertDescription` remains the baseline; `CardDescription` and `DialogDescription` now also expose `new_children(...)`, while text-only surfaces stay unchanged until upstream usage or product pressure justifies the extra API.

## D. Component enhancement / migration matrix

## D0. Description `children` API decision

| Surface | Current API | Decision | Why | Evidence |
| --- | --- | --- | --- | --- |
| `AlertDescription` | `new`, `new_children` | Keep as-is | Canonical proof that subtree description scoping works for nested passive text | `ecosystem/fret-ui-shadcn/src/alert.rs`, `alert_description_children_scope_inherited_text_style` |
| `CardDescription` | `new` | Add `new_children` | Upstream shadcn frequently nests multiline content inside card descriptions | `ecosystem/fret-ui-shadcn/src/card.rs`, `card_description_children_scope_inherited_text_style` |
| `DialogDescription` | `new` | Add `new_children` | Upstream dialogs often carry composed description copy while still needing modal description registration | `ecosystem/fret-ui-shadcn/src/dialog.rs`, `dialog_description_children_scope_inherited_text_style` |
| `SheetDescription`, `PopoverDescription`, `AlertDialogDescription`, `EmptyDescription` | `new` | Keep text-only for now | No current in-tree pressure point; subtree-scoping helper already exists if product usage appears | `ecosystem/fret-ui-shadcn/src/{sheet,popover,alert_dialog,empty}.rs` |
| `FieldDescription` | `new` + wrap/overflow knobs | Keep text-only for now | It also participates in field registry / `aria-describedby` style wiring, so we avoid widening the public surface without concrete demand | `ecosystem/fret-ui-shadcn/src/field.rs` |

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
| shadcn / list / menu-ish | `ItemDescription` | `ecosystem/fret-ui-shadcn/src/item.rs` | Root-owned inherited description refinement + bare text leaf with local two-line clamp | Landed on the shared description helper path while preserving item-specific truncation semantics | Repeated list/collection supporting copy pattern | P2 | `[x]` | `item_description_scopes_inherited_text_style` |
| shadcn / empty state | `EmptyDescription` | `ecosystem/fret-ui-shadcn/src/empty.rs` | Root-owned inherited description refinement + bare text leaf | Landed on the shared description helper path while keeping empty-state layout semantics local | Keeps empty-state body copy aligned with other description surfaces | P2 | `[x]` | `empty_description_scopes_inherited_text_style` |
| AI / voice selector | `VoiceSelectorDescription` | `ecosystem/fret-ui-ai/src/elements/voice_selector.rs` | Root-owned inherited description refinement + bare text leaf with xs fallback | Landed on shared description helper path with `component.text.xs_*` fallback while keeping one-line ellipsis local | Repeated docs-like helper copy in AI settings surfaces | P2 | `[x]` | `voice_selector_description_scopes_inherited_text_style` |
| AI / plan | `PlanDescription` | `ecosystem/fret-ui-ai/src/elements/plan.rs` | Static path delegates to `CardDescription`; streaming path now scopes inherited description typography and uses subtree-resolved `Shimmer` text style | Landed on the shared description helper path for static content and the subtree-resolved shimmer bridge for streaming content; remaining `Shimmer` follow-up stays tracked in `docs/workstreams/shimmer-text-style-source-fearless-refactor-v1/` | Closes the semantic supporting-copy gap without reintroducing leaf-owned typography | P2 | `[x]` | `plan_description_streaming_scopes_inherited_description_typography_for_shimmer`, `docs/workstreams/shimmer-text-style-source-fearless-refactor-v1/TODO.md` |
| AI / empty state | `ConversationEmptyState` description | `ecosystem/fret-ui-ai/src/elements/conversation_empty_state.rs` | Optional description leaf now scopes inherited description refinement + muted foreground while keeping the title explicitly component-owned | Landed on subtree-local description ownership without reintroducing leaf-level color/style patching | Prevent future nested-child patching pressure in chat empty states | P2 | `[x]` | `empty_state_description_scopes_inherited_description_typography` |
| AI / reasoning | `ChainOfThoughtStep` description text slot | `ecosystem/fret-ui-ai/src/elements/chain_of_thought.rs` | Slot-owned composable xs refinement + shared muted subtree scope for text/children branches | Landed on subtree-local inherited refinement so text and children slots converge without forcing leaf styles | Important mixed text/children pressure point | P2 | `[x]` | `chain_of_thought_step_description_scopes_inherited_typography_for_text_and_children` |
| kit / foundation | `Label` + passive text helpers | `ecosystem/fret-ui-kit/src/primitives/label.rs`, `ecosystem/fret-ui-kit/src/declarative/text.rs` | Root-owned inherited refinement + bare passive text leaves | Landed on composable subtree refinements; leaf style/color stay unset unless the surface semantically owns an override | Foundation surface must not fight the cascade | P0 | `[x]` | `label_defaults_match_shadcn_expectations`, `selectable_label_scopes_inherited_refinement_without_leaf_style`, `text_sm_scopes_inherited_refinement_without_leaf_style`, `prose_variants_and_code_wrap_install_semantic_inherited_overrides` |
| shadcn / data display | `TableCaption` | `ecosystem/fret-ui-shadcn/src/table.rs` | Caption leaf used explicit table text style + muted color | Landed on subtree-local inherited refinement via `scope_text_style_with_color(...)` while preserving table-local metrics | First non-description shadcn supporting-copy adopter; proves the generic subtree helper path | P1 | `[x]` | `table_caption_scopes_inherited_text_style_without_leaf_overrides` |
| kit / overlays | `Toast` / `Sonner` description path | `ecosystem/fret-ui-kit/src/window_overlays/render.rs`, `ecosystem/fret-ui-shadcn/src/sonner.rs` | Renderer-owned description leaf used explicit style + muted color | Landed on subtree-local inherited refinement via `scope_text_style_with_color(...)` while keeping title semantics explicit | High-traffic overlay supporting-copy surface outside the description family | P1 | `[x]` | `toast_description_scopes_inherited_text_style_without_leaf_overrides` |

Audit snapshot after the current in-tree pass:

- AI supporting-copy surfaces tracked in this matrix are now migrated; no additional mechanism gap is blocking them.
- Remaining open items are broader audit/cleanup work (`TSC-audit-004`, `TSC-cleanup-042`) and intentionally explicit authoring seams such as `TranscriptionSegment::text_style(...)`.

## D1. Remaining audit / enhancement candidates

| Surface | File | Current state | Recommended next action | Why it still matters |
| --- | --- | --- | --- | --- |
| Text/body passive leaves outside the current matrices | cross-crate audit | The highest-value AI/shadcn supporting-copy surfaces are migrated, but the repo still has long-tail passive leaves that may rebuild local style/color contracts | Continue `TSC-audit-004` with a bias toward supporting-copy surfaces before touching semantic owner text | Keeps ADR 0314 focused on real duplication instead of speculative abstraction |
| Explicit authoring seams such as `TranscriptionSegment::text_style(...)` | `ecosystem/fret-ui-ai/src/elements/transcription.rs` and similar surfaces | These remain intentional leaf-owned APIs even after inherited typography landed | Keep them explicit unless product usage shows a repeated subtree-default pressure point | Avoids over-generalizing the passive-text cascade into interactive/editor-like surfaces |

## D2. Title API audit decisions

| Surface | Current API | Decision | Why | Evidence |
| --- | --- | --- | --- | --- |
| `ArtifactTitle` | `new(text)` | Add `new_children(...)` | Upstream AI Elements title is paragraph-like `children` content; nested passive text should inherit component-owned title styling instead of forcing leaf patches | `ecosystem/fret-ui-ai/src/elements/artifact.rs`, `artifact_title_children_scope_inherited_title_typography` |
| `EnvironmentVariablesTitle` | `new()` + `.text(...)` | Add `new_children(...)` | Upstream heading accepts `children` with a default fallback; Fret now matches that shape without adding runtime text surface area | `ecosystem/fret-ui-ai/src/elements/environment_variables.rs`, `environment_variables_title_children_scope_inherited_typography` |
| `TerminalTitle` | `new()` + `.label(...)` + `.icon(...)` | Add `new_children(...)` while keeping convenience builders | Upstream terminal title is children-first but also carries default label + icon semantics; selective component API keeps both paths | `ecosystem/fret-ui-ai/src/elements/terminal.rs`, `terminal_title_children_scope_inherited_typography` |
| `CodeBlockTitle` | children-only container | Keep as-is | Already matches upstream composable header-title role; no extra text helper needed | `ecosystem/fret-ui-ai/src/elements/code_block.rs`, `code_block_children_can_consume_inherited_context` |
| `AnnouncementTitle` | children-only container | Keep as-is | Already composable and layout-owned; no text-style migration needed | `ecosystem/fret-ui-shadcn/src/extras/announcement.rs` |
| `BannerTitle` | `new(text)` | Keep text-first for now | Local extras recipe with plain-copy demand only; no upstream/product pressure justifies widening the public API yet | `ecosystem/fret-ui-shadcn/src/extras/banner.rs` |

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
  - `fret-ui-ai` surfaces already migrated / guarded enough to treat as reference examples: `confirmation`, `conversation_empty_state`, `artifact`, `queue`, `schema_display`, `terminal`, `package_info`, `environment_variables`, `plan`, `reasoning`, `chain_of_thought`.
  - Follow-up slices now done: `inline_citation` + `sources_block`, then `agent` + `sandbox`, then `task` + `persona`, then `audio_player` + `mic_selector` + `voice_selector` + `transcription`, and now `chain_of_thought`, all moved to the shared preset helper or inherited text scopes.
  - `fret-ui-ai` local-helper cleanup is complete; remaining work is broader ecosystem adoption plus intentional authoring seams such as `TranscriptionSegment::text_style(...)`.
- [ ] TSC-cleanup-043 Remove temporary compatibility shims once migrated surfaces are covered by tests.

## G. Docs / alignment

- [x] TSC-docs-050 Add ADR 0314.
- [x] TSC-docs-051 Add this workstream doc set.
- [x] TSC-docs-052 Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` as implementation progresses.
- [x] TSC-docs-053 Update user-facing authoring guidance after the helper surface lands.
