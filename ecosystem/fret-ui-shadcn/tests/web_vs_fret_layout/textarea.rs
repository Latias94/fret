use super::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LayoutTextareaRecipe {
    Demo,
    Disabled,
    WithButton,
    WithLabel,
    WithText,
}

#[derive(Debug, Clone, Deserialize)]
struct LayoutTextareaCase {
    id: String,
    web_name: String,
    recipe: LayoutTextareaRecipe,
}

#[test]
fn web_vs_fret_layout_textarea_geometry_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/layout_textarea_cases_v1.json"
    ));
    let suite: FixtureSuite<LayoutTextareaCase> =
        serde_json::from_str(raw).expect("layout textarea fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("layout textarea case={}", case.id);
        let web = read_web_golden(&case.web_name);
        let theme = web_theme(&web);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
        );

        match case.recipe {
            LayoutTextareaRecipe::Demo => {
                let web_textarea =
                    find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea");
                let snap = run_fret_root(bounds, |cx| {
                    let model: Model<String> = cx.app.models_mut().insert(String::new());
                    vec![
                        fret_ui_shadcn::Textarea::new(model)
                            .a11y_label("Textarea")
                            .into_element(cx),
                    ]
                });

                let textarea = find_semantics(&snap, SemanticsRole::TextField, Some("Textarea"))
                    .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
                    .expect("fret textarea semantics node");

                assert_close_px(
                    "textarea width",
                    textarea.bounds.size.width,
                    web_textarea.rect.w,
                    1.0,
                );
                assert_close_px(
                    "textarea height",
                    textarea.bounds.size.height,
                    web_textarea.rect.h,
                    1.0,
                );
            }
            LayoutTextareaRecipe::Disabled => {
                let web_textarea =
                    find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea");
                let snap = run_fret_root(bounds, |cx| {
                    let model: Model<String> = cx.app.models_mut().insert(String::new());
                    vec![
                        fret_ui_shadcn::Textarea::new(model)
                            .a11y_label("Textarea")
                            .disabled(true)
                            .into_element(cx),
                    ]
                });

                let textarea = find_semantics(&snap, SemanticsRole::TextField, Some("Textarea"))
                    .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
                    .expect("fret textarea semantics node");

                assert_close_px(
                    "textarea-disabled x",
                    textarea.bounds.origin.x,
                    web_textarea.rect.x,
                    1.0,
                );
                assert_close_px(
                    "textarea-disabled y",
                    textarea.bounds.origin.y,
                    web_textarea.rect.y,
                    1.0,
                );
                assert_close_px(
                    "textarea-disabled w",
                    textarea.bounds.size.width,
                    web_textarea.rect.w,
                    1.0,
                );
                assert_close_px(
                    "textarea-disabled h",
                    textarea.bounds.size.height,
                    web_textarea.rect.h,
                    1.0,
                );
            }
            LayoutTextareaRecipe::WithButton => {
                let web_textarea =
                    find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea");
                let web_button =
                    find_first(&theme.root, &|n| n.tag == "button").expect("web button");
                let gap = web_button.rect.y - (web_textarea.rect.y + web_textarea.rect.h);

                let snap = run_fret_root(bounds, |cx| {
                    let model: Model<String> = cx.app.models_mut().insert(String::new());
                    let textarea = fret_ui_shadcn::Textarea::new(model)
                        .a11y_label("Textarea")
                        .into_element(cx);
                    let button = fret_ui_shadcn::Button::new("Send message")
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx);

                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Vertical,
                            gap: Px(gap),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                        },
                        move |_cx| vec![textarea, button],
                    )]
                });

                let textarea = find_semantics(&snap, SemanticsRole::TextField, Some("Textarea"))
                    .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
                    .expect("fret textarea semantics node");
                let button = find_semantics(&snap, SemanticsRole::Button, Some("Send message"))
                    .or_else(|| find_semantics(&snap, SemanticsRole::Button, None))
                    .expect("fret button semantics node");

                assert_close_px(
                    "textarea-with-button textarea x",
                    textarea.bounds.origin.x,
                    web_textarea.rect.x,
                    1.0,
                );
                assert_close_px(
                    "textarea-with-button textarea y",
                    textarea.bounds.origin.y,
                    web_textarea.rect.y,
                    1.0,
                );
                assert_close_px(
                    "textarea-with-button textarea w",
                    textarea.bounds.size.width,
                    web_textarea.rect.w,
                    1.0,
                );
                assert_close_px(
                    "textarea-with-button textarea h",
                    textarea.bounds.size.height,
                    web_textarea.rect.h,
                    1.0,
                );

                assert_close_px(
                    "textarea-with-button button x",
                    button.bounds.origin.x,
                    web_button.rect.x,
                    1.0,
                );
                assert_close_px(
                    "textarea-with-button button y",
                    button.bounds.origin.y,
                    web_button.rect.y,
                    1.0,
                );
                assert_close_px(
                    "textarea-with-button button w",
                    button.bounds.size.width,
                    web_button.rect.w,
                    1.0,
                );
                assert_close_px(
                    "textarea-with-button button h",
                    button.bounds.size.height,
                    web_button.rect.h,
                    1.0,
                );
            }
            LayoutTextareaRecipe::WithLabel => {
                let web_label = find_first(&theme.root, &|n| n.tag == "label").expect("web label");
                let web_textarea =
                    find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea");
                let gap = web_textarea.rect.y - (web_label.rect.y + web_label.rect.h);

                let snap = run_fret_root(bounds, |cx| {
                    let model: Model<String> = cx.app.models_mut().insert(String::new());
                    let label = fret_ui_shadcn::Label::new("Your message").into_element(cx);
                    let textarea = fret_ui_shadcn::Textarea::new(model)
                        .a11y_label("Textarea")
                        .into_element(cx);

                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Vertical,
                            gap: Px(gap),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                        },
                        move |_cx| vec![label, textarea],
                    )]
                });

                let textarea = find_semantics(&snap, SemanticsRole::TextField, Some("Textarea"))
                    .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
                    .expect("fret textarea semantics node");

                assert_close_px(
                    "textarea-with-label textarea x",
                    textarea.bounds.origin.x,
                    web_textarea.rect.x,
                    1.0,
                );
                assert_close_px(
                    "textarea-with-label textarea y",
                    textarea.bounds.origin.y,
                    web_textarea.rect.y,
                    1.0,
                );
                assert_close_px(
                    "textarea-with-label textarea w",
                    textarea.bounds.size.width,
                    web_textarea.rect.w,
                    1.0,
                );
                assert_close_px(
                    "textarea-with-label textarea h",
                    textarea.bounds.size.height,
                    web_textarea.rect.h,
                    1.0,
                );
            }
            LayoutTextareaRecipe::WithText => {
                let web_textarea =
                    find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea");
                let web_p = find_first(&theme.root, &|n| n.tag == "p").expect("web text");

                let mut services = StyleAwareServices::default();
                let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
                    let theme = Theme::global(&*cx.app).clone();
                    let model: Model<String> = cx.app.models_mut().insert(String::new());
                    let label = fret_ui_shadcn::Label::new("Your Message").into_element(cx);
                    let textarea = fret_ui_shadcn::Textarea::new(model)
                        .a11y_label("Textarea")
                        .into_element(cx);
                    let helper = ui::text(cx, "Your message will be copied to the support team.")
                        .text_size_px(theme.metric_required("font.size"))
                        .line_height_px(theme.metric_required("font.line_height"))
                        .font_normal()
                        .into_element(cx);
                    let helper = cx.semantics(
                        fret_ui::element::SemanticsProps {
                            role: SemanticsRole::Panel,
                            label: Some(Arc::from("Golden:textarea-with-text:helper")),
                            ..Default::default()
                        },
                        move |_cx| vec![helper],
                    );

                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Vertical,
                            gap: Px(12.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                        },
                        move |_cx| vec![label, textarea, helper],
                    )]
                });

                let textarea = find_semantics(&snap, SemanticsRole::TextField, Some("Textarea"))
                    .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
                    .expect("fret textarea semantics node");
                let helper = find_semantics(
                    &snap,
                    SemanticsRole::Panel,
                    Some("Golden:textarea-with-text:helper"),
                )
                .expect("fret helper wrapper");

                assert_close_px(
                    "textarea-with-text textarea y",
                    textarea.bounds.origin.y,
                    web_textarea.rect.y,
                    1.0,
                );
                assert_close_px(
                    "textarea-with-text textarea h",
                    textarea.bounds.size.height,
                    web_textarea.rect.h,
                    1.0,
                );

                assert_close_px(
                    "textarea-with-text helper y",
                    helper.bounds.origin.y,
                    web_p.rect.y,
                    1.0,
                );
                assert_close_px(
                    "textarea-with-text helper h",
                    helper.bounds.size.height,
                    web_p.rect.h,
                    1.0,
                );
            }
        }
    }
}
