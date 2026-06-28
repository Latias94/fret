use std::hash::Hash;

use fret_core::Px;
use fret_ui::element::{
    AnyElement, InteractivityGateProps, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle,
    StackProps,
};
use fret_ui::{ElementContext, UiHost};

pub(super) fn session_shell_layout(mut layout: LayoutStyle, control_height: Px) -> LayoutStyle {
    if layout.size.min_height.is_none() {
        layout.size.min_height = Some(Length::Px(control_height));
    }
    if matches!(layout.size.width, Length::Auto) {
        layout.size.width = Length::Fill;
    }
    if matches!(layout.size.height, Length::Auto) {
        layout.size.height = Length::Px(control_height);
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

pub(super) fn inactive_session_child_layout(mut layout: LayoutStyle) -> LayoutStyle {
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

/// Keep an inactive session subtree mounted while removing it from layout, paint, hit-testing,
/// focus traversal, and semantics.
pub(super) fn session_hidden_branch<K, H>(
    cx: &mut ElementContext<'_, H>,
    key: K,
    child: AnyElement,
) -> AnyElement
where
    K: Hash,
    H: UiHost,
{
    cx.keyed(("session-hidden-branch", key), |cx| {
        cx.interactivity_gate_props(
            InteractivityGateProps {
                present: false,
                interactive: false,
                ..Default::default()
            },
            |_cx| vec![child],
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_ui::element::FlexItemStyle;

    #[test]
    fn session_shell_layout_preserves_caller_flex_item_and_adds_control_min_height() {
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
        assert_eq!(resolved.size.height, Length::Px(Px(24.0)));
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
        assert_eq!(resolved.size.height, Length::Px(Px(24.0)));
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
    fn inactive_session_child_layout_is_absolute_zero_sized() {
        let layout = inactive_session_child_layout(session_branch_layout());

        assert_eq!(layout.size.width, Length::Px(Px(0.0)));
        assert_eq!(layout.size.height, Length::Px(Px(0.0)));
        assert_eq!(layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(layout.size.min_height, Some(Length::Px(Px(0.0))));
        assert_eq!(layout.position, PositionStyle::Absolute);
        assert_eq!(layout.overflow, Overflow::Clip);
    }
}
