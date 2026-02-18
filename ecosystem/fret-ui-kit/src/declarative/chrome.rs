use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::element::{
    AnyElement, ContainerProps, Length, Overflow, PressableProps, PressableState,
};
use fret_ui::elements::GlobalElementId;

fn normalize_control_chrome_sizing(
    pressable_props: &PressableProps,
    chrome_props: &mut ContainerProps,
) {
    // Normalize chrome sizing so control min/max constraints behave like `box-sizing:
    // border-box` (Tailwind preflight default): the pressable drives the *outer* box size,
    // while the chrome node's constraints apply to its inner content box after
    // padding/border.
    //
    // Without this, shadcn-style controls that apply `min-height` + `py-*` would inflate:
    // the chrome node enforces the min-height, then adds padding on top (e.g. `h-9` becomes
    // 52px).
    let pad_x = chrome_props.padding.left.0 + chrome_props.padding.right.0;
    let pad_y = chrome_props.padding.top.0 + chrome_props.padding.bottom.0;
    let border_x = chrome_props.border.left.0 + chrome_props.border.right.0;
    let border_y = chrome_props.border.top.0 + chrome_props.border.bottom.0;
    let inset_x = pad_x + border_x;
    let inset_y = pad_y + border_y;
    let shrink_px = |v: fret_core::Px, inset: f32| fret_core::Px((v.0 - inset).max(0.0));

    let parent_size = pressable_props.layout.size;
    let child_size = &mut chrome_props.layout.size;

    // If the pressable has an explicit outer box size, the chrome should fill that box.
    //
    // This keeps border-box alignment intuitive for fixed-size controls (e.g. Checkbox,
    // Switch) and matches the browser mental model where border/padding live inside the same
    // border-box dimensions.
    let chrome_fill_w = matches!(parent_size.width, Length::Px(_) | Length::Fill)
        || pressable_props.layout.flex.grow > 0.0;
    let chrome_fill_h = matches!(parent_size.height, Length::Px(_) | Length::Fill)
        || pressable_props.layout.flex.grow > 0.0;
    if chrome_fill_w {
        child_size.width = Length::Fill;
    }
    if chrome_fill_h {
        child_size.height = Length::Fill;
    }

    if let Some(min_h) = parent_size.min_height {
        child_size.min_height = Some(shrink_px(min_h, inset_y));
    }
    if let Some(max_h) = parent_size.max_height {
        child_size.max_height = Some(shrink_px(max_h, inset_y));
    }
    if !chrome_fill_h && let Length::Px(h) = parent_size.height {
        child_size.height = Length::Px(shrink_px(h, inset_y));
    }

    if let Some(min_w) = parent_size.min_width {
        child_size.min_width = Some(shrink_px(min_w, inset_x));
    }
    if let Some(max_w) = parent_size.max_width {
        child_size.max_width = Some(shrink_px(max_w, inset_x));
    }
    if !chrome_fill_w && let Length::Px(w) = parent_size.width {
        child_size.width = Length::Px(shrink_px(w, inset_x));
    }
}

/// Composes the recommended "control chrome" structure:
///
/// - outer `Pressable` remains `Overflow::Visible` so focus rings can extend outward
/// - inner chrome `Container` is forced to `Overflow::Clip` so rounded corners/borders mask content
///
/// This matches the common shadcn/Radix mental model of:
/// `Pressable (focus ring) -> SurfaceChrome (overflow-hidden) -> content`.
#[track_caller]
pub fn control_chrome_pressable_with_id_props<'a, H, F, C, I>(
    cx: &mut ElementContext<'a, H>,
    f: F,
) -> AnyElement
where
    H: UiHost + 'a,
    F: FnOnce(
        &mut ElementContext<'a, H>,
        PressableState,
        GlobalElementId,
    ) -> (PressableProps, ContainerProps, C),
    C: for<'b> FnOnce(&'b mut ElementContext<'a, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    cx.pressable_with_id_props(|cx, st, id| {
        let (mut pressable_props, mut chrome_props, children) = f(cx, st, id);

        pressable_props.layout.overflow = Overflow::Visible;
        chrome_props.layout.overflow = Overflow::Clip;
        normalize_control_chrome_sizing(&pressable_props, &mut chrome_props);

        let content = cx.container(chrome_props, children);
        (pressable_props, vec![content])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_chrome_fills_when_pressable_flex_grows() {
        let mut pressable = PressableProps::default();
        pressable.layout.flex.grow = 1.0;

        let mut chrome = ContainerProps::default();
        assert_eq!(chrome.layout.size.width, Length::Auto);
        assert_eq!(chrome.layout.size.height, Length::Auto);

        normalize_control_chrome_sizing(&pressable, &mut chrome);
        assert_eq!(chrome.layout.size.width, Length::Fill);
        assert_eq!(chrome.layout.size.height, Length::Fill);
    }

    #[test]
    fn control_chrome_fills_when_pressable_width_is_fill() {
        let mut pressable = PressableProps::default();
        pressable.layout.size.width = Length::Fill;

        let mut chrome = ContainerProps::default();
        normalize_control_chrome_sizing(&pressable, &mut chrome);
        assert_eq!(chrome.layout.size.width, Length::Fill);
        assert_eq!(chrome.layout.size.height, Length::Auto);
    }

    #[test]
    fn control_chrome_fills_when_pressable_height_is_fill() {
        let mut pressable = PressableProps::default();
        pressable.layout.size.height = Length::Fill;

        let mut chrome = ContainerProps::default();
        normalize_control_chrome_sizing(&pressable, &mut chrome);
        assert_eq!(chrome.layout.size.width, Length::Auto);
        assert_eq!(chrome.layout.size.height, Length::Fill);
    }

    #[test]
    fn control_chrome_fills_when_pressable_width_is_px() {
        let mut pressable = PressableProps::default();
        pressable.layout.size.width = Length::Px(fret_core::Px(123.0));

        let mut chrome = ContainerProps::default();
        normalize_control_chrome_sizing(&pressable, &mut chrome);
        assert_eq!(chrome.layout.size.width, Length::Fill);
    }

    #[test]
    fn control_chrome_fills_when_pressable_height_is_px() {
        let mut pressable = PressableProps::default();
        pressable.layout.size.height = Length::Px(fret_core::Px(45.0));

        let mut chrome = ContainerProps::default();
        normalize_control_chrome_sizing(&pressable, &mut chrome);
        assert_eq!(chrome.layout.size.height, Length::Fill);
    }

    #[test]
    fn control_chrome_shrinks_min_max_constraints_by_padding_and_border() {
        let mut pressable = PressableProps::default();
        pressable.layout.size.min_width = Some(fret_core::Px(40.0));
        pressable.layout.size.max_width = Some(fret_core::Px(50.0));
        pressable.layout.size.min_height = Some(fret_core::Px(20.0));
        pressable.layout.size.max_height = Some(fret_core::Px(30.0));

        let mut chrome = ContainerProps::default();
        chrome.padding = fret_core::Edges::all(fret_core::Px(2.0));
        chrome.border = fret_core::Edges::all(fret_core::Px(1.0));

        normalize_control_chrome_sizing(&pressable, &mut chrome);

        // inset = (pad_left + pad_right) + (border_left + border_right) = (2+2) + (1+1) = 6
        assert_eq!(chrome.layout.size.min_width, Some(fret_core::Px(34.0)));
        assert_eq!(chrome.layout.size.max_width, Some(fret_core::Px(44.0)));
        assert_eq!(chrome.layout.size.min_height, Some(fret_core::Px(14.0)));
        assert_eq!(chrome.layout.size.max_height, Some(fret_core::Px(24.0)));
    }
}
