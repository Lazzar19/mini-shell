/// Text that replaces the current word, when Tab can move the cursor forward.
pub fn insertion(prefix: &str, names: &[String]) -> Option<String> {
    match names {
        [only] => Some(format!("{only} ")),
        [_, _, ..] => {
            let shared = shared_prefix(names);
            if shared.len() > prefix.len() {
                Some(shared.to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn shared_prefix(names: &[String]) -> &str {
    let Some(first) = names.first() else {
        return "";
    };

    let mut end = first.len();
    for name in &names[1..] {
        let matched = first
            .bytes()
            .zip(name.bytes())
            .take_while(|(left, right)| left == right)
            .count();
        end = end.min(matched);
    }

    while end > 0 && !first.is_char_boundary(end) {
        end -= 1;
    }

    &first[..end]
}

#[cfg(test)]
mod tests {
    use super::insertion;

    fn owned(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }

    #[test]
    fn completes_only_the_shared_prefix() {
        let names = owned(&["xyz_foo", "xyz_foo_bar", "xyz_foo_bar_quz"]);

        assert_eq!(insertion("xyz_", &names).as_deref(), Some("xyz_foo"));
        assert_eq!(insertion("xyz_foo", &names), None);
    }

    #[test]
    fn extends_a_longer_shared_prefix() {
        let names = owned(&["xyz_foo_bar_baz", "xyz_foo_bar_quz"]);

        assert_eq!(
            insertion("xyz_foo_", &names).as_deref(),
            Some("xyz_foo_bar_")
        );
    }

    #[test]
    fn completes_a_unique_command_with_a_trailing_space() {
        let names = owned(&["xyz_foo_bar_baz"]);

        assert_eq!(
            insertion("xyz_foo_bar_", &names).as_deref(),
            Some("xyz_foo_bar_baz ")
        );
    }
}
