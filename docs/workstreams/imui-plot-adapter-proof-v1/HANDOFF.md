# IMUI Plot Adapter Proof v1 - Handoff

Status: Closed
Last updated: 2026-05-25

Current slice: closed on 2026-05-25.

IPA-010 status: complete. This lane owns only the optional `fret-plot/imui` adapter. Plotting stays
in `fret-plot`; `fret-imui` and `fret-ui-kit::imui` stay free of plot dependencies.

IPA-020 status: complete. `fret-plot/imui` adds `UiWriter` helpers that delegate to the existing
declarative plot panels.

IPA-030 status: complete. Default and `imui` feature compile gates pass, the focused source-policy
test passes, the IMUI source-policy gate passes, the catalog validates 439 dedicated directories,
and `git diff --check` reports no whitespace errors.

Closeout:

1. Cookbook or canonical-workbench adoption remains deferred unless product routes expose repeated
   plot authoring friction.
2. Root `fret::imui` plot sugar remains deferred until at least two product surfaces prove the same
   shorthand is needed.
