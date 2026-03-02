# Action-First Authoring + View Runtime (Fearless Refactor v1) — Parity Matrix

Last updated: 2026-03-01

This matrix is a checklist of **outcomes**, not 1:1 API parity.

Legend:

- `✅` supported / landed
- `⚠️` partial
- `❌` not yet
- `🧭` intentionally different (document rationale)

| Area | GPUI/Zed reference | Fret today | Target (v1) | Notes |
| --- | --- | --- | --- | --- |
| Typed unit actions | `gpui::action` | ⚠️ `ActionId` alias + typed unit action macro | ✅ | v1 authoring sugar landed; dispatch remains command-based until handler tables land |
| Structured action payloads | Keymap JSON payload | ❌ | ❌ (v2) | Gate behind strict validation and determinism rules |
| Key contexts | `key_context("...")` | command routing + keymap contexts exist | ⚠️ | Converge on action-first queryable contexts |
| Availability queries | `is_action_available` | command availability exists | ⚠️ | Ensure availability traces and UI disabled states align |
| Command palette integration | actions visible as commands | command registry exists | ⚠️ | Converge metadata registries (no duplication) |
| View-level dirty/notify loop | “notify → dirty views” | in progress (cache roots workstream) | ⚠️ | Align with ADR 0165/0213 and gpui-parity plan |
| View cache reuse semantics | `AnyView::cached` | `ViewCache` experimental kind exists | ⚠️ | Make reuse predictable and diagnosable |
| Immediate-mode frontend | n/a (GPUI is declarative-first) | ✅ (`fret-imui`) | ✅ | Must dispatch actions without string glue |
| Data-driven UI spec frontend | not first-class in GPUI | ✅ (GenUI) | ✅ | Align action IDs and metadata where possible |
