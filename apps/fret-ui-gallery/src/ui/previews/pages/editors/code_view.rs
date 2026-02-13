use super::super::super::super::*;

pub(in crate::ui) fn code_view_torture_source() -> Arc<str> {
    static SOURCE: OnceLock<Arc<str>> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            let mut out = String::new();
            out.push_str("// Code View Torture Harness\n");
            out.push_str("// Generated content: large line count + long lines\n\n");
            for i in 0..8_000 {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "{i:05}: fn example_{i}() {{ let x = {i}; let y = x.wrapping_mul(31); }}\n"
                    ),
                );
            }
            Arc::<str>::from(out)
        })
        .clone()
}

pub(in crate::ui) fn preview_code_view_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress large scrollable code/text surfaces (candidate for prepaint-windowed lines)."),
                cx.text("Use scripted wheel steps + stale-paint checks to validate scroll stability."),
            ]
        },
    );

    let code = code_view_torture_source();

    let windowed =
        match std::env::var_os("FRET_UI_GALLERY_CODE_VIEW_WINDOWED").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => true,
        };

    let block = code_view::CodeBlock::new(code)
        .language("rust")
        .show_line_numbers(true)
        .windowed_lines(windowed)
        .show_scrollbar_y(true)
        .max_height(Px(420.0));
    let block = block.into_element(cx);

    let block = block.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-code-view-root"),
    );

    vec![header, block]
}

pub(in crate::ui) fn code_editor_mvp_source() -> String {
    [
        "// Code Editor MVP\n",
        "// Goals:\n",
        "// - Validate TextInputRegion focus + TextInput/Ime events\n",
        "// - Validate nested scrolling (editor owns its own scroll)\n",
        "// - Provide a base surface for code-editor-ecosystem-v1 workstream\n",
        "\n",
        "fn main() {\n",
        "    let mut sum = 0u64;\n",
        "    for i in 0..10_000 {\n",
        "        sum = sum.wrapping_add(i);\n",
        "    }\n",
        "    println!(\"sum={}\", sum);\n",
        "}\n",
        "\n",
        "struct Point { x: f32, y: f32 }\n",
        "\n",
        "impl Point {\n",
        "    fn len(&self) -> f32 {\n",
        "        (self.x * self.x + self.y * self.y).sqrt()\n",
        "    }\n",
        "}\n",
        "\n",
        "// Try: mouse drag selection, Ctrl+C/Ctrl+V, arrows, Backspace/Delete, IME.\n",
    ]
    .concat()
}

pub(in crate::ui) fn code_editor_torture_source() -> String {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            let mut out = String::new();
            out.push_str("// Code Editor Torture Harness\n");
            out.push_str("// Generated content: many lines + deterministic prefixes\n\n");
            for i in 0..20_000usize {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "{i:05}: let value_{i} = {i}; // scrolling should never show stale lines\n"
                    ),
                );
            }
            out
        })
        .clone()
}

pub(in crate::ui) fn code_editor_word_boundary_fixture() -> String {
    [
        "// Word boundary fixture (UI Gallery)\n",
        "\n",
        "世界 hello 😀 foo123_bar baz foo.bar\n",
        "a_b c\t  hello   world\n",
        "αβγ δ\n",
    ]
    .concat()
}

pub(in crate::ui) fn format_word_boundary_debug(text: &str, idx: usize) -> String {
    let idx = code_editor_view::clamp_to_char_boundary(text, idx).min(text.len());
    fn move_n_chars_left(text: &str, mut idx: usize, n: usize) -> usize {
        for _ in 0..n {
            let prev = code_editor_view::prev_char_boundary(text, idx);
            if prev == idx {
                break;
            }
            idx = prev;
        }
        idx
    }

    fn move_n_chars_right(text: &str, mut idx: usize, n: usize) -> usize {
        for _ in 0..n {
            let next = code_editor_view::next_char_boundary(text, idx);
            if next == idx {
                break;
            }
            idx = next;
        }
        idx
    }

    fn sanitize_inline(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for ch in s.chars() {
            match ch {
                '\n' => out.push('⏎'),
                '\t' => out.push('⇥'),
                '\r' => out.push('␍'),
                _ => out.push(ch),
            }
        }
        out
    }

    let ctx_start = move_n_chars_left(text, idx, 16);
    let ctx_end = move_n_chars_right(text, idx, 16);
    let ctx_start = code_editor_view::clamp_to_char_boundary(text, ctx_start).min(text.len());
    let ctx_end = code_editor_view::clamp_to_char_boundary(text, ctx_end).min(text.len());
    let ctx_before = sanitize_inline(text.get(ctx_start..idx).unwrap_or(""));
    let ctx_after = sanitize_inline(text.get(idx..ctx_end).unwrap_or(""));
    let caret_ch = text.get(idx..).and_then(|s| s.chars().next());
    let caret_ch = caret_ch.map(|c| sanitize_inline(&c.to_string()));

    let unicode = fret_runtime::TextBoundaryMode::UnicodeWord;
    let ident = fret_runtime::TextBoundaryMode::Identifier;

    let (u_a, u_b) = code_editor_view::select_word_range(text, idx, unicode);
    let (i_a, i_b) = code_editor_view::select_word_range(text, idx, ident);

    let u_l = code_editor_view::move_word_left(text, idx, unicode);
    let u_r = code_editor_view::move_word_right(text, idx, unicode);
    let i_l = code_editor_view::move_word_left(text, idx, ident);
    let i_r = code_editor_view::move_word_right(text, idx, ident);

    [
        format!(
            "idx={idx} caret_char={}",
            caret_ch.as_deref().unwrap_or("<eof>")
        ),
        format!("context: {ctx_before}|{ctx_after}"),
        format!("UnicodeWord: select={u_a}..{u_b} left={u_l} right={u_r}"),
        format!("Identifier: select={i_a}..{i_b} left={i_l} right={i_r}"),
    ]
    .join("\n")
}
