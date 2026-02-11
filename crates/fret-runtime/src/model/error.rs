#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelUpdateError {
    NotFound,
    AlreadyLeased,
    TypeMismatch,
}
