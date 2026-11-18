use std::{
    fs::{File, OpenOptions},
    io::Write,
};

enum RedirectMode {
    Overwrite,
    Append,
}

struct RedirectFile {
    filename: String,
    mode: RedirectMode,
}

impl RedirectFile {
    fn open(&self) -> Result<File, std::io::Error> {
        let append = matches!(self.mode, RedirectMode::Append);

        OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(&self.filename)
    }
}

pub struct Redirections {
    stdout: Option<RedirectFile>,
    stderr: Option<RedirectFile>,
}

impl Redirections {
    fn new() -> Self {
        Self {
            stdout: None,
            stderr: None,
        }
    }

    pub fn write_stdout(&self, output: &str) -> Result<(), std::io::Error> {
        match &self.stdout {
            Some(redirect) => {
                let mut file = redirect.open()?;
                file.write_all(output.as_bytes())
            }
            None => {
                let mut stdout = std::io::stdout();
                stdout.write_all(output.as_bytes())
            }
        }
    }

    pub fn open_stdout(&self) -> Result<Option<File>, std::io::Error> {
        match &self.stdout {
            Some(redirect) => Ok(Some(redirect.open()?)),
            None => Ok(None),
        }
    }

    pub fn write_stderr(&self, output: &str) -> Result<(), std::io::Error> {
        match &self.stderr {
            Some(redirect) => {
                let mut file = redirect.open()?;
                file.write_all(output.as_bytes())
            }
            None => {
                let mut stderr = std::io::stderr();
                stderr.write_all(output.as_bytes())
            }
        }
    }

    pub fn open_stderr(&self) -> Result<Option<File>, std::io::Error> {
        match &self.stderr {
            Some(redirect) => Ok(Some(redirect.open()?)),
            None => Ok(None),
        }
    }
}

fn redirect_operator(token: &str) -> Option<(bool, RedirectMode)> {
    match token {
        ">" | "1>" => Some((true, RedirectMode::Overwrite)),
        ">>" | "1>>" => Some((true, RedirectMode::Append)),
        "2>" => Some((false, RedirectMode::Overwrite)),
        "2>>" => Some((false, RedirectMode::Append)),
        _ => None,
    }
}

pub fn split_redirections(tokens: &[String]) -> Result<(Vec<String>, Redirections), String> {
    let mut command = Vec::new();
    let mut redirections = Redirections::new();
    let mut index = 0;

    while index < tokens.len() {
        if let Some((stdout, mode)) = redirect_operator(&tokens[index]) {
            let filename = tokens.get(index + 1).ok_or_else(|| {
                format!("syntax error: expected filename after '{}'", tokens[index])
            })?;

            if redirect_operator(filename).is_some() {
                return Err("syntax error: expected filename after redirect".to_string());
            }

            let target = if stdout {
                &mut redirections.stdout
            } else {
                &mut redirections.stderr
            };

            if target.is_some() {
                return Err(if stdout {
                    "syntax error: multiple stdout redirects".to_string()
                } else {
                    "syntax error: multiple stderr redirects".to_string()
                });
            }

            *target = Some(RedirectFile {
                filename: filename.clone(),
                mode,
            });
            index += 2;
        } else {
            command.push(tokens[index].clone());
            index += 1;
        }
    }

    if command.is_empty() && (redirections.stdout.is_some() || redirections.stderr.is_some()) {
        return Err("syntax error: expected command before redirect".to_string());
    }

    Ok((command, redirections))
}

#[cfg(test)]
mod tests {
    use super::split_redirections;

    #[test]
    fn splits_stdout_redirect() {
        let tokens = ["echo", "hello", ">", "output.txt"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        let (command, redirections) = split_redirections(&tokens).unwrap();

        assert_eq!(command, ["echo", "hello"]);
        assert!(redirections.stdout.is_some());
    }

    #[test]
    fn splits_stderr_redirect() {
        let tokens = ["ls", "missing", "2>", "errors.txt"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        let (command, redirections) = split_redirections(&tokens).unwrap();

        assert_eq!(command, ["ls", "missing"]);
        assert!(redirections.stderr.is_some());
    }

    #[test]
    fn splits_append_redirects() {
        let tokens = ["echo", "hello", ">>", "output.txt", "2>>", "errors.txt"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        let (command, redirections) = split_redirections(&tokens).unwrap();

        assert_eq!(command, ["echo", "hello"]);
        assert!(redirections.stdout.is_some());
        assert!(redirections.stderr.is_some());
    }
}
