use super::*;

pub(in crate::ui) struct DocSection {
    pub title: &'static str,
    pub description: Vec<&'static str>,
    pub preview: AnyElement,
    pub code: Option<DocCodeBlock>,
    pub max_w: Px,
    pub test_id_prefix: Option<&'static str>,
}

pub(in crate::ui) struct DocCodeBlock {
    pub language: &'static str,
    pub code: &'static str,
}

impl DocSection {
    pub(in crate::ui) fn new(title: &'static str, preview: AnyElement) -> Self {
        Self {
            title,
            description: Vec::new(),
            preview,
            code: None,
            max_w: Px(820.0),
            test_id_prefix: None,
        }
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

    pub(in crate::ui) fn code(mut self, language: &'static str, code: &'static str) -> Self {
        self.code = Some(DocCodeBlock { language, code });
        self
    }

    pub(in crate::ui) fn max_w(mut self, max_w: Px) -> Self {
        self.max_w = max_w;
        self
    }

    pub(in crate::ui) fn test_id_prefix(mut self, test_id_prefix: &'static str) -> Self {
        self.test_id_prefix = Some(test_id_prefix);
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
                out.push(shadcn::typography::muted(cx, intro));
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

fn render_section(cx: &mut ElementContext<'_, App>, section: DocSection) -> AnyElement {
    let DocSection {
        title,
        description,
        preview,
        code,
        max_w,
        test_id_prefix,
    } = section;

    let preview_shell = demo_shell(cx, max_w, preview);
    let preview = centered(cx, preview_shell);

    let content = match code {
        Some(code) => preview_code_tabs(cx, test_id_prefix, preview, max_w, code),
        None => preview,
    };

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            let mut out: Vec<AnyElement> = Vec::with_capacity(3);
            out.push(section_title(cx, title));
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
                            .map(|line| shadcn::typography::muted(cx, line))
                            .collect::<Vec<_>>()
                    },
                );
                out.push(description_stack);
            }
            out.push(content);
            out
        },
    )
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
    cx.container(props, move |_cx| [body])
}

fn preview_code_tabs(
    cx: &mut ElementContext<'_, App>,
    test_id_prefix: Option<&'static str>,
    preview: AnyElement,
    max_w: Px,
    code: DocCodeBlock,
) -> AnyElement {
    let code_shell = code_block_shell(cx, max_w, code);
    let code_el = centered(cx, code_shell);

    let base = shadcn::Tabs::uncontrolled(Some("preview"))
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .shared_indicator_motion(true);

    let tabs = if let Some(prefix) = test_id_prefix {
        let tabs_test_id = format!("{prefix}-tabs");
        base.test_id(tabs_test_id.clone()).items([
            shadcn::TabsItem::new("preview", "Preview", [preview])
                .trigger_test_id(format!("{tabs_test_id}-trigger-preview")),
            shadcn::TabsItem::new("code", "Code", [code_el])
                .trigger_test_id(format!("{tabs_test_id}-trigger-code")),
        ])
    } else {
        base.items([
            shadcn::TabsItem::new("preview", "Preview", [preview]),
            shadcn::TabsItem::new("code", "Code", [code_el]),
        ])
    };

    tabs.into_element(cx)
}

fn code_block_shell(
    cx: &mut ElementContext<'_, App>,
    max_w: Px,
    block: DocCodeBlock,
) -> AnyElement {
    let code: Arc<str> = Arc::from(block.code);
    let copy = ui_ai::CodeBlockCopyButton::new(code.clone()).into_element(cx);
    let code_block = ui_ai::CodeBlock::new(code)
        .language(block.language)
        .show_header(true)
        .show_language(true)
        .header_right([copy])
        .into_element(cx);

    let props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            ChromeRefinement::default(),
            LayoutRefinement::default().w_full().min_w_0().max_w(max_w),
        )
    });
    cx.container(props, move |_cx| [code_block])
}

fn section_title(cx: &mut ElementContext<'_, App>, title: &'static str) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let style = fret_core::TextStyle {
        font: fret_core::FontId::default(),
        size: Px(20.0),
        weight: fret_core::FontWeight::SEMIBOLD,
        slant: fret_core::TextSlant::Normal,
        line_height: theme.metric_by_key("font.line_height"),
        letter_spacing_em: None,
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
    })
}
