pub fn parse_command(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut token_started = false;
    let mut quote: Option<char> = None;
    

    for character  in input.chars() {

        match quote {
            Some(active_quote) => {
                if character == active_quote {
                    quote = None;
                } else {
                    token.push(character);
                }
            }

            None => match character {
                '\'' | '"' => {
                    quote = Some(character);
                    token_started = true;
                }

                character if character.is_whitespace() => {
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

    }


    if let Some(quote) = quote {
        let quote_name = if quote == '\'' { "single" } else { "double" };
        return Err(format!("unterminated {quote_name} quote"));
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
