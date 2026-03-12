use std::sync::Arc;

use fret_core::{
    AttributedText, Color, FontId, TextAlign, TextOverflow, TextSpan, TextStyle, TextWrap,
};
use fret_runtime::{
    CommandId, InputContext, InputDispatchPhase, KeymapService, Platform, PlatformCapabilities,
    WindowInputContextService, WindowKeyContextStackService, format_sequence,
};
use fret_ui::element::{AnyElement, StyledTextProps};
use fret_ui::{ElementContext, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ColorRef, LayoutRefinement, ui};

pub(crate) fn command_shortcut_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    command: &CommandId,
    platform: Platform,
) -> Option<Arc<str>> {
    let caps = cx
        .app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();

    let snapshot = cx
        .app
        .global::<WindowInputContextService>()
        .and_then(|svc| svc.snapshot(cx.window))
        .cloned();

    let mut base_ctx = snapshot.unwrap_or(InputContext {
        platform,
        caps: caps.clone(),
        ui_has_modal: false,
        window_arbitration: None,
        focus_is_text_input: false,
        text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
        edit_can_undo: true,
        edit_can_redo: true,
        router_can_back: false,
        router_can_forward: false,
        dispatch_phase: InputDispatchPhase::Bubble,
    });
    base_ctx.platform = platform;
    base_ctx.caps = caps;
    base_ctx.dispatch_phase = InputDispatchPhase::Bubble;

    let key_contexts: Vec<Arc<str>> = cx
        .app
        .global::<WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(cx.window))
        .map(|v| v.to_vec())
        .unwrap_or_default();

    let seq = cx
        .app
        .global::<KeymapService>()
        .and_then(|svc| {
            svc.keymap
                .display_shortcut_for_command_sequence_with_key_contexts(
                    &base_ctx,
                    &key_contexts,
                    command,
                )
        })
        .or_else(|| {
            cx.app.commands().get(command.clone()).and_then(|meta| {
                meta.default_keybindings
                    .iter()
                    .find(|kb| kb.platform.matches(platform))
                    .map(|kb| kb.sequence.clone())
            })
        })?;

    Some(Arc::from(format_sequence(platform, &seq)))
}

pub(crate) fn shortcut_text_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &ThemeSnapshot,
    text: Arc<str>,
    style: TextStyle,
    fg: Color,
    layout: LayoutRefinement,
) -> AnyElement {
    let rich = if shortcut_needs_symbol_font(text.as_ref()) {
        shortcut_symbol_font_for_platform(Platform::current()).map(|symbol_font| {
            shortcut_attributed_text_with_symbol_fallback(text.clone(), symbol_font)
        })
    } else {
        None
    };

    if let Some(rich) = rich {
        return cx.styled_text_props(StyledTextProps {
            layout: decl_style::layout_style(theme, layout),
            rich,
            style: Some(style),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: TextAlign::Start,
            ink_overflow: Default::default(),
        });
    }

    let mut out = ui::text(text)
        .layout(layout)
        .text_size_px(style.size)
        .font(style.font)
        .font_weight(style.weight)
        .nowrap()
        .text_color(ColorRef::Color(fg));

    if let Some(line_height) = style.line_height {
        out = out.fixed_line_box_px(line_height).line_box_in_bounds();
    }

    if let Some(letter_spacing_em) = style.letter_spacing_em {
        out = out.letter_spacing_em(letter_spacing_em);
    }

    out.into_element(cx)
}

pub(crate) fn shortcut_needs_symbol_font(text: &str) -> bool {
    text.chars().any(shortcut_is_symbol_char)
}

fn shortcut_is_symbol_char(ch: char) -> bool {
    matches!(ch, '⌘' | '⌥' | '⌃' | '⇧' | '⌫')
}

pub(crate) fn shortcut_symbol_font_for_platform(platform: Platform) -> Option<FontId> {
    match platform {
        Platform::Macos => Some(FontId::family("Apple Symbols")),
        Platform::Windows => Some(FontId::family("Segoe UI Symbol")),
        Platform::Linux | Platform::Web => None,
    }
}

fn shortcut_attributed_text_with_symbol_fallback(
    text: Arc<str>,
    symbol_font: FontId,
) -> AttributedText {
    if text.is_empty() {
        return AttributedText::new(text, Arc::<[TextSpan]>::from([]));
    }

    let mut spans: Vec<TextSpan> = Vec::new();
    let mut run_start = 0usize;
    let mut run_is_symbol: Option<bool> = None;

    for (idx, ch) in text.char_indices() {
        let is_symbol = shortcut_is_symbol_char(ch);
        match run_is_symbol {
            None => {
                run_start = idx;
                run_is_symbol = Some(is_symbol);
            }
            Some(prev) if prev != is_symbol => {
                let mut span = TextSpan::new(idx.saturating_sub(run_start));
                if prev {
                    span.shaping.font = Some(symbol_font.clone());
                }
                spans.push(span);
                run_start = idx;
                run_is_symbol = Some(is_symbol);
            }
            _ => {}
        }
    }

    let end = text.len();
    let prev = run_is_symbol.unwrap_or(false);
    let mut span = TextSpan::new(end.saturating_sub(run_start));
    if prev {
        span.shaping.font = Some(symbol_font);
    }
    spans.push(span);

    AttributedText::new(text, Arc::<[TextSpan]>::from(spans))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_symbol_detection_covers_menu_glyphs() {
        assert!(shortcut_needs_symbol_font("⌘P"));
        assert!(shortcut_needs_symbol_font("⌥⇧K"));
        assert!(shortcut_needs_symbol_font("⌘⌫"));
        assert!(!shortcut_needs_symbol_font("Cmd+P"));
    }

    #[test]
    fn shortcut_symbol_font_selection_covers_macos_and_windows() {
        assert_eq!(
            shortcut_symbol_font_for_platform(Platform::Macos),
            Some(FontId::family("Apple Symbols"))
        );
        assert_eq!(
            shortcut_symbol_font_for_platform(Platform::Windows),
            Some(FontId::family("Segoe UI Symbol"))
        );
    }

    #[test]
    fn shortcut_attributed_text_uses_valid_utf8_span_boundaries() {
        let rich = shortcut_attributed_text_with_symbol_fallback(
            Arc::<str>::from("⌘P"),
            FontId::family("Apple Symbols"),
        );
        assert!(rich.is_valid());
    }
}
