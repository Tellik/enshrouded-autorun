/// Release notes may contain one machine-readable line that makes a release a
/// blocking hotfix. Keep this directive out of the text shown in the UI.
pub fn parse_release_notes(body: &str) -> (bool, String) {
    let mut mandatory = false;
    let notes = body
        .lines()
        .filter(|line| {
            if line
                .trim()
                .eq_ignore_ascii_case("ShroudForge-Update: required")
            {
                mandatory = true;
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned();
    (mandatory, notes)
}
