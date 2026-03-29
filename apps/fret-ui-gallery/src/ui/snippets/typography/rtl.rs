pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn story_block(child: impl UiChild, margin_top: Option<Px>) -> impl UiChild {
    let mut block = ui::v_flex(move |cx| vec![child.into_element(cx)])
        .layout(LayoutRefinement::default().w_full());
    if let Some(margin_top) = margin_top {
        block = block.mt_px(margin_top);
    }
    block.items_start()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        ui::v_flex(|cx| {
            vec![
                story_block(
                    shadcn::raw::typography::h1("فرض الضرائب على الضحك: سجلات ضريبة النكتة"),
                    None,
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::lead(
                        "في قديم الزمان، في أرض بعيدة، كان هناك ملك كسول جداً يقضي يومه كله مستلقياً على عرشه. في أحد الأيام، جاءه مستشاروه بمشكلة: المملكة كانت تنفد من المال.",
                    ),
                    Some(Px(24.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::h2("خطة الملك"),
                    Some(Px(40.0)),
                )
                .into_element(cx),
            story_block(
                shadcn::raw::typography::p_rich([
                    shadcn::raw::typography::inline_text(
                        "فكر الملك طويلاً وبجد، وأخيراً توصل إلى ",
                    ),
                    shadcn::raw::typography::inline_link("خطة عبقرية", "#"),
                    shadcn::raw::typography::inline_text(
                        ": سيفرض ضريبة على النكات في المملكة.",
                    ),
                ]),
                Some(Px(24.0)),
            )
            .into_element(cx),
                story_block(
                    shadcn::raw::typography::blockquote(
                        "\"في النهاية،\" قال، \"الجميع يستمتع بنكتة جيدة، لذا من العدل أن يدفعوا مقابل هذا الامتياز.\"",
                    ),
                    Some(Px(24.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::h3("ضريبة النكتة"),
                    Some(Px(32.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::p(
                        "لم يكن رعايا الملك سعداء. تذمروا واشتكوا، لكن الملك كان حازماً:",
                    ),
                    Some(Px(24.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::list([
                        "المستوى الأول من التورية: 5 قطع ذهبية",
                        "المستوى الثاني من النكات: 10 قطع ذهبية",
                        "المستوى الثالث من النكات القصيرة: 20 قطعة ذهبية",
                    ]),
                    Some(Px(24.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::p(
                        "نتيجة لذلك، توقف الناس عن رواية النكات، وغرقت المملكة في الكآبة. لكن كان هناك شخص واحد رفض أن تحبطه حماقة الملك: مهرج البلاط المسمى المازح.",
                    ),
                    Some(Px(24.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::h3("ثورة المازح"),
                    Some(Px(32.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::p(
                        "بدأ المازح يتسلل إلى القلعة في منتصف الليل ويترك النكات في كل مكان: تحت وسادة الملك، في حسائه، حتى في المرحاض الملكي. كان الملك غاضباً، لكنه لم يستطع إيقاف المازح.",
                    ),
                    Some(Px(24.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::p(
                        "وبعد ذلك، في يوم من الأيام، اكتشف سكان المملكة أن النكات التي تركها المازح كانت مضحكة جداً لدرجة أنهم لم يستطيعوا منع أنفسهم من الضحك. وبمجرد أن بدأوا بالضحك، لم يستطيعوا التوقف.",
                    ),
                    Some(Px(24.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::h3("ثورة الشعب"),
                    Some(Px(32.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::p(
                        "شعر سكان المملكة بالبهجة من الضحك، وبدأوا في رواية النكات والتورية مرة أخرى، وسرعان ما أصبحت المملكة بأكملها جزءاً من النكتة.",
                    ),
                    Some(Px(24.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::table(|cx| {
                        ui::children![
                            cx;
                            shadcn::table_header(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::table_row(2, |cx| {
                                        ui::children![
                                            cx;
                                            shadcn::table_head("خزينة الملك"),
                                            shadcn::table_head("سعادة الشعب"),
                                        ]
                                    })
                                    .border_bottom(true),
                                ]
                            }),
                            shadcn::table_body(|cx| {
                                vec![
                                    shadcn::table_row(2, |cx| {
                                        ui::children![
                                            cx;
                                            shadcn::table_cell(ui::text("فارغة")),
                                            shadcn::table_cell(ui::text("فائضة")),
                                        ]
                                    })
                                    .into_element(cx),
                                    shadcn::table_row(2, |cx| {
                                        ui::children![
                                            cx;
                                            shadcn::table_cell(ui::text("متواضعة")),
                                            shadcn::table_cell(ui::text("راضٍ")),
                                        ]
                                    })
                                    .into_element(cx),
                                    shadcn::table_row(2, |cx| {
                                        ui::children![
                                            cx;
                                            shadcn::table_cell(ui::text("ممتلئة")),
                                            shadcn::table_cell(ui::text("منتشٍ")),
                                        ]
                                    })
                                    .into_element(cx),
                                ]
                            }),
                        ]
                    })
                    .refine_layout(LayoutRefinement::default().w_full()),
                    Some(Px(24.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::p(
                        "الملك، عندما رأى مدى سعادة رعاياه، أدرك خطأ طرقه وألغى ضريبة النكتة. أُعلن المازح بطلاً، وعاشت المملكة في سعادة دائمة.",
                    ),
                    Some(Px(24.0)),
                )
                .into_element(cx),
                story_block(
                    shadcn::raw::typography::p(
                        "مغزى القصة هو: لا تستهن أبداً بقوة الضحك الجيد وكن دائماً حذراً من الأفكار السيئة.",
                    ),
                    Some(Px(24.0)),
                )
                .into_element(cx),
            ]
        })
        .items_start()
        .layout(LayoutRefinement::default().w_full())
        .into_element(cx)
    })
    .test_id("ui-gallery-typography-rtl")
}
// endregion: example
