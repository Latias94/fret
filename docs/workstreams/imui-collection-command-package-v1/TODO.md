# ImUi Collection Command Package v1 - TODO

- [x] Freeze the lane-opening rationale and assumptions-first baseline for broader app-owned command-package depth after modularization closed.
- [x] Land the first duplicate-selected slice with one proof-local helper shared by keyboard, explicit button, and context menu.
- [x] Add unit/surface/source-policy gates for the command-package slice.
- [x] Land one more narrow rename-trigger slice that routes the existing inline rename flow through an explicit button without widening shared helpers.
- [x] Decide whether the current package is coherent enough to close or whether one more narrow command verb should land before closeout.
      Result: `CLOSEOUT_AUDIT_2026-04-23.md` now closes the lane on the bounded duplicate-selected
      plus explicit rename-trigger package and explicitly rejects reopening this folder for a third
      verb.
- [x] If the lane closes, refresh the default next non-multi-window priority to the second proof surface rather than reopening generic helper questions.
      Result: `docs/workstreams/imui-collection-second-proof-surface-v1/` now records the closed
      follow-on and no-helper-widening verdict, while `WORKSTREAM.json` and the umbrella priority
      audit keep this folder as a closed closeout record.
