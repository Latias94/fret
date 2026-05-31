use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    SemanticsDecoration, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::popup_list::editor_popup_list_row_gap;
use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;

use super::row::color_copy_menu_row;
use super::{ColorEditCopyEntry, color_copy_entries};

pub(super) fn color_copy_menu_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    color: Color,
    show_alpha: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let density = {
        let theme = Theme::global(&*cx.app);
        EditorDensity::resolve(theme)
    };
    let popup_chrome = {
        let theme = Theme::global(&*cx.app);
        resolve_editor_popup_surface_chrome(theme, true)
    };
    let entries = Arc::<[ColorEditCopyEntry]>::from(color_copy_entries(color, show_alpha));
    let row_test_id_prefix = test_id.clone();

    let mut panel = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(196.0)),
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges::all(Px(4.0)).into(),
            background: Some(popup_chrome.bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(popup_chrome.border),
            corner_radii: Corners::all(popup_chrome.radius),
            shadow: popup_chrome.shadow,
            ..Default::default()
        },
        {
            let entries = entries.clone();
            let row_test_id_prefix = row_test_id_prefix.clone();
            move |cx| {
                vec![cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        direction: Axis::Vertical,
                        gap: SpacingLength::Px(editor_popup_list_row_gap()),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                        wrap: false,
                    },
                    {
                        let entries = entries.clone();
                        let row_test_id_prefix = row_test_id_prefix.clone();
                        move |cx| {
                            entries
                                .iter()
                                .map(|entry| {
                                    color_copy_menu_row(
                                        cx,
                                        entry.clone(),
                                        open.clone(),
                                        density.row_height,
                                        derived_test_id(
                                            row_test_id_prefix.as_ref(),
                                            entry.format.test_suffix(),
                                        ),
                                    )
                                })
                                .collect::<Vec<_>>()
                        }
                    },
                )]
            }
        },
    );

    let mut semantics = SemanticsDecoration::default().role(SemanticsRole::Menu);
    if let Some(test_id) = test_id {
        semantics = semantics.test_id(test_id);
    }
    panel = panel.attach_semantics(semantics);
    panel
}
