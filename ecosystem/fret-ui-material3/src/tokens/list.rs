//! Component-facing token access for Material 3 lists.
//!
//! Shared list fallback/default policy lives in `tokens::list_common`; this facade keeps existing
//! list token call sites stable.

pub(crate) use crate::tokens::list_common::{
    ListItemInteraction, item_between_space, item_bottom_space,
    item_container_shape_for_interaction, item_leading_space, item_outcomes, item_top_space,
    item_trailing_space, label_text_style, leading_icon_size_with_variant,
    one_line_container_height, overline_text_color, overline_text_style,
    pressed_state_layer_opacity, selected_container_background, supporting_text_color,
    supporting_text_style, three_line_container_height, trailing_icon_size_with_variant,
    trailing_supporting_text_color, trailing_supporting_text_style, two_line_container_height,
};
