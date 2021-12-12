use crate::utils::StringUtils;
use enum_map::{enum_map, Enum, EnumMap};
use regex::Regex;

#[derive(Debug, PartialEq, Enum)]
pub enum TokenType {
    EndLineIndicator,

    ModuleKeyword,
    ExportKeyword,
    SerialKeyword,
    FunctionKeyword,
    TrueKeyword,
    FalseKeyword,

    Comment,
    Name,
    Integer,
    Float,
    String,

    OpenParenthesesSymbol,
    CloseParenthesesSymbol,
    OpenBracketsSymbol,
    CloseBracketsSymbol,

    CheckEqualsSymbol,
    AssignEqualsSymbol,
    ColonSymbol,
    PeriodSymbol,
    PlusSymbol,
    MinusSymbol,
    MultiplySymbol,
    DivideSymbol,
    ModulusSymbol,

    Unknown,
    ProgramEnd,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub col: usize,
    pub contents: String,
}

fn position_to_line_col(position: usize, code: &String) -> (usize, usize) {
    let previous = &code[0..position];
    let line = previous.chars().filter(|&c| c == '\n').count();
    let mut col = position - previous.rfind('\n').unwrap_or(0);
    if line > 0 {
        col -= 1;
    }

    return (line, col);
}

pub fn tokenize(code: &String) -> Vec<Token> {
    let token_regex: EnumMap<TokenType, Regex> = enum_map! {
        TokenType::EndLineIndicator => Regex::new(r"^[\r\t ]*?((\n).*)").unwrap(),

        TokenType::ModuleKeyword => Regex::new(r"^\s*((module)\b.*)").unwrap(),
        TokenType::ExportKeyword => Regex::new(r"^\s*((export)\b.*)").unwrap(),
        TokenType::SerialKeyword => Regex::new(r"^\s*((serial)\b.*)").unwrap(),
        TokenType::FunctionKeyword => Regex::new(r"^\s*((function)\b.*)").unwrap(),
        TokenType::TrueKeyword => Regex::new(r"^\s*((true)\b.*)").unwrap(),
        TokenType::FalseKeyword => Regex::new(r"^\s*((false)\b.*)").unwrap(),

        TokenType::Comment => Regex::new(r"^\s*((\#.*).*)").unwrap(),
        TokenType::Name => Regex::new(r"^\s*(([a-zA-Z_][a-zA-Z0-9_]*)\b.*)").unwrap(),
        TokenType::Integer => Regex::new(r"^\s*(([0-9]+).*)").unwrap(),
        TokenType::Float => Regex::new(r"^\s*(([0-9]+\.[0-9]*)\b.*)").unwrap(),
        TokenType::String => Regex::new(r#"^\s*((".*?"|'.*?').*)"#).unwrap(),

        TokenType::OpenParenthesesSymbol => Regex::new(r"^\s*((\().*)").unwrap(),
        TokenType::CloseParenthesesSymbol => Regex::new(r"^\s*((\)).*)").unwrap(),
        TokenType::OpenBracketsSymbol => Regex::new(r"^\s*((\[).*)").unwrap(),
        TokenType::CloseBracketsSymbol => Regex::new(r"^\s*((\]).*)").unwrap(),

        TokenType::CheckEqualsSymbol => Regex::new(r"^\s*((==).*)").unwrap(),
        TokenType::AssignEqualsSymbol => Regex::new(r"^\s*((=).*)").unwrap(),
        TokenType::ColonSymbol => Regex::new(r"^\s*((:).*)").unwrap(),
        TokenType::PeriodSymbol => Regex::new(r"^\s*((\.).*)").unwrap(),
        TokenType::PlusSymbol => Regex::new(r"^\s*((\+).*)").unwrap(),
        TokenType::MinusSymbol => Regex::new(r"^\s*((\-).*)").unwrap(),
        TokenType::MultiplySymbol => Regex::new(r"^\s*((\*).*)").unwrap(),
        TokenType::DivideSymbol => Regex::new(r"^\s*((/).*)").unwrap(),
        TokenType::ModulusSymbol => Regex::new(r"^\s*((%).*)").unwrap(),

        TokenType::Unknown => Regex::new(r"^\s*((.*)(\s|\b).*)").unwrap(),
        TokenType::ProgramEnd => Regex::new(r"^\s*$").unwrap(),
    };

    let mut walk = code.clone();
    let mut tokens: Vec<Token> = vec![];

    let mut position = 0;
    loop {
        for (token_type, regex) in token_regex.iter() {
            let cap = regex.captures(&walk);
            if !cap.is_none() {
                let mut contents: &str = "";
                let mut start_pos = 0;
                let mut end_pos = 0;

                let contents_op = cap.unwrap().get(2);
                if !contents_op.is_none() {
                    let contents_un = contents_op.unwrap();
                    start_pos = contents_un.start();
                    end_pos = contents_un.end();
                    contents = contents_un.as_str();
                }

                let (line, col) = position_to_line_col(position + start_pos, &code);
                let token = Token {
                    token_type: token_type,
                    line: line,
                    col: col,
                    contents: String::from(contents),
                };
                tokens.push(token);

                walk = walk.substring(end_pos, walk.len() - end_pos);
                position += end_pos;
                break;
            }
        }

        let last = &tokens.last().unwrap().token_type;
        if last == &TokenType::ProgramEnd {
            break;
        }
    }

    return tokens;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_hello_world() {
        const HELLO_WORLD_PROGRAM: &str = r#"export serial function main(args: string[])
  greeting = 'Hello '+args[0]+'!'

  serial std:println(greeting)
"#;

        let token_list: Vec<Token> = tokenize(&String::from(HELLO_WORLD_PROGRAM));

        let tokens: Vec<Token> = vec![
            Token {
                token_type: TokenType::ExportKeyword,
                line: 0,
                col: 0,
                contents: String::from("export"),
            },
            Token {
                token_type: TokenType::SerialKeyword,
                line: 0,
                col: 7,
                contents: String::from("serial"),
            },
            Token {
                token_type: TokenType::FunctionKeyword,
                line: 0,
                col: 14,
                contents: String::from("function"),
            },
            Token {
                token_type: TokenType::Name,
                line: 0,
                col: 23,
                contents: String::from("main"),
            },
            Token {
                token_type: TokenType::OpenParenthesesSymbol,
                line: 0,
                col: 27,
                contents: String::from("("),
            },
            Token {
                token_type: TokenType::Name,
                line: 0,
                col: 28,
                contents: String::from("args"),
            },
            Token {
                token_type: TokenType::ColonSymbol,
                line: 0,
                col: 32,
                contents: String::from(":"),
            },
            Token {
                token_type: TokenType::Name,
                line: 0,
                col: 34,
                contents: String::from("string"),
            },
            Token {
                token_type: TokenType::OpenBracketsSymbol,
                line: 0,
                col: 40,
                contents: String::from("["),
            },
            Token {
                token_type: TokenType::CloseBracketsSymbol,
                line: 0,
                col: 41,
                contents: String::from("]"),
            },
            Token {
                token_type: TokenType::CloseParenthesesSymbol,
                line: 0,
                col: 42,
                contents: String::from(")"),
            },
            Token {
                token_type: TokenType::EndLineIndicator,
                line: 0,
                col: 43,
                contents: String::from("\n"),
            },
            Token {
                token_type: TokenType::Name,
                line: 1,
                col: 2,
                contents: String::from("greeting"),
            },
            Token {
                token_type: TokenType::AssignEqualsSymbol,
                line: 1,
                col: 11,
                contents: String::from("="),
            },
            Token {
                token_type: TokenType::String,
                line: 1,
                col: 13,
                contents: String::from("'Hello '"),
            },
            Token {
                token_type: TokenType::PlusSymbol,
                line: 1,
                col: 21,
                contents: String::from("+"),
            },
            Token {
                token_type: TokenType::Name,
                line: 1,
                col: 22,
                contents: String::from("args"),
            },
            Token {
                token_type: TokenType::OpenBracketsSymbol,
                line: 1,
                col: 26,
                contents: String::from("["),
            },
            Token {
                token_type: TokenType::Integer,
                line: 1,
                col: 27,
                contents: String::from("0"),
            },
            Token {
                token_type: TokenType::CloseBracketsSymbol,
                line: 1,
                col: 28,
                contents: String::from("]"),
            },
            Token {
                token_type: TokenType::PlusSymbol,
                line: 1,
                col: 29,
                contents: String::from("+"),
            },
            Token {
                token_type: TokenType::String,
                line: 1,
                col: 30,
                contents: String::from("'!'"),
            },
            Token {
                token_type: TokenType::EndLineIndicator,
                line: 1,
                col: 33,
                contents: String::from("\n"),
            },
            Token {
                token_type: TokenType::EndLineIndicator,
                line: 2,
                col: 0,
                contents: String::from("\n"),
            },
            Token {
                token_type: TokenType::SerialKeyword,
                line: 3,
                col: 2,
                contents: String::from("serial"),
            },
            Token {
                token_type: TokenType::Name,
                line: 3,
                col: 9,
                contents: String::from("std"),
            },
            Token {
                token_type: TokenType::ColonSymbol,
                line: 3,
                col: 12,
                contents: String::from(":"),
            },
            Token {
                token_type: TokenType::Name,
                line: 3,
                col: 13,
                contents: String::from("println"),
            },
            Token {
                token_type: TokenType::OpenParenthesesSymbol,
                line: 3,
                col: 20,
                contents: String::from("("),
            },
            Token {
                token_type: TokenType::Name,
                line: 3,
                col: 21,
                contents: String::from("greeting"),
            },
            Token {
                token_type: TokenType::CloseParenthesesSymbol,
                line: 3,
                col: 29,
                contents: String::from(")"),
            },
            Token {
                token_type: TokenType::EndLineIndicator,
                line: 3,
                col: 30,
                contents: String::from("\n"),
            },
            Token {
                token_type: TokenType::ProgramEnd,
                line: 4,
                col: 0,
                contents: String::from(""),
            },
        ];

        assert_eq!(token_list, tokens);
    }

    #[test]
    fn unknown_characters() {
        const HELLO_WORLD_PROGRAM: &str = "export serial function @{main\n";

        let token_list: Vec<Token> = tokenize(&String::from(HELLO_WORLD_PROGRAM));

        let tokens: Vec<Token> = vec![
            Token {
                token_type: TokenType::ExportKeyword,
                line: 0,
                col: 0,
                contents: String::from("export"),
            },
            Token {
                token_type: TokenType::SerialKeyword,
                line: 0,
                col: 7,
                contents: String::from("serial"),
            },
            Token {
                token_type: TokenType::FunctionKeyword,
                line: 0,
                col: 14,
                contents: String::from("function"),
            },
            Token {
                token_type: TokenType::Unknown,
                line: 0,
                col: 23,
                contents: String::from("@{main"),
            },
            Token {
                token_type: TokenType::EndLineIndicator,
                line: 0,
                col: 29,
                contents: String::from("\n"),
            },
            Token {
                token_type: TokenType::ProgramEnd,
                line: 1,
                col: 0,
                contents: String::from(""),
            },
        ];

        assert_eq!(token_list, tokens);
    }
}
