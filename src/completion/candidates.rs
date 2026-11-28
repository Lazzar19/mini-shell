use std::{
    collections::HashSet,
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

const BUILTIN_COMMANDS: &[&str] = &["echo", "exit", "type", "pwd", "cd", "complete"];

pub fn command_candidates(prefix: &str) -> Vec<String> {
    let directories = env::var_os("PATH")
        .map(|path| env::split_paths(&path).collect::<Vec<_>>())
        .unwrap_or_default();

    command_candidates_in(prefix, &directories)
}

pub fn command_candidates_in(prefix: &str, directories: &[PathBuf]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut names = Vec::new();

    for command in BUILTIN_COMMANDS {
        if command.starts_with(prefix) && seen.insert((*command).to_string()) {
            names.push((*command).to_string());
        }
    }

    for directory in directories {
        for name in executable_names(directory, prefix) {
            if seen.insert(name.clone()) {
                names.push(name);
            }
        }
    }

    names.sort();
    names
}

fn executable_names(directory: &Path, prefix: &str) -> Vec<String> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut names = Vec::new();

    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };

        if !name.starts_with(prefix) || name.contains(char::is_whitespace) {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        // Same rule as command lookup: a regular file with any execute bit set.
        if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
            names.push(name);
        }
    }

    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
    };

    use super::{command_candidates_in, executable_names};

    #[test]
    fn lists_executable_files_matching_prefix() {
        let directory = temp_case_dir("execs");
        write_executable(&directory, "mycmd");
        write_executable(&directory, "myother");
        fs::write(directory.join("readme"), b"text").unwrap();
        fs::create_dir(directory.join("mydir")).unwrap();

        assert_eq!(
            executable_names(&directory, "my"),
            vec!["mycmd".to_string(), "myother".to_string()]
        );

        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn merges_builtins_with_path_executables_without_duplicates() {
        let first = temp_case_dir("path-a");
        let second = temp_case_dir("path-b");
        write_executable(&first, "mycmd");
        write_executable(&first, "echo");
        write_executable(&second, "mycmd");

        assert_eq!(
            command_candidates_in("my", &[first.clone(), second.clone()]),
            vec!["mycmd".to_string()]
        );

        let names = command_candidates_in("", &[first.clone()]);
        assert_eq!(names.iter().filter(|name| *name == "echo").count(), 1);
        assert!(names.iter().any(|name| name == "mycmd"));
        assert_eq!(names.len(), 7);

        let _ = fs::remove_dir_all(&first);
        let _ = fs::remove_dir_all(&second);
    }

    fn temp_case_dir(label: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "mini-shell-completion-{}-{label}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        directory
    }

    fn write_executable(directory: &Path, name: &str) {
        let path = directory.join(name);
        fs::write(&path, b"#!/bin/sh\n").unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
    }
}
