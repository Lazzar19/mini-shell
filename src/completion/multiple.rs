/// First `<TAB>`: ring the bell (`\x07`) to indicate multiple matches exist.
/// Subsequent `<TAB>` presses: list all matches in alphabetical order on a new line,
/// then redisplay the prompt.
pub enum AmbiguousTab {
    Bell,
    Show(String),
}

pub fn ambiguous_tab(names: &[String], repeat: bool) -> AmbiguousTab {
    if repeat {
        let mut ordered = names.to_vec();
        ordered.sort();
        AmbiguousTab::Show(ordered.join("  "))
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
    fn later_tabs_list_matches_alphabetically() {
        let names = ["xyz_quz", "xyz_bar", "xyz_baz"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();

        match ambiguous_tab(&names, true) {
            AmbiguousTab::Show(line) => {
                assert_eq!(line, "xyz_bar  xyz_baz  xyz_quz");
            }
            AmbiguousTab::Bell => panic!("later tabs should list matches"),
        }
    }
}
