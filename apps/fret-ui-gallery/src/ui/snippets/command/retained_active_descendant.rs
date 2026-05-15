pub const SOURCE: &str = include_str!("retained_active_descendant.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::{Point, Px, SemanticsRole};
use fret_ui::element::{
    LayoutStyle, Length, Overflow, PressableA11y, PressableProps, SemanticsProps, TextInputProps,
    VirtualListOptions,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

const LEN: usize = 48;
const ACTIVE_INDEX: usize = 2;
const AWAY_OFFSET_Y: f32 = 180.0;
const ROW_HEIGHT: f32 = 20.0;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let disabled_model = cx.local_model_keyed("retained_active_disabled", || false);
    let input_model = cx.local_model_keyed("retained_active_input", String::new);
    let scroll_handle = cx.slot_state(VirtualListScrollHandle::new, |h| h.clone());
    let active_element = cx.slot_state(
        || Rc::new(Cell::new(None::<GlobalElementId>)),
        |state| state.clone(),
    );

    let active_disabled = cx
        .app
        .models()
        .read(&disabled_model, |value| *value)
        .unwrap_or(false);

    let reset = {
        let disabled_model = disabled_model.clone();
        let scroll_handle = scroll_handle.clone();
        Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  _reason: fret_ui::action::ActivateReason| {
                scroll_handle.set_offset(Point::new(Px(0.0), Px(0.0)));
                let _ = host.models_mut().update(&disabled_model, |value| {
                    *value = false;
                });
                host.request_redraw(action_cx.window);
            },
        ) as fret_ui::action::OnActivate
    };

    let scroll_away = {
        let scroll_handle = scroll_handle.clone();
        Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  _reason: fret_ui::action::ActivateReason| {
                scroll_handle.set_offset(Point::new(Px(0.0), Px(AWAY_OFFSET_Y)));
                host.request_redraw(action_cx.window);
            },
        ) as fret_ui::action::OnActivate
    };

    let disable_active = {
        let disabled_model = disabled_model.clone();
        Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  _reason: fret_ui::action::ActivateReason| {
                let _ = host.models_mut().update(&disabled_model, |value| {
                    *value = true;
                });
                host.request_redraw(action_cx.window);
            },
        ) as fret_ui::action::OnActivate
    };

    let scroll_back = {
        let scroll_handle = scroll_handle.clone();
        Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  _reason: fret_ui::action::ActivateReason| {
                scroll_handle.set_offset(Point::new(Px(0.0), Px(0.0)));
                host.request_redraw(action_cx.window);
            },
        ) as fret_ui::action::OnActivate
    };

    let active_element_for_row = active_element.clone();
    let row = move |cx: &mut AppComponentCx<'_>, index: usize| {
        let is_active = index == ACTIVE_INDEX;
        let disabled = active_disabled && is_active;
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Px(Px(ROW_HEIGHT));
        layout.overflow = Overflow::Clip;

        let label = format!("Retained row {index}");
        let row = cx.pressable_with_id(
            PressableProps {
                layout,
                enabled: !disabled,
                focusable: false,
                a11y: PressableA11y {
                    role: Some(SemanticsRole::ListBoxOption),
                    label: Some(Arc::<str>::from(label.clone())),
                    test_id: Some(Arc::<str>::from(retained_row_test_id(index))),
                    selected: is_active,
                    pos_in_set: Some(index.saturating_add(1) as u32),
                    set_size: Some(LEN as u32),
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, _st, _id| {
                vec![
                    ui::text(label.clone())
                        .text_sm()
                        .into_element(cx)
                        .test_id(format!("{}-label", retained_row_test_id(index))),
                ]
            },
        );
        if is_active {
            active_element_for_row.set(Some(row.id));
        }
        row
    };

    let mut list_layout = LayoutStyle::default();
    list_layout.size.width = Length::Fill;
    list_layout.size.height = Length::Px(Px(64.0));
    list_layout.overflow = Overflow::Clip;

    let mut options = VirtualListOptions::known(Px(ROW_HEIGHT), 0, |_index| Px(ROW_HEIGHT));
    options.keep_alive = 16;
    options.items_revision = u64::from(active_disabled);

    let listbox_id_out: Cell<Option<GlobalElementId>> = Cell::new(None);
    let listbox = cx.semantics_with_id(
        SemanticsProps {
            role: SemanticsRole::ListBox,
            test_id: Some(Arc::from("ui-gallery-command-retained-active-listbox")),
            ..Default::default()
        },
        |cx, id| {
            listbox_id_out.set(Some(id));
            vec![cx.virtual_list_keyed_retained_with_layout_fn(
                list_layout,
                LEN,
                options,
                &scroll_handle,
                |index| index as fret_ui::ItemKey,
                row,
            )]
        },
    );

    let mut input_props = TextInputProps::new(input_model);
    input_props.layout.size.width = Length::Fill;
    input_props.layout.size.height = Length::Px(Px(28.0));
    input_props.test_id = Some(Arc::from("ui-gallery-command-retained-active-input"));
    input_props.a11y_role = Some(SemanticsRole::ComboBox);
    input_props.a11y_label = Some(Arc::from("Retained active descendant search"));
    input_props.controls_element = listbox_id_out.get().map(|id| id.0);
    input_props.active_descendant_element = active_element.get().map(|id| id.0);
    let input = cx.text_input(input_props);

    let status = ui::text(if active_disabled {
        "Active row disabled"
    } else {
        "Active row enabled"
    })
    .text_sm()
    .into_element(cx)
    .test_id("ui-gallery-command-retained-active-status");

    ui::v_flex(|cx| {
        vec![
            ui::h_flex(|cx| {
                vec![
                    shadcn::Button::new("Reset")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(reset.clone())
                        .test_id("ui-gallery-command-retained-active-reset")
                        .into_element(cx),
                    shadcn::Button::new("Scroll Away")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(scroll_away.clone())
                        .test_id("ui-gallery-command-retained-active-scroll-away")
                        .into_element(cx),
                    shadcn::Button::new("Disable Active")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(disable_active.clone())
                        .test_id("ui-gallery-command-retained-active-disable")
                        .into_element(cx),
                    shadcn::Button::new("Scroll Back")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(scroll_back.clone())
                        .test_id("ui-gallery-command-retained-active-scroll-back")
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .wrap()
            .into_element(cx),
            input,
            listbox,
            status,
        ]
    })
    .gap(Space::N2)
    .layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-command-retained-active-root")
}

fn retained_row_test_id(index: usize) -> String {
    format!("ui-gallery-command-retained-active-row-{index}")
}
// endregion: example
