# `fret-i18n`

Internationalization primitives and locale services for Fret.

This crate is intentionally lightweight and portable. It provides:

- `LocaleId` (based on `unic-langid`)
- `MessageKey` and `MessageArgs`
- `I18nLookup` (backend trait)
- `I18nService` (preferred locale chain, missing-message behavior, optional pseudo-localization)

## Status

Experimental learning project (not production-ready).

## Design goals

- Keep the core i18n surface backend-agnostic.
- Let apps choose the storage/backend (Fluent, ICU, JSON tables, etc.).
- Make fallback behavior explicit and testable.

