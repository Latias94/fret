# IME Acceptance Checklist (Chinese IME)

This checklist exists to keep IME behavior "editor-grade" as the component surface scales.

> [!NOTE]
> Maintainer/labs document.
>
> This checklist uses the `apps/fret-demo` harness and is not part of the first-hour onboarding
> path. For onboarding, start with [docs/first-hour.md](./first-hour.md) and
> [docs/examples/README.md](./examples/README.md).

Target platform for this checklist:

- Windows 11 + Microsoft Pinyin (微软拼音)

Run a harness:

- `cargo run -p fret-demo --bin ime_smoke_demo`

## Single-line input

1. Switch to Microsoft Pinyin and focus the single-line input.
2. Type `nihao`.
   - Expect: inline preedit is visible (marked/underlined) inside the input.
   - Expect: OS candidate window appears near the caret (not at a fixed screen corner).
3. While preedit is active:
   - Press `Space` to cycle candidates.
   - Press `Enter` to commit.
   - Press `Escape` to cancel.
   - Press `Tab` and confirm focus does not unexpectedly leave the input while composing.
4. After commit/cancel:
   - Press `Tab` and confirm focus traversal works again.

## Multiline textarea

1. Focus the multiline textarea.
2. Type multiple lines of ASCII text so the textarea can scroll.
3. Switch to Microsoft Pinyin and type `nihao` while the caret is:
   - near the top,
   - near the bottom,
   - after scrolling.
4. While composing:
   - Verify the candidate window stays near the caret as you move the caret or scroll.
5. Commit and cancel:
   - Verify commit replaces selection (if any) and clears the preedit state.
   - Verify cancel clears the preedit state without mutating the underlying model text.
