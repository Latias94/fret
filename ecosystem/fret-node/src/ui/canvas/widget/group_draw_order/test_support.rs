use crate::core::GroupId;

pub(super) fn group_ids(count: usize) -> Vec<GroupId> {
    (0..count).map(|_| GroupId::new()).collect()
}
