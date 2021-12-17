use strum_macros::Display;

/// A function type that takes in a code string and tries to parse that token
/// currently at the provided char index within the string. This function will
/// return a DiscoveredToken if it is capable of parsing the token, or None if
/// it is unable to correctly parse the token. The length of the token is also
/// returned as the second argument within the tuple.
type TokenHandler = fn(code: &str, char_index: usize, line: usize, col: usize) -> Option<TokenContents>;

/// A list of token handler functions that are available for parsing Vertex code
/// strings.
fn get_handlers() -> Vec<TokenHandler> {
    return vec![
        string_token_handler,
        number_token_handler,
        |code, index, _, _| symbol_token_handler(code, index, "<=", TokenType::LTEConditional),
        |code, index, _, _| symbol_token_handler(code, index, ">=", TokenType::GTEConditional),
        |code, index, _, _| symbol_token_handler(code, index, "<", TokenType::LTConditional),
        |code, index, _, _| symbol_token_handler(code, index, ">", TokenType::GTConditional),
        |code, index, _, _| symbol_token_handler(code, index, "==", TokenType::EqConditional),
        |code, index, _, _| symbol_token_handler(code, index, "{", TokenType::OpenBlock),
        |code, index, _, _| symbol_token_handler(code, index, "}", TokenType::CloseBlock),
        |code, index, _, _| symbol_token_handler(code, index, "(", TokenType::OpenParams),
        |code, index, _, _| symbol_token_handler(code, index, ")", TokenType::CloseParams),
        |code, index, _, _| symbol_token_handler(code, index, "[", TokenType::OpenArray),
        |code, index, _, _| symbol_token_handler(code, index, "]", TokenType::CloseArray),
        |code, index, _, _| symbol_token_handler(code, index, "\n", TokenType::NewLine),
        |code, index, _, _| symbol_token_handler(code, index, "=", TokenType::VarAssignment),
        |code, index, _, _| symbol_token_handler(code, index, ":", TokenType::ModuleReference),
        |code, index, _, _| {
            name_token_handler(code, index, vec![
                    ("export", TokenType::ExportKeyword),
                    ("serial", TokenType::SerialKeyword),
                    ("acceleratable", TokenType::AcceleratableKeyword),
                    ("function", TokenType::FunctionKeyword),
                    ("return", TokenType::FunctionKeyword),
                    ("true", TokenType::FunctionKeyword),
                    ("false", TokenType::FunctionKeyword),
                ],
            )
        },
    ];
}

/// An enum that defines how the content within the token contents should be
/// interpreted.
#[derive(Debug, PartialEq, Display, Copy, Clone)]
pub enum TokenType {
    String,
    Integer,
    Float,

    LTEConditional,
    GTEConditional,
    LTConditional,
    GTConditional,
    EqConditional,

    OpenBlock,
    CloseBlock,
    OpenParams,
    CloseParams,
    OpenArray,
    CloseArray,

    NewLine,
    VarAssignment,
    ModuleReference,

    Name,
    ExportKeyword,
    SerialKeyword,
    AcceleratableKeyword,
    FunctionKeyword,
    ReturnKeyword,

    TrueKeyword,
    FalseKeyword,
}

/// A wrapper for token types that also contain the location information of the
/// tken within the code string it was generated from.
#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub contents: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, PartialEq)]
pub struct TokenContents {
    pub token_type: TokenType,
    pub contents: String,
    pub skipped: usize,
    pub len: usize,
}

/// Trys to parse the next available token within the provided source code
/// string, starting at the given char index. Any leading whitespace chars are
/// skipped. The number of skipped characters is returned as the second argument
/// within the returned tuple.
pub fn read_token(code: &str, char_index: usize, line: usize, col: usize) -> Option<TokenContents> {
    let skipped = skip_whitespace(&code, char_index);
    if skipped == None {
        return None;
    }

    let token = try_token_handlers(&code, char_index + skipped.unwrap(), line, col);
    if token == None {
        return None;
    }

    let mut contents = token.unwrap();
    contents.skipped = skipped.unwrap();
    return Some(contents);
}

/// Reads over all tokens within the provided code string and returns them in
/// an ordered vector list.
pub fn read_all_tokens(code: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut char_index = 0;

    let mut line = 0;
    let mut col = 0;

    loop {
        let next = read_token(&code, char_index, line, col);
        if next == None {
            return tokens;
        }

        let contents = next.unwrap();
        let new_line = contents.token_type == TokenType::NewLine;

        col += contents.skipped;
        tokens.push(Token {
            token_type: contents.token_type,
            contents: contents.contents,
            line: line,
            col: col,
        });

        col += contents.len;
        char_index += contents.skipped + contents.len;

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
fn try_token_handlers(code: &str, char_index: usize, line: usize, col: usize) -> Option<TokenContents> {
    for handler in get_handlers() {
        let token = handler(&code, char_index, line, col);
        if token != None {
            return token;
        }
    }

    return None;
}

/// Tries to parse the token as a known keyword or a generic identifier name.
fn name_token_handler(
    code: &str,
    char_index: usize,
    keywords: Vec<(&str, TokenType)>,
) -> Option<TokenContents> {
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
    for (keyword, token_type) in keywords {
        if content.eq(keyword) {
            return Some(TokenContents {
                token_type: token_type,
                contents: String::from(content),
                skipped: 0,
                len: content.len(),
            });
        }
    }

    return Some(TokenContents {
        token_type: TokenType::Name,
        contents: String::from(content),
        skipped: 0,
        len: content.len(),
    });
}

/// Tries to parse a string token surrounded by either single quotes or double
/// quotes.
fn string_token_handler(code: &str, char_index: usize, line: usize, col: usize) -> Option<TokenContents> {
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
                "Unexpected end of file! Expected closing string character: '{}' for string started at {}:{}",
                string_char.unwrap(), line, col
            );
        }

        if c == Some('\n') {
            panic!(
                "Unexpected end of line! Expected closing string character: '{}' for string started at {}:{}",
                string_char.unwrap(), line, col
            );
        }

        len += 1;
        if skip_next {
            skip_next = false;
        } else {
            if c == string_char {
                return Some(TokenContents {
                        token_type: TokenType::String,
                        contents: String::from(&code[char_index..char_index + len]),
                        skipped: 0,
                        len: len,
                    });
            } else if c == Some('\\') {
                skip_next = true;
            }
        }
    }
}

/// Tries to read and parse the next available token as a number. The token is
/// returned as a float if the number contains a decimal point and is returned
/// as an integer if it does not.
fn number_token_handler(code: &str, char_index: usize, _line: usize, _col: usize) -> Option<TokenContents> {
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
        Ok(d) => return Some(TokenContents {
            token_type: TokenType::Integer,
            contents: content,
            skipped: 0,
            len: len,
        }),
        _ => {}
    }

    match content.parse::<f64>() {
        Ok(d) => return Some(TokenContents {
            token_type: TokenType::Float,
            contents: content,
            skipped: 0,
            len: len,
        }),
        _ => {}
    }

    return None;
}

fn symbol_token_handler(
    code: &str,
    char_index: usize,
    symbol: &str,
    token_type: TokenType,
) -> Option<TokenContents> {
    let mut source = code.chars().skip(char_index);
    let mut target = symbol.chars();

    loop {
        let target_char = target.next();
        if target_char == None {
            return Some(TokenContents {
                token_type: token_type,
                contents: String::from(symbol),
                skipped: 0,
                len: symbol.len(),
            });
        }

        if target_char != source.next() {
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
        let token = read_token(&code, 0, 0, 0);

        assert_eq!(
            token,
            Some(TokenContents {
                token_type: TokenType::Name,
                contents: String::from("apple7"),
                skipped: 2,
                len: 6,
            }),
        );
    }

    #[test]
    fn parse_function_keyword() {
        let code = "export function my_func()";
        let token = read_token(&code, 6, 0, 6);

        assert_eq!(token, Some(TokenContents {
            token_type: TokenType::FunctionKeyword,
            contents: String::from("function"),
            skipped: 1,
            len: 8,
        }));
    }

    #[test]
    fn parse_string_single_quotes() {
        let code = "name = 'Hello, world!' + '..'";
        let token = read_token(&code, 6, 0, 6);

        assert_eq!(
            token,
            Some(TokenContents {
                token_type: TokenType::String,
                contents: String::from("'Hello, world!'"),
                skipped: 1,
                len: 15,
            })
        );
    }

    #[test]
    #[should_panic]
    fn parse_string_mismatched_quotes() {
        let code = "'Hello, world!\"";
        read_token(&code, 0, 0, 0);
    }

    #[test]
    fn parse_integer() {
        let code = "123";
        let token = read_token(&code, 0, 0, 0);

        assert_eq!(token, Some(TokenContents{
            token_type: TokenType::Integer,
            contents: String::from("123"),
            skipped: 0,
            len: 3,
        }));
    }

    #[test]
    fn parse_float() {
        let code = "10.5";
        let token = read_token(&code, 0, 0, 0);

        assert_eq!(token, Some(TokenContents{
            token_type: TokenType::Float,
            contents: String::from("10.5"),
            skipped: 0,
            len: 4,
        }));
    }

    #[test]
    fn parse_all_tokens() {
        let code = "export function hello_world\n  'hi'";
        let tokens = read_all_tokens(&code);

        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::ExportKeyword,
                    contents: String::from("export"),
                    line: 0,
                    col: 0
                },
                Token {
                    token_type: TokenType::FunctionKeyword,
                    contents: String::from("function"),
                    line: 0,
                    col: 7
                },
                Token {
                    token_type: TokenType::Name,
                    contents: String::from("hello_world"),
                    line: 0,
                    col: 16
                },
                Token {
                    token_type: TokenType::NewLine,
                    contents: String::from("\n"),
                    line: 0,
                    col: 27
                },
                Token {
                    token_type: TokenType::String,
                    contents: String::from("'hi'"),
                    line: 1,
                    col: 2
                },
            ]
        );
    }
}
