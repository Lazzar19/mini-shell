use rustyline::{
    completion::{Completer, FilenameCompleter, Pair},
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
    Context, Helper,
};


pub struct ShellHelper {
    completer: FilenameCompleter,
}

impl ShellHelper {
    pub fn new() -> Self {
        Self {
            completer: FilenameCompleter::new(),
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
        context: &Context<'_>
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        self.completer.complete(line, pos, context)
    }
}


impl Hinter for ShellHelper {
    type Hint = String;
}

impl Highlighter for ShellHelper {}
impl Validator for ShellHelper {}