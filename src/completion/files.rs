use std::{env, fs, path::Path};

/// Names that can replace the path typed so far. Directories end with `/`.
pub fn file_candidates(prefix: &str) -> Vec<String> {
    let Ok(current) = env::current_dir() else {
        return Vec::new();
    };

    file_candidates_in(prefix, &current)
}

pub fn file_candidates_in(prefix: &str, current_dir: &Path) -> Vec<String> {
    let (directory_prefix, name_prefix) = split_typed_path(prefix);
    let directory = if directory_prefix.is_empty() {
        current_dir.to_path_buf()
    } else {
        current_dir.join(directory_prefix)
    };

    let Ok(entries) = fs::read_dir(&directory) else {
        return Vec::new();
    };

    let mut names = Vec::new();

    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };

        if !name.starts_with(name_prefix) || name.contains(char::is_whitespace) {
            continue;
        }

        if name_prefix.is_empty() && name.starts_with('.') {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        let mut completed = format!("{directory_prefix}{name}");
        if metadata.is_dir() {
            completed.push('/');
        }
        names.push(completed);
    }

    names.sort();
    names
}

/// A finished directory keeps the slash and does not take the trailing space
/// added for a finished file.
pub fn present(replacement: String) -> String {
    replacement
        .strip_suffix("/ ")
        .map(|directory| format!("{directory}/"))
        .unwrap_or(replacement)
}

fn split_typed_path(prefix: &str) -> (&str, &str) {
    match prefix.rfind('/') {
        Some(index) => (&prefix[..=index], &prefix[index + 1..]),
        None => ("", prefix),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{file_candidates_in, present};
    use crate::completion::multiple::{AmbiguousTab, ambiguous_tab};
    use crate::completion::partial::insertion;

    #[test]
    fn completes_a_unique_file_name() {
        let directory = temp_case_dir("unique");
        fs::write(directory.join("main.py"), b"").unwrap();
        fs::write(directory.join("readme.md"), b"").unwrap();

        let main_file = file_candidates_in("main", &directory);
        assert_eq!(insertion("main", &main_file).as_deref(), Some("main.py "));
        assert_eq!(
            insertion("read", &file_candidates_in("read", &directory)).as_deref(),
            Some("readme.md ")
        );

        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn completes_each_later_argument_on_its_own() {
        let directory = temp_case_dir("multi");
        fs::create_dir(directory.join("bar")).unwrap();
        fs::write(directory.join("foo"), b"").unwrap();

        let directory_match = file_candidates_in("b", &directory);
        assert_eq!(present(insertion("b", &directory_match).unwrap()), "bar/");

        let file_match = file_candidates_in("f", &directory);
        assert_eq!(insertion("f", &file_match).as_deref(), Some("foo "));

        assert!(file_candidates_in("x", &directory).is_empty());

        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn stops_at_the_shared_prefix_when_several_files_match() {
        let directory = temp_case_dir("partial");
        fs::write(directory.join("main.rs"), b"").unwrap();
        fs::write(directory.join("main.cpp"), b"").unwrap();
        fs::write(directory.join("main.c"), b"").unwrap();

        let names = file_candidates_in("main", &directory);
        assert_eq!(
            names,
            vec![
                "main.c".to_string(),
                "main.cpp".to_string(),
                "main.rs".to_string()
            ]
        );
        assert_eq!(insertion("main", &names).as_deref(), Some("main."));

        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn completes_inside_a_typed_directory() {
        let directory = temp_case_dir("nested");
        fs::create_dir(directory.join("src")).unwrap();
        fs::write(directory.join("src/main.rs"), b"").unwrap();

        assert_eq!(
            file_candidates_in("src/mai", &directory),
            vec!["src/main.rs".to_string()]
        );

        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn empty_argument_lists_files_and_directories_on_the_second_tab() {
        let directory = temp_case_dir("several");
        fs::write(directory.join("bar.txt"), b"").unwrap();
        fs::create_dir(directory.join("foo")).unwrap();

        let names = file_candidates_in("", &directory);
        assert_eq!(names, vec!["bar.txt".to_string(), "foo/".to_string()]);
        assert_eq!(insertion("", &names), None);
        assert!(matches!(ambiguous_tab(&names, false), AmbiguousTab::Bell));
        match ambiguous_tab(&names, true) {
            AmbiguousTab::Show(line) => assert_eq!(line, "bar.txt  foo/"),
            AmbiguousTab::Bell => panic!("second tab should list matches"),
        }

        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn marks_a_unique_directory_with_a_slash() {
        let directory = temp_case_dir("dir");
        fs::create_dir(directory.join("docs")).unwrap();

        let names = file_candidates_in("do", &directory);
        assert_eq!(present(insertion("do", &names).unwrap()), "docs/");

        let _ = fs::remove_dir_all(&directory);
    }

    fn temp_case_dir(label: &str) -> std::path::PathBuf {
        let directory =
            std::env::temp_dir().join(format!("mini-shell-files-{}-{label}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        directory
    }
}
