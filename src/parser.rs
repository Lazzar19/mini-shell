pub fn parse_command(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut token_started = false;
    let mut in_single_quotes = false;

    for character in input.chars() {
        match character {
            '\'' => {
                in_single_quotes = !in_single_quotes;
                token_started = true;
            }
            character if character.is_whitespace() && !in_single_quotes => {
                if token_started {
                    tokens.push(std::mem::take(&mut token));
                    token_started = false;
                }
            }
            character => {
                token.push(character);
                token_started = true;
            }
        }
    }

    if in_single_quotes {
        return Err("unterminated single quote".to_string());
    }

    if token_started {
        tokens.push(token);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::parse_command;

    #[test]
    fn parses_single_quoted_whitespace_as_one_token() {
        assert_eq!(
            parse_command("echo 'hello      world'").unwrap(),
            vec!["echo", "hello      world"]
        );
    }

    #[test]
    fn preserves_empty_single_quoted_token() {
        assert_eq!(parse_command("echo ''").unwrap(), vec!["echo", ""]);
    }

    #[test]
    fn joins_quoted_and_unquoted_text() {
        assert_eq!(
            parse_command("echo 'hello'world").unwrap(),
            vec!["echo", "helloworld"]
        );
    }

    #[test]
    fn reports_unterminated_single_quote() {
        assert!(parse_command("echo 'hello").is_err());
    }
}
