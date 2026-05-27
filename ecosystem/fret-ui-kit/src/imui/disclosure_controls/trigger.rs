use fret_ui::element::{AnyElement, LayoutStyle, Length, PressableProps, SizeStyle};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{spec::DisclosureSpec, visual};
use crate::primitives::collapsible as radix_collapsible;

mod behavior;

pub(super) fn disclosure_header_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    spec: DisclosureSpec,
    open_model: fret_runtime::Model<bool>,
    content_id: GlobalElementId,
    open_now: bool,
    enabled: bool,
    trigger_response: &mut super::super::ResponseExt,
) -> AnyElement {
    cx.named("header", |cx| {
        let spec_for_pressable = spec.clone();
        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled;
        props.layout = LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        };
        props.a11y = visual::disclosure_a11y(&spec, open_now);

        let mut header = cx.pressable_with_id(props, move |cx, state, trigger_id| {
            let spec = spec_for_pressable.clone();
            let action_label = spec.label.clone();
            behavior::install_disclosure_trigger_behavior(
                cx,
                &state,
                trigger_id,
                &spec,
                open_model.clone(),
                enabled,
                trigger_response,
            );

            vec![visual::header_row(cx, &spec, action_label, open_now, state)]
        });

        if spec.has_children() {
            header = radix_collapsible::apply_collapsible_trigger_controls_expanded(
                header, content_id, open_now,
            );
        }
        if let Some(test_id) = spec.header_test_id.as_ref() {
            header = header.test_id(test_id.clone());
        }
        header
    })
}
