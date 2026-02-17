use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_label(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct LabelPageModels {
        demo_email: Option<Model<String>>,
        field_email: Option<Model<String>>,
        rtl_name: Option<Model<String>>,
    }

    let (demo_email, field_email, rtl_name) = cx.with_state(LabelPageModels::default, |st| {
        (
            st.demo_email.clone(),
            st.field_email.clone(),
            st.rtl_name.clone(),
        )
    });

    let (demo_email, field_email, rtl_name) = match (demo_email, field_email, rtl_name) {
        (Some(demo_email), Some(field_email), Some(rtl_name)) => {
            (demo_email, field_email, rtl_name)
        }
        _ => {
            let demo_email = cx.app.models_mut().insert(String::new());
            let field_email = cx.app.models_mut().insert(String::new());
            let rtl_name = cx.app.models_mut().insert(String::new());

            cx.with_state(LabelPageModels::default, |st| {
                st.demo_email = Some(demo_email.clone());
                st.field_email = Some(field_email.clone());
                st.rtl_name = Some(rtl_name.clone());
            });

            (demo_email, field_email, rtl_name)
        }
    };

    let max_w = LayoutRefinement::default().w_full().max_w(Px(420.0));

    let demo = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(max_w.clone()),
            |cx| {
                vec![
                    shadcn::Label::new("Your email address").into_element(cx),
                    shadcn::Input::new(demo_email)
                        .placeholder("you@example.com")
                        .a11y_label("Email")
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-label-demo");
        content
    };

    let label_in_field = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(max_w.clone()),
            |cx| {
                vec![
                    shadcn::typography::muted(
                        cx,
                        "For forms, prefer Field + FieldLabel for built-in description/error structure.",
                    ),
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("Work email").into_element(cx),
                        shadcn::Input::new(field_email)
                            .placeholder("name@company.com")
                            .a11y_label("Work email")
                            .into_element(cx),
                        shadcn::FieldDescription::new("We use this email for notifications.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-label-field");
        content
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(max_w.clone()),
            |cx| {
                vec![
                    shadcn::Label::new("????? ??????").into_element(cx),
                    shadcn::Input::new(rtl_name)
                        .placeholder("???? ???")
                        .a11y_label("????? ??????")
                        .into_element(cx),
                ]
            },
        )
    })
    .test_id("ui-gallery-label-rtl");

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/label.rs` (Label) and `ecosystem/fret-ui-shadcn/src/field.rs` (FieldLabel).",
            "Label is a lightweight text primitive; form semantics and helper/error text live in `Field`.",
            "Current Label API does not expose `htmlFor` binding; accessibility is handled by control a11y labels and Field composition.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Label docs order: Demo, Label in Field, RTL."),
        vec![
            DocSection::new("Demo", demo)
                .description("Basic label above an input.")
                .code(
                    "rust",
                    r#"shadcn::Label::new("Your email address").into_element(cx);"#,
                ),
            DocSection::new("Label in Field", label_in_field)
                .description("Prefer Field + FieldLabel for form layouts.")
                .code(
                    "rust",
                    r#"shadcn::Field::new([
    shadcn::FieldLabel::new("Work email").into_element(cx),
    shadcn::Input::new(model).into_element(cx),
    shadcn::FieldDescription::new("...").into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Label and input alignment under an RTL direction provider.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        shadcn::Label::new("????? ??????").into_element(cx);
        shadcn::Input::new(model).a11y_label("????? ??????").into_element(cx);
    },
);"#,
                ),
            DocSection::new("Notes", notes).description("API reference pointers and caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-label")]
}
