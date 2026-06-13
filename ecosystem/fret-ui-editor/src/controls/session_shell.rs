use fret_core::Px;
use fret_ui::element::{
    AnyElement, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle, StackProps,
};
use fret_ui::{ElementContext, UiHost};

pub(super) fn session_shell_layout(mut layout: LayoutStyle, row_height: Px) -> LayoutStyle {
    if layout.size.min_height.is_none() {
        layout.size.min_height = Some(Length::Px(row_height));
    }
    if matches!(layout.size.width, Length::Auto) {
        layout.size.width = Length::Fill;
    }
    layout
}

pub(super) fn session_branch_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            min_width: Some(Length::Px(Px(0.0))),
            ..Default::default()
        },
        overflow: Overflow::Clip,
        ..Default::default()
    }
}

pub(super) fn hidden_session_branch_layout(mut layout: LayoutStyle) -> LayoutStyle {
    layout.size = SizeStyle {
        width: Length::Px(Px(0.0)),
        height: Length::Px(Px(0.0)),
        min_width: Some(Length::Px(Px(0.0))),
        min_height: Some(Length::Px(Px(0.0))),
        ..Default::default()
    };
    layout.position = PositionStyle::Absolute;
    layout.overflow = Overflow::Clip;
    layout
}

pub(super) fn session_shell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutStyle,
    children: Vec<AnyElement>,
) -> AnyElement {
    cx.stack_props(StackProps { layout }, move |_cx| children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_ui::element::FlexItemStyle;

    #[test]
    fn session_shell_layout_preserves_caller_flex_and_adds_row_min_height() {
        let layout = LayoutStyle {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Auto,
                ..Default::default()
            },
            flex: FlexItemStyle {
                grow: 1.0,
                basis: Length::Px(Px(0.0)),
                ..Default::default()
            },
            ..Default::default()
        };

        let resolved = session_shell_layout(layout, Px(24.0));

        assert_eq!(resolved.size.width, Length::Fill);
        assert_eq!(resolved.size.height, Length::Auto);
        assert_eq!(resolved.size.min_height, Some(Length::Px(Px(24.0))));
        assert_eq!(resolved.flex.grow, 1.0);
        assert_eq!(resolved.flex.basis, Length::Px(Px(0.0)));
    }

    #[test]
    fn session_shell_layout_keeps_explicit_min_height_and_width() {
        let layout = LayoutStyle {
            size: SizeStyle {
                width: Length::Px(Px(64.0)),
                min_height: Some(Length::Px(Px(32.0))),
                ..Default::default()
            },
            ..Default::default()
        };

        let resolved = session_shell_layout(layout, Px(24.0));

        assert_eq!(resolved.size.width, Length::Px(Px(64.0)));
        assert_eq!(resolved.size.min_height, Some(Length::Px(Px(32.0))));
    }

    #[test]
    fn active_session_branch_fills_shell_without_external_flex() {
        let layout = session_branch_layout();

        assert_eq!(layout.size.width, Length::Fill);
        assert_eq!(layout.size.height, Length::Fill);
        assert_eq!(layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(layout.flex.grow, 0.0);
        assert_eq!(layout.flex.basis, Length::Auto);
    }

    #[test]
    fn hidden_session_branch_is_absolute_zero_sized() {
        let layout = hidden_session_branch_layout(session_branch_layout());

        assert_eq!(layout.size.width, Length::Px(Px(0.0)));
        assert_eq!(layout.size.height, Length::Px(Px(0.0)));
        assert_eq!(layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(layout.size.min_height, Some(Length::Px(Px(0.0))));
        assert_eq!(layout.position, PositionStyle::Absolute);
        assert_eq!(layout.overflow, Overflow::Clip);
    }
}
