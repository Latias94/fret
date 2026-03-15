pub const SOURCE: &str = include_str!("dialog.rs");

// region: example
use std::sync::Arc;

use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::action::OnActivate;
use fret_ui_kit::{ColorRef, WidgetStateProperty};
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut UiCx<'_>, last_action: Model<Arc<str>>) -> impl UiChild + use<> {
    let default_dialog = material3::Dialog::uncontrolled(cx);
    let open = default_dialog.open_model();
    let override_open = cx.local_model_keyed("override_open", || false);

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let override_is_open = cx
        .get_model_copied(&override_open, Invalidation::Layout)
        .unwrap_or(false);

    let open_dialog: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let close_dialog: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let confirm_action: OnActivate = {
        let open = open.clone();
        let last_action = last_action.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from("material3.dialog.confirm")
            });
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };

    let theme = cx.theme().clone();
    let select_items: Arc<[material3::SelectItem]> = (0..20)
        .map(|i| {
            material3::SelectItem::new(
                Arc::<str>::from(format!("item-{i:02}")),
                Arc::<str>::from(format!("Item {i:02}")),
            )
            .test_id(format!("ui-gallery-material3-dialog-select-item-{i:02}"))
        })
        .collect::<Vec<_>>()
        .into();
    let override_style = material3::DialogStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_token("md.sys.color.secondary-container"),
        ))))
        .headline_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_token("md.sys.color.on-secondary-container"),
        ))))
        .supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_token("md.sys.color.on-secondary-container"),
        ))));

    let open_dialog_override: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            let _ = host.models_mut().update(&override_open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };
    let close_dialog_override: OnActivate = {
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let confirm_action_override: OnActivate = {
        let override_open = override_open.clone();
        let last_action = last_action.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from("material3.dialog.confirm.override")
            });
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };

    let build_dialog = |cx: &mut UiCx<'_>,
                        mut dialog: material3::Dialog,
                        style: Option<material3::DialogStyle>,
                        id_prefix: &'static str,
                        open_action: OnActivate,
                        close_action: OnActivate,
                        confirm_action: OnActivate| {
        dialog = dialog
            .headline("Discard draft?")
            .supporting_text("This action cannot be undone.")
            .actions(vec![
                material3::DialogAction::new("Cancel")
                    .test_id(format!("{id_prefix}-action-cancel"))
                    .on_activate(close_action.clone()),
                material3::DialogAction::new("Discard")
                    .test_id(format!("{id_prefix}-action-discard"))
                    .on_activate(confirm_action.clone()),
            ])
            .test_id(format!("{id_prefix}"));

        if let Some(style) = style {
            dialog = dialog.style(style);
        }

        dialog.into_element(
            cx,
            move |cx| {
                ui::v_flex(move |cx| {
                        vec![
                            material3::Button::new("Open dialog")
                                .variant(material3::ButtonVariant::Filled)
                                .on_activate(open_action.clone())
                                .test_id(format!("{id_prefix}-open"))
                                .into_element(cx),
                            material3::Button::new("Underlay focus probe")
                                .variant(material3::ButtonVariant::Outlined)
                                .test_id(format!("{id_prefix}-underlay-probe"))
                                .into_element(cx),
                            cx.text("Tip: press Esc or click the scrim to close; Tab should stay inside the dialog while open."),
                        ]
                    })
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N4).into_element(cx)
            },
            {
                let select_items = select_items.clone();
                move |cx| {
                    let spacer = cx.container(
                        fret_ui::element::ContainerProps {
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.size.width = fret_ui::element::Length::Fill;
                                l.size.height = fret_ui::element::Length::Px(Px(480.0));
                                l
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::<AnyElement>::new(),
                    );

                    vec![ui::v_flex(move |cx| {
                            vec![
                                material3::Select::uncontrolled(cx)
                                    .a11y_label("Material 3 Select (dialog)")
                                    .placeholder("Pick one")
                                    .items(select_items.clone())
                                    .match_anchor_width(false)
                                    .test_id(format!("{id_prefix}-select"))
                                    .into_element(cx),
                                cx.text(
                                    "Bottom-edge clamping probe: open the Select menu near the window bottom.",
                                ),
                                spacer,
                                material3::Select::uncontrolled(cx)
                                    .a11y_label("Material 3 Select (dialog, bottom)")
                                    .placeholder("Pick one")
                                    .items(select_items.clone())
                                    .match_anchor_width(false)
                                    .test_id(format!("{id_prefix}-select-bottom"))
                                    .into_element(cx),
                            ]
                        })
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .gap(Space::N4).into_element(cx)]
                }
            },
        )
    };

    let default_dialog = build_dialog(
        cx,
        default_dialog,
        None,
        "ui-gallery-material3-dialog",
        open_dialog.clone(),
        close_dialog.clone(),
        confirm_action.clone(),
    );
    let override_dialog = build_dialog(
        cx,
        material3::Dialog::new(override_open.clone()),
        Some(override_style),
        "ui-gallery-material3-dialog-override",
        open_dialog_override.clone(),
        close_dialog_override.clone(),
        confirm_action_override.clone(),
    );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let build_container = |dialog| {
        let mut layout = fret_ui::element::LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Px(Px(360.0));
        ui::container_props(
            fret_ui::element::ContainerProps {
                layout,
                ..Default::default()
            },
            move |_cx| [dialog],
        )
    };

    let containers = ui::h_row(move |cx| {
        ui::children![cx; build_container(default_dialog), build_container(override_dialog)]
    })
    .gap(Space::N4)
    .items_center()
    .into_element(cx);

    ui::v_flex(|cx| {
            vec![
                cx.text(
                    "Material 3 Dialog: modal barrier + focus trap/restore + token-shaped dialog actions.",
                ),
                containers,
                cx.text(format!(
                    "open={} override_open={} last_action={}",
                    is_open as u8,
                    override_is_open as u8,
                    last.as_ref()
                )),
            ]
        })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start().into_element(cx)
    .into()
}

// endregion: example
