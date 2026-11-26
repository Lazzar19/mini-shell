mod candidates;
mod files;
mod multiple;
mod partial;

use std::{
    cell::RefCell,
    io::{self, Write},
};

use rustyline::{
    Context, Helper,
    completion::{Completer, Pair},
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
};

use candidates::command_candidates;
use files::{file_candidates, present};
use multiple::AmbiguousTab;

struct PendingList {
    line: String,
    position: usize,
}

pub struct ShellHelper {
    pending_list: RefCell<Option<PendingList>>,
}

impl ShellHelper {
    pub fn new() -> Self {
        Self {
            pending_list: RefCell::new(None),
        }
    }

    /// The first Tab on this input is not a repeat. Later Tabs stay repeats until the input changes.
    fn repeat_of_previous_tab(&self, line: &str, position: usize) -> bool {
        let mut pending = self.pending_list.borrow_mut();
        let same = pending
            .as_ref()
            .is_some_and(|previous| previous.line == line && previous.position == position);

        if !same {
            *pending = Some(PendingList {
                line: line.to_string(),
                position,
            });
        }

        same
    }
}

#[cfg(test)]
mod tests {
    use super::ShellHelper;

    #[test]
    fn tabs_after_the_first_keep_listing_until_the_input_changes() {
        let helper = ShellHelper::new();

        assert!(!helper.repeat_of_previous_tab("stat ", 5));
        assert!(helper.repeat_of_previous_tab("stat ", 5));
        assert!(helper.repeat_of_previous_tab("stat ", 5));
        assert!(!helper.repeat_of_previous_tab("stat b", 7));
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
        let completing_command = before_word.is_empty();
        let _ = context;

        let names = if completing_command {
            command_candidates(prefix)
        } else {
            file_candidates(prefix)
        };

        if names.is_empty() {
            self.pending_list.borrow_mut().take();
            return Ok((word_start, Vec::new()));
        }

        if let Some(text) = partial::insertion(prefix, &names) {
            self.pending_list.borrow_mut().take();
            let text = if completing_command {
                text
            } else {
                present(text)
            };
            return Ok((word_start, vec![pair(text)]));
        }

        let repeat = self.repeat_of_previous_tab(line, pos);
        match multiple::ambiguous_tab(&names, repeat) {
            AmbiguousTab::Bell => Ok((word_start, Vec::new())),
            AmbiguousTab::Show(matches) => {
                print_matches(&matches);
                Ok((word_start, vec![pair(prefix.to_string())]))
            }
        }
    }
}

impl Hinter for ShellHelper {
    type Hint = String;
}

impl Highlighter for ShellHelper {}
impl Validator for ShellHelper {}

fn pair(replacement: String) -> Pair {
    Pair {
        display: replacement.clone(),
        replacement,
    }
}

fn print_matches(matches: &str) {
    let mut stdout = io::stdout();
    let _ = writeln!(stdout, "\n{matches}");
    let _ = stdout.flush();
}
