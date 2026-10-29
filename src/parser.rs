pub fn parse_command(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut token_started = false;
    let mut quote: Option<char> = None;
    let mut characters = input.chars().peekable();    

    while let Some(character) = characters.next() {
        match quote {
            Some('\'') => {
                if character == '\'' {
                    quote = None;
                } else {
                    token.push(character);
                }
            }
            
            Some('"') => match character {
                '"' => quote = None,
                '\\' => match characters.peek().copied() {
                    Some('$' | '`' | '"' | '\\') => {
                        token.push(characters.next().unwrap());
                    }
                    Some(_) | None => token.push('\\'),
                }
                character => token.push(character),
            }

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

                '\\' => {
                    token_started = true;
                    if let Some(escaped_char) = characters.next() {
                        token.push(escaped_char);
                    } else {
                        token.push('\\');
                    }
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


        #[test]
    fn removes_backslash_outside_quotes() {
        assert_eq!(
            parse_command(r#"echo hello\ world"#).unwrap(),
            vec!["echo", "hello world"]
        );
    }

    #[test]
    fn preserves_backslash_inside_single_quotes() {
        assert_eq!(
            parse_command(r#"echo 'hello\ world'"#).unwrap(),
            vec!["echo", r#"hello\ world"#]
        );
    }
    
    #[test]
    fn handles_escapes_inside_double_quotes() {
        assert_eq!(
            parse_command(r#"echo "hello\"world\\\$""#).unwrap(),
            vec!["echo", r#"hello"world\$"#]
        );
    }
}
