---
title: shadcn/ui v4 Component Parity (archived)
---

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

# shadcn/ui v4 Component Parity (archived)

This document is archived and kept for historical context only.

**Source of truth:** `docs/shadcn-declarative-progress.md`

## Notes

- At the pinned `repo-ref/ui` commit, the registry index references `ui/toast.tsx`, `ui/toaster.tsx`,
  and `hooks/use-toast.ts`, but those files are not present in-tree. Prefer using `sonner` as the
  upstream toast reference until the pin is updated.
