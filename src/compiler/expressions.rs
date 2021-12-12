use crate::compiler::tokenization::{Token, TokenType};
use crate::core::types::Module;

struct TokenWalk<'a> {
    tokens: &'a Vec<Token>,
    position: usize,
}

impl TokenWalk<'_> {
    fn remaining(&self) -> usize {
        return self.tokens.len() - self.position;
    }

    fn next(&mut self) -> &Token {
        let token = &self.tokens[self.position];
        self.position += 1;
        return token;
    }

    fn peek(&self) -> &Token {
        return &self.tokens[self.position];
    }
}

pub fn build_expressions(tokens: &Vec<Token>) -> Vec<Module> {
    let mut walk = TokenWalk {
        tokens: &tokens,
        position: 0,
    };
    return try_parse_root(&mut walk);
}

fn try_parse_root(walk: &mut TokenWalk) -> Vec<Module> {
    let mut expressions: Vec<Module> = vec![];

    while walk.remaining() > 0 {
        let token = walk.next();
        match token.token_type {
            TokenType::ModuleKeyword => expressions.push(try_parse_module(walk)),
            _ => panic!(
                "Unexpected token: '{}' as location {}:{}. Expected: 'module'",
                token.contents, token.line, token.col
            ),
        }
    }

    return expressions;
}

fn try_parse_module(walk: &mut TokenWalk) -> Module {
    let name_token = walk.next();
    if name_token.token_type != TokenType::Name {
        panic!(
            "Unexpected token: '{}' as location {}:{}. Expected module name.",
            name_token.contents, name_token.line, name_token.col
        );
    }

    let module = Module {
        name: name_token.contents.clone(),
        functions: vec![],
        data_types: vec![],
    };

    return module;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn unexpected_root_keyword() {
        let tokens: Vec<Token> = vec![Token {
            token_type: TokenType::ExportKeyword,
            line: 0,
            col: 0,
            contents: String::from("export"),
        }];

        build_expressions(&tokens);
    }
}
