use super::super::super::super::super::*;

pub(in crate::ui) fn preview_text_mixed_script_fallback(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct MixedScriptFallbackState {
        injected: bool,
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(MixedScriptFallbackState::default())),
        |st| st.clone(),
    );

    {
        let mut st = state.borrow_mut();
        if !st.injected {
            // Ensure the bundled default font set is registered even in "no system fonts" mode.
            let fonts = fret_fonts::default_fonts()
                .iter()
                .map(|b| b.to_vec())
                .collect::<Vec<_>>();
            cx.app
                .push_effect(fret_runtime::Effect::TextAddFonts { fonts });
            cx.app.request_redraw(cx.window);
            st.injected = true;
        }
    }

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: ensure mixed-script fallback stays tofu-free with bundled fonts."),
                cx.text("Tip: set FRET_TEXT_SYSTEM_FONTS=0 to validate the deterministic no-system-fonts path on native."),
                cx.text("This page injects the bundled default font set once, then renders a few coverage strings."),
            ]
        },
    );

    fn sample_row(
        cx: &mut ElementContext<'_, App>,
        theme: &Theme,
        label: &'static str,
        sample: &'static str,
        test_id: &'static str,
    ) -> AnyElement {
        let label = shadcn::FieldLabel::new(label).into_element(cx);
        let text = cx
            .text_props(TextProps {
                layout: Default::default(),
                text: Arc::from(sample),
                style: Some(TextStyle {
                    font: FontId::ui(),
                    size: Px(28.0),
                    ..Default::default()
                }),
                color: Some(theme.color_required("foreground")),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })
            .test_id(test_id);

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N1),
            |_cx| vec![label, text],
        )
    }

    let panel = cx
        .container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .bg(ColorRef::Color(theme.color_required("background")))
                    .p(Space::N4),
                LayoutRefinement::default().w_full(),
            ),
            |cx| {
                let latin = sample_row(
                    cx,
                    theme,
                    "Latin",
                    "m",
                    "ui-gallery-text-mixed-script-fallback-latin",
                );
                let cjk = sample_row(
                    cx,
                    theme,
                    "CJK",
                    "你",
                    "ui-gallery-text-mixed-script-fallback-cjk",
                );
                let emoji = sample_row(
                    cx,
                    theme,
                    "Emoji",
                    "\u{1F600}",
                    "ui-gallery-text-mixed-script-fallback-emoji",
                );
                let mixed = sample_row(
                    cx,
                    theme,
                    "Mixed",
                    "m你\u{1F600}",
                    "ui-gallery-text-mixed-script-fallback-mixed",
                );

                vec![stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .gap(Space::N4),
                    |_cx| vec![latin, cjk, emoji, mixed],
                )]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-text-mixed-script-fallback-root"),
        );

    vec![header, panel]
}
