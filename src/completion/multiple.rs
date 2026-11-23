/// What to do when several commands match and none can be added yet.
pub enum AmbiguousTab {
    /// First Tab: ring the bell and leave the word unchanged.
    Bell,
    /// Second Tab: one line of matches, separated by two spaces.
    Show(String),
}

pub fn ambiguous_tab(names: &[String], repeat: bool) -> AmbiguousTab {
    if repeat {
        AmbiguousTab::Show(names.join("  "))
    } else {
        AmbiguousTab::Bell
    }
}

#[cfg(test)]
mod tests {
    use super::AmbiguousTab;
    use super::ambiguous_tab;

    fn names() -> Vec<String> {
        ["xyz_bar", "xyz_baz", "xyz_quz"]
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn first_tab_rings_without_listing() {
        assert!(matches!(ambiguous_tab(&names(), false), AmbiguousTab::Bell));
    }

    #[test]
    fn second_tab_lists_matches_with_two_spaces() {
        match ambiguous_tab(&names(), true) {
            AmbiguousTab::Show(line) => {
                assert_eq!(line, "xyz_bar  xyz_baz  xyz_quz");
            }
            AmbiguousTab::Bell => panic!("second tab should list matches"),
        }
    }
}
