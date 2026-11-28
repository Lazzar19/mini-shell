use std::collections::BTreeMap;

#[derive(Default)]
pub struct CompletionSpecs {
    completers: BTreeMap<String, String>,
}

pub enum CompleteLine {
    Stdout(String),
    Stderr(String),
}

impl CompletionSpecs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, command: impl Into<String>, completer: impl Into<String>) {
        self.completers.insert(command.into(), completer.into());
    }
}

pub fn run(args: &[String], specs: &mut CompletionSpecs) -> Vec<CompleteLine> {
    let Some(flag) = args.first() else {
        return Vec::new();
    };

    if flag == "-C" || flag == "--C" {
        return register(args, specs);
    }

    if flag != "-p" && flag != "--p" {
        return Vec::new();
    }

    args[1..]
        .iter()
        .map(|name| match specs.completers.get(name) {
            Some(completer) => CompleteLine::Stdout(format_spec(name, completer)),
            None => {
                CompleteLine::Stderr(format!("complete: {name}: no completion specification\n"))
            }
        })
        .collect()
}

fn register(args: &[String], specs: &mut CompletionSpecs) -> Vec<CompleteLine> {
    let Some(completer) = args.get(1) else {
        return vec![CompleteLine::Stderr(
            "complete: -C: option requires an argument\n".to_string(),
        )];
    };

    let commands = &args[2..];
    if commands.is_empty() {
        return vec![CompleteLine::Stderr(
            "complete: -C: command name required\n".to_string(),
        )];
    }

    for command in commands {
        specs.insert(command, completer);
    }

    Vec::new()
}

fn format_spec(name: &str, completer: &str) -> String {
    format!("complete -C {} {name}\n", single_quote(completer))
}

fn single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::{CompleteLine, CompletionSpecs, run};

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn prints_a_stored_completion_spec() {
        let mut specs = CompletionSpecs::new();
        specs.insert("git", "/path/to/git/completer");

        let lines = run(&args(&["-p", "git"]), &mut specs);
        assert!(matches!(
            lines.as_slice(),
            [CompleteLine::Stdout(text)] if text == "complete -C '/path/to/git/completer' git\n"
        ));
    }

    #[test]
    fn registers_a_completer_then_prints_it() {
        let mut specs = CompletionSpecs::new();

        assert!(run(&args(&["-C", "/path/to/git/completer", "git"]), &mut specs).is_empty());
        let git = run(&args(&["-p", "git"]), &mut specs);
        assert!(matches!(
            git.as_slice(),
            [CompleteLine::Stdout(text)] if text == "complete -C '/path/to/git/completer' git\n"
        ));

        assert!(
            run(
                &args(&["-C", "/path/to/docker/completer", "docker"]),
                &mut specs
            )
            .is_empty()
        );
        let docker = run(&args(&["-p", "docker"]), &mut specs);
        assert!(matches!(
            docker.as_slice(),
            [CompleteLine::Stdout(text)] if text == "complete -C '/path/to/docker/completer' docker\n"
        ));

        let git_again = run(&args(&["-p", "git"]), &mut specs);
        assert!(matches!(
            git_again.as_slice(),
            [CompleteLine::Stdout(text)] if text == "complete -C '/path/to/git/completer' git\n"
        ));
    }

    #[test]
    fn reports_a_missing_completion_spec() {
        let mut specs = CompletionSpecs::new();
        let lines = run(&args(&["-p", "git"]), &mut specs);

        assert!(matches!(
            lines.as_slice(),
            [CompleteLine::Stderr(text)] if text == "complete: git: no completion specification\n"
        ));
    }
}
