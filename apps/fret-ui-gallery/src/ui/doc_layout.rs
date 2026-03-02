use super::*;

pub(in crate::ui) struct DocSection {
    pub title: &'static str,
    pub title_test_id: Option<&'static str>,
    pub description: Vec<&'static str>,
    pub preview: AnyElement,
    pub code: Option<DocCodeBlock>,
    pub tabs_sizing: DocTabsSizing,
    pub max_w: Px,
    pub test_id_prefix: Option<Arc<str>>,
    pub shell: bool,
}

/// Layout contract for the docs scaffold's Preview/Code tab panels.
///
/// - `Intrinsic` keeps tab panels content-sized by default (shrink-wrap).
/// - `FillRemaining` allows `TabsContent` to fill remaining main-axis space when the tabs root is
///   laid out under a definite-size budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(in crate::ui) enum DocTabsSizing {
    #[default]
    Intrinsic,
    FillRemaining,
}

pub(in crate::ui) struct DocCodeBlock {
    pub language: &'static str,
    pub code: Arc<str>,
}

impl DocSection {
    pub(in crate::ui) fn new(title: &'static str, preview: AnyElement) -> Self {
        Self {
            title,
            title_test_id: None,
            description: Vec::new(),
            preview,
            code: None,
            tabs_sizing: DocTabsSizing::default(),
            max_w: Px(820.0),
            test_id_prefix: None,
            shell: true,
        }
    }

    pub(in crate::ui) fn title_test_id(mut self, title_test_id: &'static str) -> Self {
        self.title_test_id = Some(title_test_id);
        self
    }

    pub(in crate::ui) fn description(mut self, description: &'static str) -> Self {
        self.description.push(description);
        self
    }

    pub(in crate::ui) fn descriptions(
        mut self,
        descriptions: impl IntoIterator<Item = &'static str>,
    ) -> Self {
        self.description.extend(descriptions);
        self
    }

    pub(in crate::ui) fn code(mut self, language: &'static str, code: impl Into<Arc<str>>) -> Self {
        self.code = Some(DocCodeBlock {
            language,
            code: code.into(),
        });
        self
    }

    pub(in crate::ui) fn code_rust(self, code: impl Into<Arc<str>>) -> Self {
        self.code("rust", code)
    }

    pub(in crate::ui) fn code_from_file(self, language: &'static str, file: &'static str) -> Self {
        self.code(language, Arc::<str>::from(file))
    }

    pub(in crate::ui) fn code_rust_from_file(self, file: &'static str) -> Self {
        self.code_from_file("rust", file)
    }

    pub(in crate::ui) fn code_from_file_region(
        self,
        language: &'static str,
        file: &'static str,
        region: &'static str,
    ) -> Self {
        let sliced = slice_code_region(file, region)
            .unwrap_or_else(|| format!("// region `{region}` not found\n{file}"));
        self.code(language, Arc::<str>::from(sliced))
    }

    pub(in crate::ui) fn code_rust_from_file_region(
        self,
        file: &'static str,
        region: &'static str,
    ) -> Self {
        self.code_from_file_region("rust", file, region)
    }

    /// Controls whether Preview/Code tabs should shrink-wrap their content (default) or fill any
    /// available main-axis space under definite-size ancestors (Tailwind-like `flex-1`).
    pub(in crate::ui) fn tabs_sizing(mut self, sizing: DocTabsSizing) -> Self {
        self.tabs_sizing = sizing;
        self
    }

    pub(in crate::ui) fn max_w(mut self, max_w: Px) -> Self {
        self.max_w = max_w;
        self
    }

    pub(in crate::ui) fn test_id_prefix(mut self, test_id_prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(test_id_prefix.into());
        self
    }

    pub(in crate::ui) fn no_shell(mut self) -> Self {
        self.shell = false;
        self
    }
}

pub(in crate::ui) fn render_doc_page(
    cx: &mut ElementContext<'_, App>,
    intro: Option<&'static str>,
    sections: Vec<DocSection>,
) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            let mut out: Vec<AnyElement> = Vec::with_capacity(sections.len() + 1);
            if let Some(intro) = intro {
                out.push(muted_full_width(cx, intro));
            }
            out.extend(
                sections
                    .into_iter()
                    .map(|section| render_section(cx, section)),
            );
            out
        },
    )
}

pub(in crate::ui) fn wrap_preview_page(
    cx: &mut ElementContext<'_, App>,
    intro: Option<&'static str>,
    section_title: &'static str,
    elements: Vec<AnyElement>,
) -> AnyElement {
    let preview = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4)
            .items_start(),
        |_cx| elements,
    );

    render_doc_page(
        cx,
        intro,
        vec![DocSection::new(section_title, preview)
            .no_shell()
            .max_w(Px(980.0))],
    )
}

/// A flex row that wraps on narrow widths.
///
/// Prefer this over `stack::hstack` for "control bars" that can contain many toggles/buttons.
pub(in crate::ui) fn wrap_row(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    gap: Space,
    align: fret_ui::element::CrossAlign,
    children: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) -> AnyElement {
    let gap = fret_ui_kit::MetricRef::space(gap).resolve(theme);
    let layout = decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0());
    cx.flex(
        fret_ui::element::FlexProps {
            layout,
            direction: fret_core::Axis::Horizontal,
            gap: gap.into(),
            padding: Edges::all(Px(0.0)).into(),
            justify: fret_ui::element::MainAlign::Start,
            align,
            wrap: true,
        },
        children,
    )
}

pub(in crate::ui) fn wrap_controls_row(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    gap: Space,
    children: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) -> AnyElement {
    wrap_row(
        cx,
        theme,
        gap,
        fret_ui::element::CrossAlign::Center,
        children,
    )
}

pub(in crate::ui) fn wrap_row_snapshot(
    cx: &mut ElementContext<'_, App>,
    theme: &fret_ui::ThemeSnapshot,
    gap: Space,
    align: fret_ui::element::CrossAlign,
    children: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) -> AnyElement {
    let gap = fret_ui_kit::MetricRef::space(gap).resolve(theme);
    let layout = decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0());
    cx.flex(
        fret_ui::element::FlexProps {
            layout,
            direction: fret_core::Axis::Horizontal,
            gap: gap.into(),
            padding: Edges::all(Px(0.0)).into(),
            justify: fret_ui::element::MainAlign::Start,
            align,
            wrap: true,
        },
        children,
    )
}

pub(in crate::ui) fn wrap_controls_row_snapshot(
    cx: &mut ElementContext<'_, App>,
    theme: &fret_ui::ThemeSnapshot,
    gap: Space,
    children: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) -> AnyElement {
    wrap_row_snapshot(
        cx,
        theme,
        gap,
        fret_ui::element::CrossAlign::Center,
        children,
    )
}

pub(in crate::ui) fn muted_full_width<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let (style, color) = {
        let theme = Theme::global(&*cx.app);
        let style = fret_ui_kit::typography::control_text_style(
            theme,
            fret_ui_kit::typography::UiTextSize::Xs,
        );
        let color = theme
            .color_by_key("muted-foreground")
            .or_else(|| theme.color_by_key("muted_foreground"))
            .unwrap_or_else(|| theme.color_token("foreground"));
        (style, color)
    };

    cx.text_props(TextProps {
        layout: {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.size.width = fret_ui::element::Length::Fill;
            layout
        },
        text: text.into(),
        style: Some(style),
        color: Some(color),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        ink_overflow: fret_ui::element::TextInkOverflow::None,
    })
}

fn muted_inline<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let (style, color) = {
        let theme = Theme::global(&*cx.app);
        let style = fret_ui_kit::typography::control_text_style(
            theme,
            fret_ui_kit::typography::UiTextSize::Xs,
        );
        let color = theme
            .color_by_key("muted-foreground")
            .or_else(|| theme.color_by_key("muted_foreground"))
            .unwrap_or_else(|| theme.color_token("foreground"));
        (style, color)
    };

    cx.text_props(TextProps {
        layout: fret_ui::element::LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        ink_overflow: fret_ui::element::TextInkOverflow::None,
    })
}

pub(in crate::ui) fn notes<I, T>(cx: &mut ElementContext<'_, App>, lines: I) -> AnyElement
where
    I: IntoIterator<Item = T>,
    T: Into<Arc<str>>,
{
    let lines = lines.into_iter().map(Into::into).collect::<Vec<Arc<str>>>();

    fn muted_flex_1_min_w_0<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        text: impl Into<Arc<str>>,
    ) -> AnyElement {
        let (style, color) = {
            let theme = Theme::global(&*cx.app);
            let style = fret_ui_kit::typography::control_text_style(
                theme,
                fret_ui_kit::typography::UiTextSize::Xs,
            );
            let color = theme
                .color_by_key("muted-foreground")
                .or_else(|| theme.color_by_key("muted_foreground"))
                .unwrap_or_else(|| theme.color_token("foreground"));
            (style, color)
        };

        cx.text_props(TextProps {
            layout: {
                let mut layout = fret_ui::element::LayoutStyle::default();
                layout.flex.grow = 1.0;
                layout.flex.shrink = 1.0;
                layout.size.min_width = Some(fret_ui::element::Length::Px(Px(0.0)));
                layout
            },
            text: text.into(),
            style: Some(style),
            color: Some(color),
            wrap: TextWrap::WordBreak,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        })
    }

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N1)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            lines
                .iter()
                .cloned()
                .map(|line| {
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N1)
                            .items_start()
                            .layout(LayoutRefinement::default().w_full().min_w_0()),
                        move |cx| [muted_inline(cx, "•"), muted_flex_1_min_w_0(cx, line)],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

pub(in crate::ui) fn rtl(
    cx: &mut ElementContext<'_, App>,
    f: impl FnOnce(&mut ElementContext<'_, App>) -> AnyElement,
) -> AnyElement {
    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        f,
    )
}

pub(in crate::ui) fn icon(cx: &mut ElementContext<'_, App>, id: &'static str) -> AnyElement {
    shadcn::icon::icon(cx, fret_icons::IconId::new_static(id))
}

#[allow(dead_code)]
pub(in crate::ui) fn gap_card(
    cx: &mut ElementContext<'_, App>,
    title: &'static str,
    details: &'static str,
    test_id: &'static str,
) -> (&'static str, AnyElement) {
    let alert_content = shadcn::Alert::new([
        icon(cx, "lucide.info"),
        shadcn::AlertTitle::new("Guide-aligned placeholder").into_element(cx),
        shadcn::AlertDescription::new(details).into_element(cx),
    ])
    .variant(shadcn::AlertVariant::Default)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(700.0)))
    .into_element(cx)
    .test_id(test_id);
    (title, alert_content)
}

fn render_section(cx: &mut ElementContext<'_, App>, section: DocSection) -> AnyElement {
    let DocSection {
        title,
        title_test_id,
        description,
        preview,
        code,
        tabs_sizing,
        max_w,
        test_id_prefix,
        shell,
    } = section;

    let has_code = code.is_some();
    let test_id_prefix = if has_code && test_id_prefix.is_none() {
        Some(auto_tabs_test_id_prefix(title, title_test_id))
    } else {
        test_id_prefix
    };

    let preview_shell = if shell {
        demo_shell(cx, max_w, preview)
    } else {
        layout_only_shell(cx, max_w, preview)
    };
    let preview = preview_shell;

    let content = match code {
        Some(code) => preview_code_tabs(
            cx,
            test_id_prefix.as_deref(),
            preview,
            max_w,
            code,
            tabs_sizing,
        ),
        None => preview,
    };

    let section_max_w = Px(max_w.0.max(Px(820.0).0));
    let section_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(
                LayoutRefinement::default()
                    .w_full()
                    .min_w_0()
                    .max_w(section_max_w),
            ),
        move |cx| {
            let mut out: Vec<AnyElement> = Vec::with_capacity(3);
            let title_el = section_title(cx, title);
            out.push(match (title_test_id, test_id_prefix.as_deref()) {
                (Some(test_id), _) => title_el.test_id(test_id),
                (None, Some(prefix)) => title_el.test_id(format!("{prefix}-title")),
                (None, None) => title_el,
            });
            if !description.is_empty() {
                let description_stack = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N1)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().min_w_0()),
                    move |cx| {
                        description
                            .into_iter()
                            .map(|line| muted_full_width(cx, line))
                            .collect::<Vec<_>>()
                    },
                );
                out.push(if let Some(prefix) = test_id_prefix.as_deref() {
                    description_stack.test_id(format!("{prefix}-description"))
                } else {
                    description_stack
                });
            }
            out.push(if let Some(prefix) = test_id_prefix.as_deref() {
                content.test_id(format!("{prefix}-content"))
            } else {
                content
            });
            out
        },
    );

    centered(cx, section_body)
}

fn centered(cx: &mut ElementContext<'_, App>, body: AnyElement) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .justify_center(),
        move |_cx| [body],
    )
}

fn demo_shell(cx: &mut ElementContext<'_, App>, max_w: Px, body: AnyElement) -> AnyElement {
    let props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .p(Space::N4),
            LayoutRefinement::default().w_full().min_w_0().max_w(max_w),
        )
    });
    cx.container(props, move |cx| [centered(cx, body)])
}

fn layout_only_shell(cx: &mut ElementContext<'_, App>, max_w: Px, body: AnyElement) -> AnyElement {
    let props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            ChromeRefinement::default(),
            LayoutRefinement::default().w_full().min_w_0().max_w(max_w),
        )
    });
    cx.container(props, move |_cx| [body])
}

fn auto_tabs_test_id_prefix(title: &'static str, title_test_id: Option<&'static str>) -> Arc<str> {
    if let Some(title_test_id) = title_test_id {
        let base = title_test_id
            .strip_suffix("-title")
            .unwrap_or(title_test_id);
        return Arc::from(base);
    }

    let slug = slugify_for_test_id(title);
    Arc::<str>::from(format!("docsec-{slug}"))
}

fn slugify_for_test_id(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_dash = false;

    for ch in input.chars() {
        let ch = ch.to_ascii_lowercase();
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
            continue;
        }

        if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }

    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}

fn preview_code_tabs(
    cx: &mut ElementContext<'_, App>,
    test_id_prefix: Option<&str>,
    preview: AnyElement,
    max_w: Px,
    code: DocCodeBlock,
    tabs_sizing: DocTabsSizing,
) -> AnyElement {
    let code_shell = code_block_shell(cx, test_id_prefix, max_w, code);
    let code_el = code_shell;

    let wrap_panel = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .items_start()
                .justify_center()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |_cx| vec![body],
        )
    };

    let base = shadcn::Tabs::uncontrolled(Some("preview"))
        .content_fill_remaining(matches!(tabs_sizing, DocTabsSizing::FillRemaining))
        .refine_layout(LayoutRefinement::default().w_full().min_w_0());

    let tabs = if let Some(prefix) = test_id_prefix {
        let tabs_test_id = format!("{prefix}-tabs");
        base.test_id(tabs_test_id.clone()).items([
            shadcn::TabsItem::new("preview", "Preview", [wrap_panel(cx, preview)])
                .trigger_test_id(format!("{tabs_test_id}-trigger-preview")),
            shadcn::TabsItem::new("code", "Code", [wrap_panel(cx, code_el)])
                .trigger_test_id(format!("{tabs_test_id}-trigger-code")),
        ])
    } else {
        base.items([
            shadcn::TabsItem::new("preview", "Preview", [wrap_panel(cx, preview)]),
            shadcn::TabsItem::new("code", "Code", [wrap_panel(cx, code_el)]),
        ])
    };

    tabs.into_element(cx)
}

fn code_block_shell(
    cx: &mut ElementContext<'_, App>,
    test_id_prefix: Option<&str>,
    max_w: Px,
    block: DocCodeBlock,
) -> AnyElement {
    let code: Arc<str> = block.code;
    let copy = match test_id_prefix {
        Some(prefix) => ui_ai::CodeBlockCopyButton::new(code.clone())
            .test_id(format!("{prefix}-code-block-copy"))
            .copied_marker_test_id(format!("{prefix}-code-block-copy-copied"))
            .into_element(cx),
        None => ui_ai::CodeBlockCopyButton::new(code.clone()).into_element(cx),
    };

    let mut code_block = ui_ai::CodeBlock::new(code)
        .language(block.language)
        .show_header(true)
        .show_language(true)
        .show_line_numbers(true)
        .max_height(Px(400.0))
        .windowed_lines(true)
        .header_right([copy])
        .into_element(cx);

    if let Some(prefix) = test_id_prefix {
        code_block = code_block.test_id(format!("{prefix}-code-block"));
    }

    let props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            // Match the Preview tab's comfortable padding so Code tabs don't look "flush-left"
            // compared to the demo shell.
            ChromeRefinement::default().p(Space::N4),
            LayoutRefinement::default().w_full().min_w_0().max_w(max_w),
        )
    });
    cx.container(props, move |_cx| [code_block])
}

fn slice_code_region(code: &str, region: &str) -> Option<String> {
    let mut inside = false;
    let mut out: Vec<&str> = Vec::new();

    for line in code.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("// region:") {
            inside = name.trim() == region;
            continue;
        }
        if trimmed == "// endregion" {
            if inside {
                break;
            }
            continue;
        }
        if let Some(name) = trimmed.strip_prefix("// endregion:") {
            if inside && (name.trim().is_empty() || name.trim() == region) {
                break;
            }
            continue;
        }
        if inside {
            out.push(line);
        }
    }

    if out.is_empty() {
        return None;
    }

    let mut joined = out.join("\n");
    joined.push('\n');
    Some(joined)
}

fn section_title(cx: &mut ElementContext<'_, App>, title: &'static str) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let style = fret_core::TextStyle {
        font: fret_core::FontId::default(),
        size: Px(20.0),
        weight: fret_core::FontWeight::SEMIBOLD,
        slant: fret_core::TextSlant::Normal,
        line_height: theme.metric_by_key("font.line_height"),
        line_height_policy: fret_core::TextLineHeightPolicy::FixedFromStyle,
        letter_spacing_em: None,
        ..Default::default()
    };

    cx.text_props(TextProps {
        layout: {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.size.width = fret_ui::element::Length::Fill;
            layout
        },
        text: Arc::from(title),
        style: Some(style),
        color: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: fret_core::TextAlign::Start,
        ink_overflow: fret_ui::element::TextInkOverflow::None,
    })
}
