# fret-ui-ai

Opinionated AI/chat UI components for Fret, built on top of `fret-ui-shadcn`.

This crate is an ecosystem-layer port inspired by Vercel's `ai-elements`:

- Upstream reference: `repo-ref/ai-elements/packages/elements`
- UI baseline: `repo-ref/ai-elements/packages/shadcn-ui`

The goal is to provide a modular, policy-heavy component surface for "assistant / tooling" UIs
(messages, conversation views, prompts, tool calls, diffs, file trees, etc.) without pushing these
policies into `fret-ui` / `fret-ui-kit`.

