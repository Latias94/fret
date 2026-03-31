use super::super::super::super::super::*;
use crate::ui::doc_layout;
use fret::UiChild;
use fret::UiCx;

pub(in crate::ui) fn preview_text_feature_toggles(
    cx: &mut UiCx<'_>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct FeatureTogglesState {
        liga: bool,
        calt: bool,
        ss01: bool,
    }

    let state = cx.slot_state(
        || std::rc::Rc::new(std::cell::RefCell::new(FeatureTogglesState::default())),
        |st| st.clone(),
    );

    let _ = crate::driver::ensure_ui_gallery_default_profile_fonts_present(cx.app, cx.window);

    let header = ui::v_flex(|cx| {
        vec![
            cx.text(
                "Goal: validate OpenType feature overrides (`TextShapingStyle.features`) end-to-end.",
            ),
            cx.text(
                "This is best-effort: visible differences depend on the chosen font. Inter typically shows `liga` (fi/fl/ffi/ffl).",
            ),
            cx.text(
                "Tip: set FRET_TEXT_SYSTEM_FONTS=0 to validate the deterministic no-system-fonts path on native.",
            ),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N2);

    fn toggle_button(
        cx: &mut UiCx<'_>,
        label: &'static str,
        value: bool,
        test_id: &'static str,
        on_activate: fret_ui::action::OnActivate,
    ) -> impl UiChild + use<> {
        let _ = cx;
        let txt = format!("{label}: {}", if value { "on" } else { "off" });
        shadcn::Button::new(txt)
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .on_activate(on_activate)
            .test_id(test_id)
    }

    let toolbar = {
        let state_liga = state.clone();
        let on_liga: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let mut st = state_liga.borrow_mut();
            st.liga = !st.liga;
            host.request_redraw(action_cx.window);
        });

        let state_calt = state.clone();
        let on_calt: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let mut st = state_calt.borrow_mut();
            st.calt = !st.calt;
            host.request_redraw(action_cx.window);
        });

        let state_ss01 = state.clone();
        let on_ss01: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let mut st = state_ss01.borrow_mut();
            st.ss01 = !st.ss01;
            host.request_redraw(action_cx.window);
        });

        let st = state.borrow();
        let liga_btn = toggle_button(
            cx,
            "liga",
            st.liga,
            "ui-gallery-text-feature-toggles-liga",
            on_liga,
        );
        let calt_btn = toggle_button(
            cx,
            "calt",
            st.calt,
            "ui-gallery-text-feature-toggles-calt",
            on_calt,
        );
        let ss01_btn = toggle_button(
            cx,
            "ss01",
            st.ss01,
            "ui-gallery-text-feature-toggles-ss01",
            on_ss01,
        );

        ui::h_flex(|cx| ui::children![cx; liga_btn, calt_btn, ss01_btn])
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2)
            .items_center()
    };

    fn sample_text(
        cx: &mut UiCx<'_>,
        theme: &Theme,
        label: &'static str,
        text: &'static str,
        features: Option<fret_core::TextShapingStyle>,
        test_id: &'static str,
    ) -> impl UiChild + use<> {
        let label = shadcn::FieldLabel::new(label).into_element(cx);

        let span = if let Some(shaping) = features {
            TextSpan {
                len: text.len(),
                shaping,
                paint: fret_core::TextPaintStyle {
                    fg: Some(theme.color_token("foreground")),
                    ..Default::default()
                },
            }
        } else {
            TextSpan {
                len: text.len(),
                shaping: Default::default(),
                paint: fret_core::TextPaintStyle {
                    fg: Some(theme.color_token("foreground")),
                    ..Default::default()
                },
            }
        };

        let rich = AttributedText::new(Arc::<str>::from(text), Arc::<[TextSpan]>::from([span]));

        let mut props = fret_ui::element::SelectableTextProps::new(rich);
        props.style = Some(TextStyle {
            font: FontId::family("Inter"),
            size: Px(30.0),
            ..Default::default()
        });
        props.wrap = TextWrap::None;
        props.overflow = TextOverflow::Clip;
        props.layout.size.width = fret_ui::element::Length::Fill;

        let text = cx.selectable_text_props(props).test_id(test_id);

        ui::v_flex(|_cx| vec![label, text])
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N1)
    }

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_token("background")))
                .p(Space::N4),
            LayoutRefinement::default().w_full(),
        ),
        |cx| {
            let sample = "office affine offline (fi fl ffi ffl)  -> => != <= >= ===";

            let baseline = sample_text(
                cx,
                theme,
                "Baseline (no explicit features)",
                sample,
                None,
                "ui-gallery-text-feature-toggles-baseline",
            )
            .into_element(cx);

            let st = state.borrow();
            let shaping = fret_core::TextShapingStyle::default()
                .with_feature("liga", if st.liga { 1 } else { 0 })
                .with_feature("calt", if st.calt { 1 } else { 0 })
                .with_feature("ss01", if st.ss01 { 1 } else { 0 });

            let overridden = sample_text(
                cx,
                theme,
                "Overrides (`TextShapingStyle.features`)",
                sample,
                Some(shaping),
                "ui-gallery-text-feature-toggles-overrides",
            )
            .into_element(cx);

            vec![
                ui::v_flex(|cx| ui::children![cx; toolbar, baseline, overridden])
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N4)
                    .into_element(cx),
            ]
        },
    );

    let header = header.into_element(cx);
    let page = doc_layout::wrap_preview_page(cx, None, "Text feature toggles", vec![header, panel]);

    vec![page.into_element(cx)]
}
