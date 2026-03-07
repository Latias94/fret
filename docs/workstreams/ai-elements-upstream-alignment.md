---
title: AI Elements upstream alignment (inventory + mapping)
status: active
date: 2026-02-12
scope: ecosystem/fret-ui-ai, repo-ref/ai-elements
---

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Vercel AI Elements: https://github.com/vercel/ai-elements

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

# AI Elements upstream alignment (inventory + mapping)

This document is the mechanical source-of-truth for **what exists upstream** and how it maps into
`ecosystem/fret-ui-ai`.

Workstream narrative: `docs/workstreams/ai-elements-port.md`.
TODO tracker: `docs/workstreams/ai-elements-port-todo.md`.

## Upstream snapshot

Local checkout (optional repo-ref checkout):

- `repo-ref/ai-elements`

Version stamp:

- Repo: `vercel/ai-elements`
- Commit: `e2045329c8445ebd0523de6aa755a39d6193841f` (date `2026-02-06`)

## Inventory source

Upstream exports each `.tsx` file directly:

- `repo-ref/ai-elements/packages/elements/src/*.tsx`

## Naming notes (TSX → Rust)

Upstream uses kebab-case filenames (e.g. `prompt-input.tsx`), while Rust modules use snake_case
(e.g. `prompt_input.rs`).

Some upstream "single surfaces" are intentionally split into multiple composable parts in Fret for
app-owned effects and better reuse. Example: upstream `sources.tsx` maps to `SourcesBlock` +
supporting types in `sources_block.rs`.

## Mapping table (current)

Legend:

- **Ported (prototype)**: exists in `fret-ui-ai`, but may still diverge from upstream composition or
  interaction details.
- **Not started**: no Fret surface yet (or intentionally deferred).

| Upstream file | Fret owner | Rust module | Status | Notes |
| --- | --- | --- | --- | --- |
| `conversation.tsx` | `fret-ui-ai` | `conversation*.rs`, `ai_chat.rs`, `ai_conversation.rs` | Ported (prototype) | Compound `Conversation` + `ConversationContent` + overlay parts now mirror the official docs composition more closely, while `AiChat` remains the convenience shell. UI Gallery pages: `ai_chat_demo`, `ai_transcript_torture`, `ai_conversation_demo`. Gates: `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-conversation-demo-scroll-button.json`, `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-conversation-demo-prompt-send-click.json`. |
| `message.tsx` | `fret-ui-ai` | `message*.rs`, `message_response.rs` | Ported (prototype) | Split into message wrapper, toolbar/actions, and response rendering. UI Gallery pages: `ai_message_demo`, `ai_message_branch_demo`, `ai_chat_demo`. |
| `prompt-input.tsx` | `fret-ui-ai` | `prompt_input.rs` | Ported (prototype) | Effects remain app-owned; interaction gated via diag. |
| `tool.tsx` | `fret-ui-ai` | `tool.rs`, `tool_call_block.rs` | Ported (prototype) | Collapsible tool call blocks + status outcomes. |
| `sources.tsx` | `fret-ui-ai` | `sources_block.rs`, `inline_citation.rs` | Ported (prototype) | Split into `SourcesBlock` + `InlineCitation` policy. UI Gallery pages: `ai_sources_demo`, `ai_chat_demo`. |
| `inline-citation.tsx` | `fret-ui-ai` | `inline_citation.rs` | Ported (prototype) | HoverCard outcomes + selection seam. UI Gallery pages: `ai_inline_citation_demo`, `ai_chat_demo`. |
| `reasoning.tsx` | `fret-ui-ai` | `reasoning.rs` | Ported (prototype) | Streaming-driven auto-open/close policy. |
| `suggestion.tsx` | `fret-ui-ai` | `suggestions.rs` | Ported (prototype) | Fret uses plural module name; surfaces `Suggestions`/`Suggestion`. |
| `queue.tsx` | `fret-ui-ai` | `queue.rs` | Ported (prototype) | Policy-heavy; gated via diag. |
| `model-selector.tsx` | `fret-ui-ai` | `model_selector.rs` | Ported (prototype) | Provider icons are placeholders (no remote fetch). UI Gallery page: `ai_model_selector_demo`. Gate: `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-model-selector-demo-open-filter-select.json` (requires `fret-ui-gallery --features gallery-dev`). |
| `attachments.tsx` | `fret-ui-ai` | `attachments.rs` | Ported (prototype) | Chips grid; add/remove intents are app-owned. |
| `chain-of-thought.tsx` | `fret-ui-ai` | `chain_of_thought.rs` | Ported (prototype) | Step-list disclosure surface. |
| `checkpoint.tsx` | `fret-ui-ai` | `checkpoint.rs` | Ported (prototype) | Basic alignment. |
| `confirmation.tsx` | `fret-ui-ai` | `confirmation.rs` | Ported (prototype) | Approve/deny policy surface. Direct compound-children composition and UI Gallery docs page now mirror the official examples. |
| `context.tsx` | `fret-ui-ai` | `context.rs` | Ported (prototype) | Context usage hovercard (percent + progress + compact counts). |
| `plan.tsx` | `fret-ui-ai` | `plan.rs` | Ported (prototype) | Plan item list outcomes. |
| `shimmer.tsx` | `fret-ui-ai` | `shimmer.rs` | Ported (prototype) | Animated shimmer text surface. |
| `artifact.tsx` | `fret-ui-ai` | `artifact.rs` | Ported (prototype) | Artifact container chrome. |
| `code-block.tsx` | `fret-ui-ai` | `code_block.rs` | Ported (prototype) | Backed by `ecosystem/fret-code-view`. |
| `snippet.tsx` | `fret-ui-ai` | `snippet.rs` | Ported (prototype) | Copyable snippet outcomes. UI Gallery pages: `ai_snippet_demo`, `ai_code_block_demo`. |
| `file-tree.tsx` | `fret-ui-ai` | `file_tree.rs` | Ported (prototype) | Flattens via UI Kit tree + `VirtualList`; small-tree focused. |
| `commit.tsx` | `fret-ui-ai` | `commit.rs` | Ported (prototype) | Disclosure + copy actions + file rows. |
| `stack-trace.tsx` | `fret-ui-ai` | `stack_trace.rs` | Ported (prototype) | Parsed frames + copy + seams. |
| `schema-display.tsx` | `fret-ui-ai` | `schema_display.rs` | Ported (prototype) | Schema viewer outcomes. |
| `test-results.tsx` | `fret-ui-ai` | `test_results.rs` | Ported (prototype) | Suite/test disclosures + optional activate seam. |
| `environment-variables.tsx` | `fret-ui-ai` | `environment_variables.rs` | Ported (prototype) | Table-like key/value outcomes. |
| `web-preview.tsx` | `fret-ui-ai` | `web_preview.rs` | Ported (prototype) | Chrome always available; native embed via `webview-wry` behind feature flags. |
| `image.tsx` | `fret-ui-ai` | `image.rs` | Ported (prototype) | Rendering only; decoding/upload is app-owned. UI Gallery page: `ai_image_demo`. |
| `terminal.tsx` | `fret-ui-ai` | `terminal.rs` | Ported (prototype) | Viewer-only (output + copy/clear + auto-scroll); no PTY/TTY in v1. |
| `package-info.tsx` | `fret-ui-ai` | `package_info.rs` | Ported (prototype) | Package card (name/change badge + version row + deps list building blocks). |
| `open-in-chat.tsx` | `fret-ui-ai` | `open_in_chat.rs` | Ported (prototype) | Provider dropdown menu; selecting an entry emits `Effect::OpenUrl` (URLs match upstream). |
| `task.tsx` | `fret-ui-ai` | `task.rs` | Ported (prototype) | Collapsible task surface (trigger + indented content) for “search/plan step” UI. |
| `audio-player.tsx` | `fret-ui-ai` | `audio_player.rs` | Ported (prototype) | UI-only chrome port (controls + time/volume sliders). Playback remains app-owned. |
| `transcription.tsx` | `fret-ui-ai` | `transcription.rs` | Ported (prototype) | Segment surface + optional seek seam (`on_seek`). Playback timing remains app-owned. |
| `jsx-preview.tsx` | n/a | n/a | N/A | Upstream is web-only (JSX render preview). In Fret, prefer app-owned webview previews if needed. |
| `agent.tsx` | `fret-ui-ai` | `agent.rs` | Ported (prototype) | UI-only chrome (instructions/tools/output schema). Add gates: `tools/diag-scripts/ui-gallery-ai-agent-demo-expand-tool.json`. |
| `persona.tsx` | `fret-ui-ai` | `persona.rs` | Ported (prototype) | Variant-aware placeholder shell (upstream uses Rive/webgl2) plus Fret-specific custom visual slot seam. Gate: `tools/diag-scripts/ui-gallery-ai-persona-demo.json`. |
| `sandbox.tsx` | `fret-ui-ai` | `sandbox.rs` | Ported (prototype) | UI-only chrome (collapsible + tabs). Add gates: `tools/diag-scripts/ui-gallery-ai-sandbox-demo-switch-tab.json`. |
| `mic-selector.tsx` | `fret-ui-ai` | `mic_selector.rs` | Ported (prototype) | UI-only chrome + explicit seams (device enumeration is app-owned). Thin selector-level `MicSelectorItem` / `MicSelectorEmpty` wrappers exist; remaining gap is render-props list composition. |
| `speech-input.tsx` | `fret-ui-ai` | `speech_input.rs` | Ported (prototype) | UI-only chrome + explicit seams (capture/ASR backends app-owned). |
| `voice-selector.tsx` | `fret-ui-ai` | `voice_selector.rs` | Ported (prototype) | UI-only chrome + explicit seams (voices list app-owned). |
| `canvas.tsx` | `fret-ui-ai` (chrome) | `workflow/canvas.rs` | Ported (prototype) | UI-only canvas host (editor-like wheel pan + ctrl/cmd wheel zoom via `fret-canvas/ui`). UI Gallery pages: `ai_workflow_canvas_demo`, `ai_workflow_chrome_demo`. Gate: `tools/diag-scripts/ui-gallery-ai-workflow-canvas-demo.json`. |
| `node.tsx` | `fret-ui-ai` (chrome) | `workflow/node.rs` | Ported (prototype) | UI-only node chrome + handle indicators. UI Gallery pages: `ai_workflow_node_demo`, `ai_workflow_chrome_demo`. Gate: `tools/diag-scripts/ui-gallery-ai-workflow-node-demo.json`. |
| `edge.tsx` | `fret-ui-ai` (chrome) | `workflow/edge.rs` | Ported (prototype) | UI-only edge chrome (temporary + animated). UI Gallery pages: `ai_workflow_edge_demo`, `ai_workflow_chrome_demo`. Gate: `tools/diag-scripts/ui-gallery-ai-workflow-edge-demo.json`. |
| `panel.tsx` | `fret-ui-ai` (chrome) | `workflow/panel.rs` | Ported (prototype) | UI-only panel chrome. UI Gallery pages: `ai_workflow_panel_demo`, `ai_workflow_chrome_demo`. Gate: `tools/diag-scripts/ui-gallery-ai-workflow-chrome-demo.json`. |
| `toolbar.tsx` | `fret-ui-ai` (chrome) | `workflow/toolbar.rs` | Ported (prototype) | UI-only toolbar chrome. UI Gallery pages: `ai_workflow_toolbar_demo`, `ai_workflow_chrome_demo`. Gate: `tools/diag-scripts/ui-gallery-ai-workflow-chrome-demo.json`. |
| `controls.tsx` | `fret-ui-ai` (chrome) | `workflow/controls.rs` | Ported (prototype) | UI-only controls chrome. UI Gallery pages: `ai_workflow_controls_demo`, `ai_workflow_chrome_demo`. Gate: `tools/diag-scripts/ui-gallery-ai-workflow-controls-demo.json`. |
| `connection.tsx` | `fret-ui-ai` (chrome) | `workflow/connection.rs` | Ported (prototype) | UI-only connection line chrome. UI Gallery page: `ai_workflow_connection_demo`. Gate: `tools/diag-scripts/ui-gallery-ai-workflow-edge-demo.json`. |

## Selector surface alignment (2026-03-06)

The upstream `mic-selector`, `voice-selector`, and `model-selector` surfaces look similar at a glance,
but they should not collapse into one policy-heavy abstraction in Fret.

The current alignment decision is:

| Surface | Current Fret shape | Upstream shape | Recommendation |
| --- | --- | --- | --- |
| `MicSelector` | UI-only root with controlled/uncontrolled `value_model` + `open_model`, a Rust compound entrypoint (`into_element_with_children(...)`), explicit `MicSelectorItem` / `MicSelectorEmpty` wrappers, and a `MicSelectorList` that supports both auto rows and explicit entries. | Root + `Trigger` + `Value` + `Content` + `Input` + render-props `List(children(data))` + explicit `Empty` / `Item` / `Label`. | Keep the current UI-only seam (device enumeration and permission prompts remain app-owned). Treat the remaining gap as **ecosystem surface alignment**, not a runtime contract gap. The main remaining delta is upstream-style render-props list composition, which should be solved on top of shared `Command` composition rather than by adding new runtime mechanisms. |
| `VoiceSelector` | Richest selector-owned compound surface today: root, content/input/list, shared `Command*` aliases, plus metadata/presentation parts such as `Name`, `Description`, `Gender`, `Accent`, `Age`, `Attributes`, `Bullet`, and `Preview`. | Root + dialog/content/input/list/items plus metadata/presentation parts and a context hook. | Use this as the **naming and taxonomy baseline** for selector compounds, but do not force every selector to grow the same presentation parts. The metadata/presentation parts are selector policy, not universal selector contracts. |
| `ModelSelector` | Thin wrapper over dialog + shared `Command*` parts, plus selector-specific presentation helpers (`Logo`, `LogoGroup`, `Name`). | Thin wrapper over `Dialog` + `Command*` + provider logo/name helpers. | Keep this surface intentionally thin and alias-heavy. Do not turn `ModelSelector` into a policy-heavy root just to match `VoiceSelector`. Thin-wrapper parity is the correct outcome here. |

### Smallest common selector surface

The three selectors should align around a **smallest common surface**, not a fully identical API:

- Root + trigger/content decomposition.
- Shared command-family parts where they exist: `Input`, `List`, `Empty`, `Item`, `Group`,
  `Separator`, `Shortcut`.
- Controlled/uncontrolled open state on the root; controlled/uncontrolled selection state only for
  selectors that semantically own a selected value.
- Docs-style gallery pages that explain which parts are selector-owned vs shared `Command*`
  composition.

### Intentional divergences

These are **intentional** and should stay out of `crates/fret-ui`:

- `MicSelector`: device enumeration, permission prompts, and `devicechange` refresh remain app-owned.
- `VoiceSelector`: voice inventory and preview playback transport remain app-owned.
- `ModelSelector`: provider logo fetching remains local/recipe-owned for now (the current port uses a
  local placeholder badge rather than remote `models.dev` fetches).

### Mechanism vs recipe note

The March 2026 `MicSelector` width regression was not a `crates/fret-ui` contract bug. The root cause
was recipe-level width/stretch propagation in `fret-ui-shadcn::PopoverContent`, fixed in
`ecosystem/fret-ui-shadcn/src/popover.rs` with a focused regression test.

### Evidence anchors

- `ecosystem/fret-ui-ai/src/elements/mic_selector.rs`
- `ecosystem/fret-ui-ai/src/elements/voice_selector.rs`
- `ecosystem/fret-ui-ai/src/elements/model_selector.rs`
- `apps/fret-ui-gallery/src/ui/pages/ai_mic_selector_demo.rs`
- `apps/fret-ui-gallery/src/ui/pages/ai_voice_selector_demo.rs`
- `apps/fret-ui-gallery/src/ui/pages/ai_model_selector_demo.rs`
- `ecosystem/fret-ui-shadcn/src/popover.rs`

## 2026-03-07 closure audit (docs parity + text infra)

The upstream docs inventory is now fully represented in `fret-ui-ai` and in the AI UI Gallery when
`fret-ui-gallery` is built with `--features gallery-dev`. The remaining closure work is now mostly
**parity polish and text-infrastructure cleanup**, not missing-component work.

| Family | Docs / Gallery status | Text infra status | Remaining closure work |
| --- | --- | --- | --- |
| `Conversation` / `AiChat` | Docs-aligned composition is present in `conversation_demo.rs` and `chat_demo.rs`. `AiChat` now reuses the `Conversation` compound instead of hand-rolling transcript overlays. | Stable enough for the current parity pass; not a text-cascade hotspot. | Write the long-transcript selection/search contract and finish the scroll-to-bottom token/styling polish. |
| `Confirmation` | Docs-aligned compound examples exist in `confirmation_demo.rs`, `confirmation_request.rs`, `confirmation_accepted.rs`, and `confirmation_rejected.rs`. | Migrated to inherited description typography and covered by regression tests. | No immediate migration work beyond routine token cleanup. |
| `SourcesBlock` / `InlineCitation` | Upstream docs page and Gallery demos exist (`sources_demo.rs`, `inline_citation_demo.rs`). | Leaf typography now uses the shared preset helper (`typography::preset_text_style_with_overrides`), but source-link / anchor behavior items are still open. | Finish source open-url / stable anchor contracts and keep the component-specific text helper cleanup moving into the remaining AI families. |
| `Agent` / `Sandbox` | Docs pages and Gallery demos exist (`agent_demo.rs`, `sandbox_demo.rs`). | Still use local `text_*` helpers rather than the newer shared passive-text helpers. | Migrate labels / descriptions to shared typography helpers and then remove duplicated local style helpers. |
| Voice surfaces (`AudioPlayer`, `MicSelector`, `VoiceSelector`, `Transcription`) | Docs pages and Gallery demos exist. | Still use local `text_sm` / `text_xs` helpers; this is the largest remaining AI-family text-style migration cluster. | Batch-migrate to shared typography helpers, then close the remaining token cleanup TODOs for voice UI. |
| `Task` / `Persona` | Docs pages and Gallery demos exist (`task_demo.rs`, `persona_demo.rs` plus persona variants). | Still rely on local text helper functions. | Migrate to shared typography helpers and verify children/default-copy parity against upstream docs examples. |
| `Artifact`, `Queue`, `Reasoning`, `Plan`, `EnvironmentVariables`, `PackageInfo`, `SchemaDisplay`, `Terminal`, `ConversationEmptyState`, `ChainOfThought` | Docs pages and Gallery demos exist. | Already migrated or guarded by inherited-typography regression tests / shared helper usage. | Keep parity polish focused on behavior/tokens, not on text-style infrastructure rewrites. |

### Audit takeaway

- The repo already has an alignment-document trail: `ai-elements-upstream-alignment`,
  `ai-elements-port-todo`, `ai-elements-port-milestones`, and the text-style cascade workstream.
- The next efficient migration batch is **not** another container/overlay refactor. It is the
  **local text-helper cleanup batch**: `inline_citation`, `sources_block`, `agent`, `sandbox`, and
  the voice-family surfaces.
- `Conversation` and `Confirmation` should now be treated as reference examples for the desired
  direction: compound children APIs + inherited typography + focused regression gates.

## Known upstream files not yet ported

As of the snapshot above, **all** upstream `.tsx` surfaces are accounted for in `fret-ui-ai`.
 

## Evidence bundles (local)

- 2026-03-06 mic selector open -> filter -> select -> close (PASS):
  - Script: `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-mic-selector-demo-select.json`
  - Launch: `target/debug/fret-ui-gallery` via `cargo run -q -p fretboard -- diag run ... --launch -- target/debug/fret-ui-gallery`
  - Session: `target/fret-diag-mic-selector-binary-debug/sessions/1772779723882-41378`
  - Run id: `1772779726455-ui-gallery-ai-mic-selector-demo-select`

- 2026-03-05 model selector open → filter → select → close (PASS):
  - Script: `tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-model-selector-demo-open-filter-select.json`
  - Launch: `cargo run -p fret-ui-gallery --release --features gallery-dev`
  - Session: `target/fret-diag-codex/sessions/1772683677934-7716`
  - Packed: `target/fret-diag-codex/sessions/1772683677934-7716/share/1772683839231.zip`
  - Run id: `1772683839231`

## Regenerating this diff (developer note)

PowerShell snippet used for the inventory diff (update paths as needed):

```powershell
$ws = git rev-parse --show-toplevel
$up = Join-Path $ws 'repo-ref/ai-elements/packages/elements/src'
$rs = Join-Path $ws 'ecosystem/fret-ui-ai/src/elements'

$upNorm = (Get-ChildItem $up -File -Filter '*.tsx').BaseName | ForEach-Object { $_ -replace '-', '_' }
$rsNames = (Get-ChildItem $rs -File -Filter '*.rs').BaseName | Where-Object { $_ -ne 'mod' }

'missing:'; $upNorm | Where-Object { $rsNames -notcontains $_ } | Sort-Object
```
