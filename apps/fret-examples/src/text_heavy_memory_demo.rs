use fret::app::prelude::*;
use fret_core::{Px, TextAlign, TextOverflow, TextStyle, TextWrap};
use fret_ui::ElementContext;
use fret_ui::element::{
    ContainerProps, LayoutStyle, Length, ScrollProps, SizeStyle, SpacingEdges, SpacingLength,
};
use std::sync::Arc;

struct TextHeavyMemoryView {
    content: Arc<str>,
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("text-heavy-memory-demo")
        .window("text_heavy_memory_demo", (980.0, 720.0))
        .setup(fret_bootstrap::install_default_i18n_backend)
        .view::<TextHeavyMemoryView>()?
        .run()
        .map_err(anyhow::Error::from)
}

impl View for TextHeavyMemoryView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self {
            content: build_text_heavy_content(),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        view(cx.elements(), &self.content)
    }
}

fn view(cx: &mut ElementContext<'_, App>, content: &Arc<str>) -> Ui {
    let scroll = cx.scroll(
        ScrollProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        |cx| {
            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: SpacingEdges::all(SpacingLength::Px(Px(16.0))),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.text_props(fret_ui::element::TextProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        text: content.clone(),
                        style: Some(TextStyle {
                            size: Px(16.0),
                            ..Default::default()
                        }),
                        color: None,
                        wrap: TextWrap::Word,
                        overflow: TextOverflow::Clip,
                        align: TextAlign::Start,
                        ink_overflow: Default::default(),
                    })]
                },
            )]
        },
    );

    vec![scroll].into()
}

fn build_text_heavy_content() -> Arc<str> {
    const EMOJI_PER_RANGE: usize = 256;
    const EMOJI_PER_LINE: usize = 64;
    const VS16: char = '\u{FE0F}';

    let mut out = String::new();

    out.push_str("Text heavy memory demo\n");
    out.push_str(
        "This demo exists to grow text shaping caches and (on macOS) the color emoji atlas.\n",
    );
    out.push('\n');
    out.push_str("ASCII: The quick brown fox jumps over the lazy dog. 0123456789\n");
    out.push_str("CJK: 你好，世界！日本語のテキストです。한국어 텍스트입니다。\n");
    out.push_str("RTL: العربية نص تجريبي لاختبار تشكيل النص واتجاهه.\n");
    out.push_str("Indic: नमस्ते दुनिया — देवनागरी लिपि परीक्षण।\n");
    out.push('\n');
    out.push_str("Emoji blocks (with VS16):\n");

    let ranges: &[(u32, u32)] = &[
        (0x1F300, 0x1F5FF), // Misc Symbols and Pictographs
        (0x1F600, 0x1F64F), // Emoticons
        (0x1F680, 0x1F6FF), // Transport and Map
        (0x1F900, 0x1F9FF), // Supplemental Symbols and Pictographs
        (0x1FA70, 0x1FAFF), // Symbols and Pictographs Extended-A
    ];

    for (start, end) in ranges {
        let mut count = 0usize;
        let mut line_count = 0usize;
        for cp in *start..=*end {
            if count >= EMOJI_PER_RANGE {
                break;
            }
            let Some(ch) = char::from_u32(cp) else {
                continue;
            };
            out.push(ch);
            out.push(VS16);
            out.push(' ');
            count += 1;
            line_count += 1;
            if line_count >= EMOJI_PER_LINE {
                out.push('\n');
                line_count = 0;
            }
        }
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }

    Arc::from(out)
}
