pub fn run_public_diag_contract(args: Vec<String>) -> Result<(), String> {
    fret_diag::diag_cmd_with_mode(fret_diag::DiagCliMode::PublicAppAuthor, args)
}
