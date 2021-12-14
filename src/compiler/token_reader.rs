/// A function type that takes in a code string and tries to parse that token
/// currently at the provided char index within the string. This function will
/// return a DiscoveredToken if it is capable of parsing the token, or None if
/// it is unable to correctly parse the token. The length of the token is also
/// returned as the second argument within the tuple.
type TokenHandler = fn(code: &str, char_index: usize) -> Option<(TokenType, usize)>;

/// A list of token handler functions that are available for parsing Vertex code
/// strings.
fn get_handlers() -> Vec<TokenHandler> {
    return vec![
        number_token_handler,
        name_token_handler,
        string_token_handler,

        |code, char_index| symbol_token_handler(code, char_index, "<=", TokenType::LTEConditional),
        |code, char_index| symbol_token_handler(code, char_index, ">=", TokenType::GTEConditional),
        |code, char_index| symbol_token_handler(code, char_index, "<", TokenType::LTConditional),
        |code, char_index| symbol_token_handler(code, char_index, ">", TokenType::GTConditional),
        |code, char_index| symbol_token_handler(code, char_index, "==", TokenType::EqualConditional),

        |code, char_index| symbol_token_handler(code, char_index, "\n", TokenType::NewLine),
        |code, char_index| symbol_token_handler(code, char_index, "=", TokenType::VariableAssignment),
    ];
}

/// An enum that defines how the content within the token contents should be
/// interpreted.
#[derive(Debug, PartialEq)]
pub enum TokenType {
    String(String),
    Integer(i64),
    Float(f64),

    LTEConditional,
    GTEConditional,
    LTConditional,
    GTConditional,
    EqualConditional,

    NewLine,
    VariableAssignment,

    Name(String),
    ExportKeyword,
    FunctionKeyword,
}

/// A wrapper for token types that also contain the location information of the
/// tken within the code string it was generated from.
#[derive(Debug, PartialEq)]
pub struct Token {
    token_type: TokenType,
    line: usize,
    col: usize,
}

/// Trys to parse the next available token within the provided source code
/// string, starting at the given char index. Any leading whitespace chars are
/// skipped. The number of skipped characters is returned as the second argument
/// within the returned tuple.
pub fn read_token(code: &str, char_index: usize) -> Option<(TokenType, usize, usize)> {
    let skipped = skip_whitespace(&code, char_index);
    if skipped == None {
        return None;
    }

    let token = try_token_handlers(&code, char_index + skipped.unwrap());
    if token == None {
        return None;
    }

    let (t, len) = token.unwrap();
    return Some((t, skipped.unwrap(), len));
}

/// Reads over all tokens within the provided code string and returns them in
/// an ordered vector list.
pub fn read_all_tokens(code: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut char_index = 0;

    let mut line = 0;
    let mut col = 0;

    loop {
        let next = read_token(&code, char_index);
        if next == None {
            return tokens;
        }

        let (t, skipped, len) = next.unwrap();
        let new_line = t == TokenType::NewLine;

        col += skipped;
        tokens.push(Token {
            token_type: t,
            line: line,
            col: col,
        });

        col += len;
        char_index += skipped + len;

        if new_line {
            line += 1;
            col = 0;
        }
    }
}

/// Counts the number of whitespace characters that need to be skipped in order
/// to find the next available token, starting at the provided char index within
/// the code string.
fn skip_whitespace(code: &str, char_index: usize) -> Option<usize> {
    let mut chars = code.chars().skip(char_index);
    let mut skipped = 0;

    loop {
        let c = chars.next();
        if c == None {
            return None;
        }

        if c == Some('\r') || c == Some('\t') || c == Some(' ') {
            skipped += 1;
        } else {
            return Some(skipped);
        }
    }
}

/// Iterates over all token handlers in order and attempts to parse the token at
/// the current char index. If a handler returns a null value, the next handler
/// in the list is attempted. Otherwise, the parsed token type is returned. The
/// length of the token is also returns as the second argument within the tuple.
fn try_token_handlers(code: &str, char_index: usize) -> Option<(TokenType, usize)> {
    for handler in get_handlers() {
        let token = handler(&code, char_index);
        if token != None {
            return token;
        }
    }

    return None;
}

/// Tries to parse the token as a known keyword or a generic identifier name.
fn name_token_handler(code: &str, char_index: usize) -> Option<(TokenType, usize)> {
    let mut chars = code.chars().skip(char_index);

    let mut len = 0;
    loop {
        let c = chars.next();
        if c == None {
            break;
        }

        match c.unwrap() {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => len += 1,
            _ => break,
        }
    }

    if len == 0 {
        return None;
    }

    let content = &code[char_index..char_index + len];
    return Some(match content {
        "export" => (TokenType::ExportKeyword, 6),
        "function" => (TokenType::FunctionKeyword, 8),
        _ => (TokenType::Name(String::from(content)), content.len()),
    });
}

/// Tries to parse a string token surrounded by either single quotes or double
/// quotes.
fn string_token_handler(code: &str, char_index: usize) -> Option<(TokenType, usize)> {
    let mut chars = code.chars().skip(char_index);

    let string_char = chars.next();
    if string_char != Some('"') && string_char != Some('\'') {
        return None;
    }

    let mut skip_next = false;
    let mut len = 1;
    loop {
        let c = chars.next();
        if c == None {
            panic!(
                "Unexpected end of file! Expected closing string character: {}",
                string_char.unwrap()
            );
        }

        len += 1;
        if skip_next {
            skip_next = false;
        } else {
            if c == string_char {
                return Some((TokenType::String(String::from(
                    &code[char_index..char_index + len],
                )), len));
            } else if c == Some('\\') {
                skip_next = true;
            }
        }
    }
}

/// Tries to read and parse the next available token as a number. The token is
/// returned as a float if the number contains a decimal point and is returned
/// as an integer if it does not.
fn number_token_handler(code: &str, char_index: usize) -> Option<(TokenType, usize)> {
    let mut chars = code.chars().skip(char_index);

    let mut len = 0;
    loop {
        let c = chars.next();
        if c == None {
            break;
        }

        match c.unwrap() {
            '0'..='9' | '.' | '-' => len += 1,
            _ => break,
        }
    }

    let content = String::from(&code[char_index..char_index + len]);

    match content.parse::<i64>() {
        Ok(d) => return Some((TokenType::Integer(d), content.len())),
        _ => {}
    }

    match content.parse::<f64>() {
        Ok(d) => return Some((TokenType::Float(d), content.len())),
        _ => {}
    }

    return None;
}

fn symbol_token_handler(code: &str, char_index: usize, symbol: &str, token_type: TokenType) -> Option<(TokenType, usize)> {
    let mut source = code.chars().skip(char_index);
    let mut target = symbol.chars();

    loop {
        let source_char = source.next();
        let target_char = target.next();

        if target_char == None {
            return Some((token_type, symbol.len()));
        }

        if target_char != source_char {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_generic_name() {
        let code = "  apple7  ";
        let token = read_token(&code, 0);

        assert_eq!(
            token,
            Some((TokenType::Name(String::from("apple7")), 2usize, 6usize))
        );
    }

    #[test]
    fn parse_function_keyword() {
        let code = "export function my_func()";
        let token = read_token(&code, 6);

        assert_eq!(token, Some((TokenType::FunctionKeyword, 1usize, 8usize)));
    }

    #[test]
    fn parse_string_single_quotes() {
        let code = "name = 'Hello, world!' + '..'";
        let token = read_token(&code, 6);

        assert_eq!(
            token,
            Some((TokenType::String(String::from("'Hello, world!'")), 1usize, 15usize))
        );
    }

    #[test]
    #[should_panic]
    fn parse_string_mismatched_quotes() {
        let code = "'Hello, world!\"";
        read_token(&code, 0);
    }

    #[test]
    fn parse_integer() {
        let code = "123";
        let token = read_token(&code, 0);

        assert_eq!(token, Some((TokenType::Integer(123i64), 0usize, 3usize)));
    }

    #[test]
    fn parse_float() {
        let code = "10.5";
        let token = read_token(&code, 0);

        assert_eq!(token, Some((TokenType::Float(10.5f64), 0usize, 4usize)));
    }

    #[test]
    fn parse_all_tokens() {
        let code = "export function hello_world\n  'hi'";
        let tokens = read_all_tokens(&code);

        assert_eq!(tokens, vec![
            Token { token_type: TokenType::ExportKeyword, line: 0, col: 0 },
            Token { token_type: TokenType::FunctionKeyword, line: 0, col: 7 },
            Token { token_type: TokenType::Name(String::from("hello_world")), line: 0, col: 16 },
            Token { token_type: TokenType::NewLine, line: 0, col: 27 },
            Token { token_type: TokenType::String(String::from("'hi'")), line: 1, col: 2 },
        ]);
    }
}
