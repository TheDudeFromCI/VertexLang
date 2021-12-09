use crate::utils::StringUtils;
use enum_map::{enum_map, Enum, EnumMap};
use regex::Regex;

#[derive(Debug, PartialEq, Enum)]
pub enum TokenType {
    EndLineIndicator,

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
    token_type: TokenType,
    contents: String,
}

pub fn tokenize(code: &String) -> Vec<Token> {
    let token_regex: EnumMap<TokenType, Regex> = enum_map! {
        TokenType::EndLineIndicator => Regex::new(r"^[\r\t ]*?((\n).*)").unwrap(),

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

    loop {
        for (token_type, regex) in token_regex.iter() {
            let cap = regex.captures(&walk);
            if !cap.is_none() {
                let mut contents: &str = "";
                let mut skip = 0;

                let contents_op = cap.unwrap().get(2);
                if !contents_op.is_none() {
                    let contents_un = contents_op.unwrap();
                    skip = contents_un.end();
                    contents = contents_un.as_str();
                }

                let token = Token {
                    token_type: token_type,
                    contents: String::from(contents),
                };
                tokens.push(token);

                walk = walk.substring(skip, walk.len() - skip);
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
        const HELLO_WORLD_PROGRAM: &str = "export serial function main(args: string[])\n\
                                             greeting = 'Hello '+args[0]+'!'\n\n\
                                             serial std:println(greeting)\n";

        let token_list: Vec<Token> = tokenize(&String::from(HELLO_WORLD_PROGRAM));

        let tokens: Vec<Token> = vec![
            Token {
                token_type: TokenType::ExportKeyword,
                contents: String::from("export"),
            },
            Token {
                token_type: TokenType::SerialKeyword,
                contents: String::from("serial"),
            },
            Token {
                token_type: TokenType::FunctionKeyword,
                contents: String::from("function"),
            },
            Token {
                token_type: TokenType::Name,
                contents: String::from("main"),
            },
            Token {
                token_type: TokenType::OpenParenthesesSymbol,
                contents: String::from("("),
            },
            Token {
                token_type: TokenType::Name,
                contents: String::from("args"),
            },
            Token {
                token_type: TokenType::ColonSymbol,
                contents: String::from(":"),
            },
            Token {
                token_type: TokenType::Name,
                contents: String::from("string"),
            },
            Token {
                token_type: TokenType::OpenBracketsSymbol,
                contents: String::from("["),
            },
            Token {
                token_type: TokenType::CloseBracketsSymbol,
                contents: String::from("]"),
            },
            Token {
                token_type: TokenType::CloseParenthesesSymbol,
                contents: String::from(")"),
            },
            Token {
                token_type: TokenType::EndLineIndicator,
                contents: String::from("\n"),
            },
            Token {
                token_type: TokenType::Name,
                contents: String::from("greeting"),
            },
            Token {
                token_type: TokenType::AssignEqualsSymbol,
                contents: String::from("="),
            },
            Token {
                token_type: TokenType::String,
                contents: String::from("'Hello '"),
            },
            Token {
                token_type: TokenType::PlusSymbol,
                contents: String::from("+"),
            },
            Token {
                token_type: TokenType::Name,
                contents: String::from("args"),
            },
            Token {
                token_type: TokenType::OpenBracketsSymbol,
                contents: String::from("["),
            },
            Token {
                token_type: TokenType::Integer,
                contents: String::from("0"),
            },
            Token {
                token_type: TokenType::CloseBracketsSymbol,
                contents: String::from("]"),
            },
            Token {
                token_type: TokenType::PlusSymbol,
                contents: String::from("+"),
            },
            Token {
                token_type: TokenType::String,
                contents: String::from("'!'"),
            },
            Token {
                token_type: TokenType::EndLineIndicator,
                contents: String::from("\n"),
            },
            Token {
                token_type: TokenType::EndLineIndicator,
                contents: String::from("\n"),
            },
            Token {
                token_type: TokenType::SerialKeyword,
                contents: String::from("serial"),
            },
            Token {
                token_type: TokenType::Name,
                contents: String::from("std"),
            },
            Token {
                token_type: TokenType::ColonSymbol,
                contents: String::from(":"),
            },
            Token {
                token_type: TokenType::Name,
                contents: String::from("println"),
            },
            Token {
                token_type: TokenType::OpenParenthesesSymbol,
                contents: String::from("("),
            },
            Token {
                token_type: TokenType::Name,
                contents: String::from("greeting"),
            },
            Token {
                token_type: TokenType::CloseParenthesesSymbol,
                contents: String::from(")"),
            },
            Token {
                token_type: TokenType::EndLineIndicator,
                contents: String::from("\n"),
            },
            Token {
                token_type: TokenType::ProgramEnd,
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
                contents: String::from("export"),
            },
            Token {
                token_type: TokenType::SerialKeyword,
                contents: String::from("serial"),
            },
            Token {
                token_type: TokenType::FunctionKeyword,
                contents: String::from("function"),
            },
            Token {
                token_type: TokenType::Unknown,
                contents: String::from("@{main"),
            },
            Token {
                token_type: TokenType::EndLineIndicator,
                contents: String::from("\n"),
            },
            Token {
                token_type: TokenType::ProgramEnd,
                contents: String::from(""),
            },
        ];

        assert_eq!(token_list, tokens);
    }
}
