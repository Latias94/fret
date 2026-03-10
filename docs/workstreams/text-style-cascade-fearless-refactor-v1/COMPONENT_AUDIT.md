# Text Style Cascade (Fearless Refactor v1) — Component Audit

Status: In progress.

Primary contract: `docs/adr/0314-inherited-text-style-cascade-and-refinement-v1.md`

This note complements `DESIGN.md`, `TODO.md`, and `MILESTONES.md`.
It answers a narrower question: after ADR 0314 lands, which component text surfaces still need
migration, and which ones should remain explicit semantic owners?

## Decision rubric

- **Migrate now**: description/supporting-copy surfaces that still rebuild the same inherited
  typography contract locally.
- **Keep explicit owner**: the component semantically owns the final typography, casing, or
  foreground outcome; subtree inheritance is not the right abstraction.
- **Selective children API**: keep ownership in the component, but allow nested passive text by
  exposing a composable `children` entrypoint at the ecosystem layer.
- **Out of scope for v1**: editor/input/code surfaces or highly specialized leaves that ADR 0314
  intentionally leaves unchanged.

## Current audit snapshot

| Surface | Layer | Current shape | Decision | Why | Evidence |
| --- | --- | --- | --- | --- | --- |
| `ConversationEmptyState` description | `fret-ui-ai` | Empty-state supporting copy under a centered shell | Migrate now | It is description-like supporting text and should consume subtree-local inherited refinement instead of leaf-owned style/color | `ecosystem/fret-ui-ai/src/elements/conversation_empty_state.rs`, `repo-ref/ai-elements/packages/elements/src/conversation.tsx` |
| `ArtifactTitle` | `fret-ui-ai` | Title surface with paragraph-like upstream composition | Selective children API | Upstream uses `children`; nested passive text should inherit title typography without adding runtime policy | `ecosystem/fret-ui-ai/src/elements/artifact.rs`, `repo-ref/ai-elements/packages/elements/src/artifact.tsx` |
| `EnvironmentVariablesTitle` | `fret-ui-ai` | Heading with default fallback plus optional custom content | Selective children API | Upstream accepts `children` while preserving a default label | `ecosystem/fret-ui-ai/src/elements/environment_variables.rs`, `repo-ref/ai-elements/packages/elements/src/environment-variables.tsx` |
| `TerminalTitle` | `fret-ui-ai` | Title with default label + icon semantics | Selective children API | Upstream is children-first, but Fret still needs convenient default label/icon builders | `ecosystem/fret-ui-ai/src/elements/terminal.rs`, `repo-ref/ai-elements/packages/elements/src/terminal.tsx` |
| `CodeBlockTitle` | `fret-ui-ai` | Composable header title container | Keep explicit owner | Already matches the upstream composable shape; no new text helper is needed | `ecosystem/fret-ui-ai/src/elements/code_block.rs`, `repo-ref/ai-elements/packages/elements/src/code-block.tsx` |
| `ConfirmationTitle` | `fret-ui-ai` | Component-owned title with nested description pressure below it | Keep explicit owner | The migration target was the inherited supporting-copy path, not a generic title helper | `ecosystem/fret-ui-ai/src/elements/confirmation.rs`, `repo-ref/ai-elements/apps/docs/content/components/(chatbot)/confirmation.mdx` |
| `VoiceSelectorDescription` | `fret-ui-ai` | Supporting-copy description slot | Already migrated | It now uses subtree-local inherited description typography instead of a leaf-owned contract | `ecosystem/fret-ui-ai/src/elements/voice_selector.rs` |
| `QueueItemDescription` | `fret-ui-ai` | Supporting-copy row beneath queue content | Already migrated | It is a canonical description-style surface and now consumes the shared helper path | `ecosystem/fret-ui-ai/src/elements/queue.rs` |
| `PackageInfoDescription` | `fret-ui-ai` | Supporting-copy body text | Already migrated | Another non-overlay description surface now aligned with subtree-local defaults | `ecosystem/fret-ui-ai/src/elements/package_info.rs` |
| `SchemaDisplayDescription` | `fret-ui-ai` | Supporting-copy description under schema header | Already migrated | Demonstrates the shared helper path outside shadcn card/dialog shells | `ecosystem/fret-ui-ai/src/elements/schema_display.rs` |
| `PlanTitle` / `PlanDescription` | `fret-ui-ai` | Streaming-aware title/description surfaces | Already migrated | These surfaces prove inherited typography plus shimmer-resolved fallback can coexist cleanly | `ecosystem/fret-ui-ai/src/elements/plan.rs`, `docs/adr/0315-shimmer-resolved-text-style-source-v1.md` |
| `BannerTitle` | `fret-ui-shadcn` extras | Local text-first recipe surface | Keep explicit owner | There is no upstream/product pressure for a wider public API yet | `ecosystem/fret-ui-shadcn/src/extras/banner.rs` |
| `AnnouncementTitle` | `fret-ui-shadcn` extras | Composable title container | Keep explicit owner | It is already composable and does not indicate a missing mechanism | `ecosystem/fret-ui-shadcn/src/extras/announcement.rs` |
| `ToolHeader` / `ToolSectionTitle` | `fret-ui-ai` | Chrome labels, uppercase section headings, and status-aware labels | Keep explicit owner | These are semantic chrome labels, not generic supporting-copy surfaces | `ecosystem/fret-ui-ai/src/elements/tool.rs` |
| `ContextContentHeader` / `ContextContentFooter` | `fret-ui-ai` | Numeric metrics, counters, and cost labels | Keep explicit owner | These leaves intentionally own measurement, casing, alignment, and monospace choices | `ecosystem/fret-ui-ai/src/elements/context.rs` |
| `TextInput`, `TextArea`, editors, code/editor surfaces | runtime / ecosystem | Interactive text ownership | Out of scope for v1 | ADR 0314 explicitly limits v1 to passive text | `docs/adr/0314-inherited-text-style-cascade-and-refinement-v1.md` |

## Practical conclusion

The remaining work is **not** “add more runtime text helpers”.

The repo now needs three narrower follow-ups instead:

1. Land the remaining **supporting-copy migrations** that are already sitting in the tree.
2. Keep auditing title-like surfaces as **component-layer API decisions**, not runtime mechanism
   work.
3. Avoid adding new passive-text components that manually rebuild description/body copy contracts
   outside `fret_ui_kit::typography`.
