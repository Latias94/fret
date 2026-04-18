pub const SOURCE: &str = include_str!("select.rs");

// region: example
use std::sync::Arc;

use fret::{AppComponentCx, UiChild};
use fret_core::{Corners, Edges, Point, Px, Transform2D};
use fret_ui::action::OnActivate;
use fret_ui::element::{ContainerProps, LayoutStyle, Length, Overflow};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let default_select = material3::Select::uncontrolled(cx);
    let selected = default_select.value_model();
    let menu_width_floor_enabled = cx.local_model_keyed("menu_width_floor_enabled", || true);
    let menu_width_floor_enabled_now = cx
        .get_model_copied(&menu_width_floor_enabled, Invalidation::Layout)
        .unwrap_or(true);

    let typeahead_delay_ms = cx.local_model_keyed("typeahead_delay_ms", || 200u32);
    let typeahead_delay_ms_now = cx
        .get_model_copied(&typeahead_delay_ms, Invalidation::Layout)
        .unwrap_or(200);

    let items: Arc<[material3::SelectItem]> = vec![
        material3::SelectItem::new("alpha", "Alpha").test_id("ui-gallery-material3-select-a"),
        material3::SelectItem::new("beta", "Beta").test_id("ui-gallery-material3-select-b"),
        material3::SelectItem::new("charlie", "Charlie (disabled)")
            .disabled(true)
            .test_id("ui-gallery-material3-select-c-disabled"),
    ]
    .into();

    let default = default_select
        .clone()
        .a11y_label("Material 3 Select")
        .placeholder("Pick one")
        .items(items.clone())
        .test_id("ui-gallery-material3-select")
        .into_element(cx);

    let (primary, primary_container, secondary_container) = cx.with_theme(|theme| {
        (
            theme.color_token("md.sys.color.primary"),
            theme.color_token("md.sys.color.primary-container"),
            theme.color_token("md.sys.color.secondary-container"),
        )
    });

    let override_style = material3::SelectStyle::default()
        .container_background(
            WidgetStateProperty::new(None)
                .when(WidgetStates::OPEN, Some(ColorRef::Color(primary_container))),
        )
        .outline_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::FOCUS_VISIBLE, Some(ColorRef::Color(primary))),
        )
        .trailing_icon_color(
            WidgetStateProperty::new(None).when(WidgetStates::OPEN, Some(ColorRef::Color(primary))),
        )
        .menu_selected_container_color(WidgetStateProperty::new(Some(ColorRef::Color(
            secondary_container,
        ))));

    let overridden = material3::Select::new(selected.clone())
        .a11y_label("Material 3 Select (override)")
        .placeholder("Pick one")
        .items(items)
        .style(override_style)
        .test_id("ui-gallery-material3-select-overridden")
        .into_element(cx);

    let unclamped_items: Arc<[material3::SelectItem]> = vec![
        material3::SelectItem::new("short", "Short")
            .test_id("ui-gallery-material3-select-unclamped-item-short"),
        material3::SelectItem::new("medium", "Medium option")
            .test_id("ui-gallery-material3-select-unclamped-item-medium"),
        material3::SelectItem::new(
            "long",
            "A very long option label that should expand the menu beyond the anchor width",
        )
        .test_id("ui-gallery-material3-select-unclamped-item-long"),
        material3::SelectItem::new("long2", "Another long-ish label for measuring menu width")
            .test_id("ui-gallery-material3-select-unclamped-item-long2"),
        material3::SelectItem::new(
            "xl",
            "Extra long: The quick brown fox jumps over the lazy dog",
        )
        .test_id("ui-gallery-material3-select-unclamped-item-xl"),
    ]
    .into();

    let unclamped = material3::Select::uncontrolled(cx)
        .a11y_label("Material 3 Select (unclamped menu width)")
        .placeholder("Unclamped")
        .items(unclamped_items)
        .match_anchor_width(false)
        .menu_width_floor(if menu_width_floor_enabled_now {
            Px(210.0)
        } else {
            Px(0.0)
        })
        .typeahead_delay_ms(typeahead_delay_ms_now)
        .test_id("ui-gallery-material3-select-unclamped")
        .into_element(cx);

    let floor_toggle = material3::Switch::new(menu_width_floor_enabled.clone())
        .a11y_label("Select menu width floor (210px)")
        .test_id("ui-gallery-material3-select-menu-width-floor-toggle")
        .into_element(cx);

    let typeahead_items: Arc<[material3::SelectItem]> = vec![
        material3::SelectItem::new("beta", "Beta")
            .test_id("ui-gallery-material3-select-typeahead-item-beta"),
        material3::SelectItem::new("charlie", "Charlie (disabled)")
            .disabled(true)
            .test_id("ui-gallery-material3-select-typeahead-item-charlie-disabled"),
        material3::SelectItem::new("delta", "Delta")
            .test_id("ui-gallery-material3-select-typeahead-item-delta"),
        material3::SelectItem::new("echo", "Echo")
            .test_id("ui-gallery-material3-select-typeahead-item-echo"),
    ]
    .into();

    let set_delay_button = |cx: &mut AppComponentCx<'_>, ms: u32| {
        let delay_model = typeahead_delay_ms.clone();
        let on_activate: OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&delay_model, |v| *v = ms);
            host.request_redraw(action_cx.window);
        });

        material3::Button::new(format!("{ms}ms"))
            .variant(if typeahead_delay_ms_now == ms {
                material3::ButtonVariant::Filled
            } else {
                material3::ButtonVariant::Outlined
            })
            .test_id(format!("ui-gallery-material3-select-typeahead-delay-{ms}"))
            .on_activate(on_activate)
            .into_element(cx)
    };

    let typeahead_select = material3::Select::uncontrolled(cx)
        .a11y_label("Material 3 Select (typeahead delay)")
        .placeholder("Typeahead probe")
        .items(typeahead_items)
        .typeahead_delay_ms(typeahead_delay_ms_now)
        .test_id("ui-gallery-material3-select-typeahead")
        .into_element(cx);

    let rich_items: Arc<[material3::SelectItem]> = vec![
        material3::SelectItem::new("alpha", "Alpha")
            .supporting_text("Supporting: quick summary")
            .trailing_supporting_text("⌘A")
            .leading_icon(fret_icons::ids::ui::SEARCH)
            .test_id("ui-gallery-material3-select-rich-item-alpha"),
        material3::SelectItem::new("beta", "Beta")
            .supporting_text("Supporting: secondary line")
            .trailing_supporting_text("⌘B")
            .leading_icon(fret_icons::ids::ui::SETTINGS)
            .test_id("ui-gallery-material3-select-rich-item-beta"),
        material3::SelectItem::new("charlie", "Charlie (disabled)")
            .supporting_text("Disabled items are skipped by typeahead/roving")
            .disabled(true)
            .leading_icon(fret_icons::ids::ui::SLASH)
            .test_id("ui-gallery-material3-select-rich-item-charlie-disabled"),
        material3::SelectItem::new("delta", "Delta")
            .supporting_text("Trailing-only still aligns")
            .trailing_supporting_text("⌘D")
            .test_id("ui-gallery-material3-select-rich-item-delta"),
    ]
    .into();

    let rich_select = material3::Select::uncontrolled(cx)
        .a11y_label("Material 3 Select (supporting text options)")
        .placeholder("Option richness probe")
        .items(rich_items)
        .typeahead_delay_ms(typeahead_delay_ms_now)
        .test_id("ui-gallery-material3-select-rich")
        .into_element(cx);

    let transformed_items: Arc<[material3::SelectItem]> = vec![
        material3::SelectItem::new("alpha", "Alpha")
            .test_id("ui-gallery-material3-select-transformed-item-alpha"),
        material3::SelectItem::new("beta", "Beta")
            .test_id("ui-gallery-material3-select-transformed-item-beta"),
        material3::SelectItem::new("gamma", "Gamma")
            .test_id("ui-gallery-material3-select-transformed-item-gamma"),
    ]
    .into();

    let transformed_select = material3::Select::uncontrolled(cx)
        .a11y_label("Material 3 Select (transformed)")
        .placeholder("Transformed")
        .items(transformed_items)
        .test_id("ui-gallery-material3-select-transformed")
        .into_element(cx);

    let (probe_bg, probe_border) = cx.with_theme(|theme| {
        let bg = theme
            .color_by_key("md.sys.color.surface-container")
            .or_else(|| theme.color_by_key("md.sys.color.surface"))
            .unwrap_or(fret_core::Color::TRANSPARENT);
        let border = theme
            .color_by_key("md.sys.color.outline-variant")
            .unwrap_or(fret_core::Color::TRANSPARENT);
        (bg, border)
    });
    let transformed_probe = cx.container(
        ContainerProps {
            layout: {
                let mut l = LayoutStyle::default();
                l.size.width = Length::Fill;
                l.size.height = Length::Px(Px(88.0));
                l.overflow = Overflow::Clip;
                l
            },
            background: Some(probe_bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(probe_border),
            corner_radii: Corners::all(Px(12.0)),
            padding: Edges::all(Px(12.0)).into(),
            ..Default::default()
        },
        move |cx| {
            let transform = Transform2D::translation(Point::new(Px(12.0), Px(6.0)))
                * Transform2D::scale_uniform(0.92);
            vec![cx.visual_transform(transform, move |_cx| vec![transformed_select])]
        },
    );

    ui::v_flex(move |cx| {
            vec![
                cx.text(
                    "Material 3 Select: token-driven trigger + listbox overlay + ADR 0220 style overrides.",
                ),
                ui::h_row(move |_cx| vec![default, overridden]).gap(Space::N4).items_center().into_element(cx),
                cx.text("Option richness probe (Material Web select-option supporting slots):"),
                rich_select,
                cx.text(
                    "Menu width probe (Material Web min-width behavior + optional 210px floor):",
                ),
                ui::h_row(move |cx| {
                        vec![
                            cx.text("menu_width_floor=210px"),
                            floor_toggle,
                            cx.text(if menu_width_floor_enabled_now {
                                "on"
                            } else {
                                "off"
                            }),
                        ]
                    }).gap(Space::N2).items_center().into_element(cx),
                unclamped,
                cx.text(format!(
                    "Typeahead delay probe (Material Web typeaheadDelay): current={}ms",
                    typeahead_delay_ms_now
                )),
                ui::h_row(move |cx| {
                        vec![
                            set_delay_button(cx, 200),
                            set_delay_button(cx, 500),
                            set_delay_button(cx, 1000),
                        ]
                    }).gap(Space::N2).items_center().into_element(cx),
                typeahead_select,
                cx.text(
                    "Menu positioning probe (Material Web menuPositioning): select is render-transformed + clipped; overlay should still align and avoid clipping.",
                ),
                transformed_probe,
            ]
        })
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4)
            .items_start().into_element(cx)
}

// endregion: example
