use crate::ui::canvas::widget::*;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

pub(super) fn context_menu_activation_payload(
    menu: &ContextMenuState,
    index: usize,
) -> Option<(
    ContextMenuTarget,
    Point,
    NodeGraphContextMenuItem,
    Vec<InsertNodeCandidate>,
)> {
    let item = menu.items.get(index).cloned()?;
    if !item.enabled {
        return None;
    }
    Some((
        menu.target.clone(),
        menu.invoked_at,
        item,
        menu.candidates.clone(),
    ))
}
