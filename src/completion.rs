use std::{
    collections::HashSet,
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use rustyline::{
    Context, Helper,
    completion::{Completer, FilenameCompleter, Pair},
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
};

const BUILTIN_COMMANDS: &[&str] = &["echo", "exit", "type", "pwd", "cd"];

pub struct ShellHelper {
    filename_completer: FilenameCompleter,
}

impl ShellHelper {
    pub fn new() -> Self {
        Self {
            filename_completer: FilenameCompleter::new(),
        }
    }
}

impl Helper for ShellHelper {}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        context: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let word_start = line[..pos]
            .char_indices()
            .rev()
            .find(|(_, character)| character.is_whitespace())
            .map(|(index, character)| index + character.len_utf8())
            .unwrap_or(0);

        let before_word = line[..word_start].trim();
        let prefix = &line[word_start..pos];

        if before_word.is_empty() {
            let candidates = command_candidates(prefix);
            return Ok((word_start, candidates));
        }

        self.filename_completer.complete(line, pos, context)
    }
}

impl Hinter for ShellHelper {
    type Hint = String;
}

impl Highlighter for ShellHelper {}
impl Validator for ShellHelper {}

fn builtin_candidates(prefix: &str) -> Vec<Pair> {
    BUILTIN_COMMANDS
        .iter()
        .filter(|command| command.starts_with(prefix))
        .map(|command| Pair {
            display: (*command).to_string(),
            replacement: (*command).to_string(),
        })
        .collect()
}

fn command_candidates(prefix: &str) -> Vec<Pair> {
    let directories = env::var_os("PATH")
        .map(|path| env::split_paths(&path).collect::<Vec<_>>())
        .unwrap_or_default();

    command_candidates_in(prefix, &directories)
}

fn command_candidates_in(prefix: &str, directories: &[PathBuf]) -> Vec<Pair> {
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();

    for candidate in builtin_candidates(prefix) {
        seen.insert(candidate.replacement.clone());
        candidates.push(candidate);
    }

    for directory in directories {
        for name in executable_names(directory, prefix) {
            if seen.insert(name.clone()) {
                candidates.push(Pair {
                    display: name.clone(),
                    replacement: name,
                });
            }
        }
    }

    candidates.sort_by(|left, right| left.replacement.cmp(&right.replacement));
    candidates
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

    use super::{builtin_candidates, command_candidates_in, executable_names};

    #[test]
    fn completes_builtin_commands_by_prefix() {
        let candidates = builtin_candidates("ec");

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].replacement, "echo");
    }

    #[test]
    fn returns_all_builtins_for_empty_prefix() {
        assert_eq!(builtin_candidates("").len(), 5);
    }

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

        let candidates = command_candidates_in("my", &[first.clone(), second.clone()]);
        assert_eq!(replacements(&candidates), vec!["mycmd".to_string()]);

        let with_builtin = command_candidates_in("", &[first.clone()]);
        let names = replacements(&with_builtin);
        assert_eq!(names.iter().filter(|name| *name == "echo").count(), 1);
        assert!(names.iter().any(|name| name == "mycmd"));

        let _ = fs::remove_dir_all(&first);
        let _ = fs::remove_dir_all(&second);
    }

    fn replacements(candidates: &[super::Pair]) -> Vec<String> {
        candidates
            .iter()
            .map(|candidate| candidate.replacement.clone())
            .collect()
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
