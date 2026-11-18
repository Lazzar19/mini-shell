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
            let candidates = builtin_candidates(prefix);
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

#[cfg(test)]
mod tests {
    use super::builtin_candidates;

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
}
