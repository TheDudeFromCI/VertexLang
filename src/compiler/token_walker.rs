use crate::compiler::context::{Context, ModulePath};
use crate::compiler::token_reader::Token;
use crate::compiler::token_reader::TokenType;
use crate::{continue_if, return_if};

pub struct TokenWalk<'a, 'b> {
    tokens: &'a Vec<Token>,
    context: &'b mut Context,
    module_path: ModulePath,
    stack: Vec<TokenStack>,
}

struct TokenStack {
    pos: usize,
}

impl TokenWalk<'_, '_> {
    fn new<'a, 'b>(
        tokens: &'a Vec<Token>,
        context: &'b mut Context,
        module_path: ModulePath,
    ) -> TokenWalk<'a, 'b> {
        return TokenWalk {
            tokens: tokens,
            context: context,
            module_path: module_path,
            stack: vec![TokenStack { pos: 0 }],
        };
    }

    pub fn get_pos(&self) -> usize {
        return self.stack[self.stack.len() - 1].pos;
    }

    pub fn push_stack(&mut self) {
        let pos = self.get_pos();
        self.stack.push(TokenStack { pos: pos });
    }

    pub fn get_stack_depth(&self) -> usize {
        return self.stack.len() - 1;
    }

    pub fn pop_stack(&mut self, apply: bool) {
        if self.stack.len() <= 1 {
            panic!("Unexpected state! Tried to pop the stack too many times!");
        }

        let pos = self.stack.pop().unwrap().pos;
        if apply {
            let depth = self.get_stack_depth();
            self.stack[depth].pos = pos;
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        if self.has_remaining() {
            return Some(&self.tokens[self.get_pos()]);
        } else {
            return None;
        }
    }

    pub fn next(&mut self) -> Option<&Token> {
        if self.has_remaining() {
            let pos = self.get_pos();
            let depth = self.get_stack_depth();
            self.stack[depth].pos += 1;
            return Some(&self.tokens[pos]);
        } else {
            return None;
        }
    }

    pub fn remaining(&self) -> usize {
        return self.tokens.len() - self.get_pos();
    }

    pub fn has_remaining(&self) -> bool {
        return self.remaining() > 0;
    }
}

pub fn evaluate_token_list(
    context: &mut Context,
    module_name: &str,
    tokens: &Vec<Token>,
) -> Result<(), String> {
    let path = context.add_module(String::from(module_name)).path;
    let mut walk = TokenWalk::new(tokens, context, path);
    return try_parse_root(&mut walk);
}

fn try_parse_root(walk: &mut TokenWalk) -> Result<(), String> {
    loop {
        if !walk.has_remaining() {
            return Ok(());
        }

        let parse = try_parse_oneof(walk, &[try_parse_exportable()]);

        return_if!(parse.is_err(), Err(parse.unwrap_err()));
        continue_if!(parse.unwrap());
    }
}

type ParsingFunction = Box<dyn Fn(&mut TokenWalk) -> Result<bool, String>>;

fn try_parse_oneof(walk: &mut TokenWalk, funcs: &[ParsingFunction]) -> Result<bool, String> {
    for func in funcs {
        walk.push_stack();
        let parsed_func = func(walk);
        return_if!(parsed_func.is_err(), Err(parsed_func.unwrap_err()));
        if parsed_func.unwrap() {
            walk.pop_stack(true);
            return Ok(true);
        } else {
            walk.pop_stack(false);
        }
    }

    if !walk.has_remaining() {
        panic!("Unexpected state! Walk not cleaned up correctly.");
    }

    let token = walk.next().unwrap();
    return Err(format!(
        "Unexpected token {} at {}:{}",
        token.token_type, token.line, token.col,
    ));
}

fn compare_and_advance(walk: &mut TokenWalk, token_type: TokenType) -> bool {
    let token = walk.peek();
    if token.is_none() || token.unwrap().token_type != token_type {
        return false;
    }

    walk.next();
    return true;
}

fn expect_token(walk: &mut TokenWalk, token_type: TokenType) -> Result<String, String> {
    let token = walk.next();
    if token.is_none() {
        return Err(format!("Unexpected end of file! Expected: '{}'", token_type));
    }

    let t = &token.unwrap().token_type;
    if *t != token_type {
        return Err(format!("Unexpected token: '{}'! Expected: '{}'", t, token_type));
    }

    return Ok(token.unwrap().contents.clone());
}

fn try_parse_exportable() -> ParsingFunction {
    Box::new(move |walk: &mut TokenWalk| {
        let export_keyword = compare_and_advance(walk, TokenType::ExportKeyword);
        return try_parse_oneof(walk, &[try_parse_function(export_keyword)]);
    })
}

fn try_parse_function(exportable: bool) -> ParsingFunction {
    Box::new(move |walk: &mut TokenWalk| {
        walk.push_stack();
        let mut acceleratable_keyword = compare_and_advance(walk, TokenType::AcceleratableKeyword);
        let serial_keyword = compare_and_advance(walk, TokenType::SerialKeyword);

        // This is called twice in order to provide better user error feedback in case
        // the user wrote them in the opposite order.
        acceleratable_keyword |= compare_and_advance(walk, TokenType::AcceleratableKeyword);

        if acceleratable_keyword && serial_keyword {
            walk.pop_stack(false);
            let token = walk.peek().unwrap();
            return Err(format!("A function cannot be serial and acceleratable at the same time! Error at {}:{}",
                    token.line, token.col));
        } else {
            walk.pop_stack(true);
        }

        let expect_function = expect_token(walk, TokenType::FunctionKeyword);
        return_if!(expect_function.is_err(), Err(expect_function.unwrap_err()));

        let expect_func_name = expect_token(walk, TokenType::Name);
        return_if!(expect_func_name.is_err(), Err(expect_func_name.unwrap_err()));
        let function_name = expect_func_name.unwrap();

        walk.context.add_function(function_name, walk.module_path, serial_keyword, exportable, acceleratable_keyword);
        return Ok(true);
    })
}

#[cfg(test)]
mod tests {
    use crate::compiler::token_reader::read_all_tokens;
    use super::*;

    #[test]
    fn walk_simple_hello_world() {
        let code = "export serial function main() {
                      serial print('Hello, world!')
                    }";

        let tokens = read_all_tokens(code);

        let mut context = Context::new();
        let result = evaluate_token_list(&mut context, "my_mod", &tokens);
        assert_eq!(result.is_err(), false);

        let module = context.find_module(&String::from("my_mod")).unwrap();
        let function = context.find_function(module.path, &String::from("main")).unwrap();
        assert_eq!(function.exportable, true);
        assert_eq!(function.serial, true);
        assert_eq!(function.acceleratable, false);
    }
}
