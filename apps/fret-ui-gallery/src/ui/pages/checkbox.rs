use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::checkbox as snippets;

pub(super) fn preview_checkbox(
    cx: &mut ElementContext<'_, App>,
    model: Model<bool>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct CheckboxModels {
        checked_controlled: Option<Model<bool>>,
        checked_optional: Option<Model<Option<bool>>>,
        invalid: Option<Model<bool>>,
        description: Option<Model<bool>>,
        disabled: Option<Model<bool>>,
        with_title: Option<Model<bool>>,
        group_security: Option<Model<bool>>,
        group_updates: Option<Model<bool>>,
        group_marketing: Option<Model<bool>>,
        table_all: Option<Model<bool>>,
        table_row_1: Option<Model<bool>>,
        table_row_2: Option<Model<bool>>,
        table_row_3: Option<Model<bool>>,
        rtl: Option<Model<bool>>,
    }

    let (
        checked_controlled,
        checked_optional,
        invalid,
        description,
        disabled,
        with_title,
        group_security,
        group_updates,
        group_marketing,
        table_all,
        table_row_1,
        table_row_2,
        table_row_3,
        rtl,
    ) = cx.with_state(CheckboxModels::default, |st| {
        (
            st.checked_controlled.clone(),
            st.checked_optional.clone(),
            st.invalid.clone(),
            st.description.clone(),
            st.disabled.clone(),
            st.with_title.clone(),
            st.group_security.clone(),
            st.group_updates.clone(),
            st.group_marketing.clone(),
            st.table_all.clone(),
            st.table_row_1.clone(),
            st.table_row_2.clone(),
            st.table_row_3.clone(),
            st.rtl.clone(),
        )
    });

    let (
        checked_controlled,
        checked_optional,
        invalid,
        description,
        disabled,
        with_title,
        group_security,
        group_updates,
        group_marketing,
        table_all,
        table_row_1,
        table_row_2,
        table_row_3,
        rtl,
    ) = match (
        checked_controlled,
        checked_optional,
        invalid,
        description,
        disabled,
        with_title,
        group_security,
        group_updates,
        group_marketing,
        table_all,
        table_row_1,
        table_row_2,
        table_row_3,
        rtl,
    ) {
        (
            Some(checked_controlled),
            Some(checked_optional),
            Some(invalid),
            Some(description),
            Some(disabled),
            Some(with_title),
            Some(group_security),
            Some(group_updates),
            Some(group_marketing),
            Some(table_all),
            Some(table_row_1),
            Some(table_row_2),
            Some(table_row_3),
            Some(rtl),
        ) => (
            checked_controlled,
            checked_optional,
            invalid,
            description,
            disabled,
            with_title,
            group_security,
            group_updates,
            group_marketing,
            table_all,
            table_row_1,
            table_row_2,
            table_row_3,
            rtl,
        ),
        _ => {
            let checked_controlled = cx.app.models_mut().insert(true);
            let checked_optional = cx.app.models_mut().insert(None);
            let invalid = cx.app.models_mut().insert(false);
            let description = cx.app.models_mut().insert(false);
            let disabled = cx.app.models_mut().insert(true);
            let with_title = cx.app.models_mut().insert(true);
            let group_security = cx.app.models_mut().insert(true);
            let group_updates = cx.app.models_mut().insert(false);
            let group_marketing = cx.app.models_mut().insert(false);
            let table_all = cx.app.models_mut().insert(false);
            let table_row_1 = cx.app.models_mut().insert(true);
            let table_row_2 = cx.app.models_mut().insert(false);
            let table_row_3 = cx.app.models_mut().insert(false);
            let rtl = cx.app.models_mut().insert(true);

            cx.with_state(CheckboxModels::default, |st| {
                st.checked_controlled = Some(checked_controlled.clone());
                st.checked_optional = Some(checked_optional.clone());
                st.invalid = Some(invalid.clone());
                st.description = Some(description.clone());
                st.disabled = Some(disabled.clone());
                st.with_title = Some(with_title.clone());
                st.group_security = Some(group_security.clone());
                st.group_updates = Some(group_updates.clone());
                st.group_marketing = Some(group_marketing.clone());
                st.table_all = Some(table_all.clone());
                st.table_row_1 = Some(table_row_1.clone());
                st.table_row_2 = Some(table_row_2.clone());
                st.table_row_3 = Some(table_row_3.clone());
                st.rtl = Some(rtl.clone());
            });

            (
                checked_controlled,
                checked_optional,
                invalid,
                description,
                disabled,
                with_title,
                group_security,
                group_updates,
                group_marketing,
                table_all,
                table_row_1,
                table_row_2,
                table_row_3,
                rtl,
            )
        }
    };

    let demo = snippets::demo::render(cx, model.clone());
    let checked_state =
        snippets::checked_state::render(cx, checked_controlled.clone(), checked_optional.clone());
    let invalid_state = snippets::invalid_state::render(cx, invalid.clone());
    let basic = snippets::basic::render(cx, model.clone());
    let description_section = snippets::description::render(cx, description.clone());
    let disabled_section = snippets::disabled::render(cx, disabled.clone());
    let with_title_section = snippets::with_title::render(cx, with_title.clone());
    let group = snippets::group::render(
        cx,
        group_security.clone(),
        group_updates.clone(),
        group_marketing.clone(),
    );
    let table = snippets::table::render(
        cx,
        table_all.clone(),
        table_row_1.clone(),
        table_row_2.clone(),
        table_row_3.clone(),
    );
    let rtl_section = snippets::rtl::render(cx, rtl.clone());

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/checkbox.rs` (Checkbox).",
            "Use Field composition (FieldLabel/FieldDescription) to keep label, helper text, and toggle target aligned.",
            "For indeterminate behavior, prefer `Checkbox::new_optional(Model<Option<bool>>)`, where `None` maps to mixed state.",
            "Table selection patterns should keep row-level and header-level states explicit; avoid hidden coupling in demos.",
            "When validating parity, test both keyboard focus ring and RTL label alignment in addition to pointer clicks.",
        ],
    )
    .test_id("ui-gallery-checkbox-notes");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Checkbox docs flow: Demo -> Checked State -> Invalid State -> Basic -> Description -> Disabled -> With Title -> Group -> Table -> RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Single checkbox with a label.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Checked State", checked_state)
                .description("Controlled checked model and optional/indeterminate model.")
                .code_rust_from_file_region(snippets::checked_state::SOURCE, "example"),
            DocSection::new("Invalid State", invalid_state)
                .description("Invalid styling uses `aria_invalid` on the checkbox and destructive label text.")
                .code_rust_from_file_region(snippets::invalid_state::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("Field + checkbox + label composition.")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Description", description_section)
                .description("FieldContent keeps label and helper text aligned with the control.")
                .code_rust_from_file_region(snippets::description::SOURCE, "example"),
            DocSection::new("Disabled", disabled_section)
                .description("Disabled checkbox should block interaction and use muted styling.")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("With Title", with_title_section)
                .description("FieldLabel can wrap a full Field layout (card-style label).")
                .code_rust_from_file_region(snippets::with_title::SOURCE, "example"),
            DocSection::new("Group", group)
                .description("Checkbox group pattern with per-item descriptions.")
                .code_rust_from_file_region(snippets::group::SOURCE, "example"),
            DocSection::new("Table", table)
                .description("Table selection pattern with header and row checkboxes.")
                .code_rust_from_file_region(snippets::table::SOURCE, "example"),
            DocSection::new("RTL", rtl_section)
                .description("Checkbox + label alignment under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("API reference pointers and parity notes."),
        ],
    );

    vec![body.test_id("ui-gallery-checkbox")]
}
