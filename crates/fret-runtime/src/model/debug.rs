#[derive(Debug, Clone, Copy)]
pub struct ModelCreatedDebugInfo {
    pub type_name: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ModelChangedDebugInfo {
    pub type_name: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}
