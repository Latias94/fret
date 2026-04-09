pub(crate) mod contracts {
    pub(crate) use fretboard::scaffold::contracts::*;
}

pub(crate) fn run_new_contract(args: contracts::NewCommandArgs) -> Result<(), String> {
    let workspace_root = crate::cli::workspace_root()?;
    fretboard::scaffold::run_repo_new_contract(args, &workspace_root)
}
