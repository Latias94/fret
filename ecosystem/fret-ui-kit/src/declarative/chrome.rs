use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::element::{
    AnyElement, ContainerProps, Length, Overflow, PressableProps, PressableState,
};
use fret_ui::elements::GlobalElementId;

/// Composes the recommended "control chrome" structure:
///
/// - outer `Pressable` remains `Overflow::Visible` so focus rings can extend outward
/// - inner chrome `Container` is forced to `Overflow::Clip` so rounded corners/borders mask content
///
/// This matches the common shadcn/Radix mental model of:
/// `Pressable (focus ring) -> SurfaceChrome (overflow-hidden) -> content`.
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
        let chrome_fill_w = matches!(parent_size.width, Length::Px(_) | Length::Fill);
        let chrome_fill_h = matches!(parent_size.height, Length::Px(_) | Length::Fill);
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

        let content = cx.container(chrome_props, children);
        (pressable_props, vec![content])
    })
}
